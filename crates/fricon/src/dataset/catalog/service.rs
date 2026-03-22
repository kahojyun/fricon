use std::{path::Path, sync::Arc};

use tracing::{error, info, instrument, warn};

use crate::{
    dataset::{
        NormalizedTag,
        catalog::{CatalogError, DatasetCatalogRepository},
        events::{DatasetEvent, DatasetEventPublisher},
        model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
        portability::{self, ImportPreview},
        storage,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub(crate) struct DatasetCatalogService {
    repository: Arc<dyn DatasetCatalogRepository>,
    paths: WorkspacePaths,
}

impl DatasetCatalogService {
    #[must_use]
    pub(crate) fn new(
        repository: Arc<dyn DatasetCatalogRepository>,
        paths: WorkspacePaths,
    ) -> Self {
        Self { repository, paths }
    }

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub(crate) fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError> {
        self.repository.get_dataset(id)
    }

    #[instrument(skip(self, query_options))]
    pub(crate) fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError> {
        self.repository.list_datasets(query_options)
    }

    #[instrument(skip(self))]
    pub(crate) fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError> {
        self.repository.list_dataset_tags()
    }

    #[instrument(skip(self, events, update_payload), fields(dataset.id = id))]
    pub(crate) fn update_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        update_payload: DatasetUpdate,
        events: &P,
    ) -> Result<(), CatalogError> {
        self.repository.update_dataset(id, update_payload)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self, events, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub(crate) fn add_tags<P: DatasetEventPublisher>(
        &self,
        id: i32,
        tags: Vec<String>,
        events: &P,
    ) -> Result<(), CatalogError> {
        let tags = NormalizedTag::parse_many(tags);
        if tags.is_empty() {
            return Ok(());
        }
        self.repository.add_tags(id, &tags)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self, events, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub(crate) fn remove_tags<P: DatasetEventPublisher>(
        &self,
        id: i32,
        tags: Vec<String>,
        events: &P,
    ) -> Result<(), CatalogError> {
        let tags = NormalizedTag::parse_many(tags);
        if tags.is_empty() {
            return Ok(());
        }
        self.repository.remove_tags(id, &tags)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self, events), fields(dataset.id = id))]
    pub(crate) fn delete_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        events: &P,
    ) -> Result<(), CatalogError> {
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        Self::ensure_not_deleted_record(&record)?;
        if record.metadata.trashed_at.is_none() {
            return Err(anyhow::anyhow!(
                "dataset must be moved to trash before permanent deletion"
            )
            .into());
        }

        let dataset_path = self.paths.dataset_path_from_uid(record.metadata.uid);
        let graveyard_path = self
            .paths
            .graveyard_dataset_path_from_uid(record.metadata.uid);
        if dataset_path.exists() {
            storage::move_dataset(&dataset_path, &graveyard_path).inspect_err(|e| {
                error!(error = %e, dataset.id = id, "Dataset graveyard staging failed");
            })?;
        }

        let deleted_record = self.repository.mark_dataset_deleted(id)?;
        events.publish(DatasetEvent::Updated(deleted_record));

        if let Err(error) = storage::delete_dataset(&graveyard_path) {
            error!(
                error = %error,
                dataset.id = id,
                "Dataset graveyard cleanup failed"
            );
        }
        Ok(())
    }

    #[instrument(skip(self, events), fields(dataset.id = id))]
    pub(crate) fn trash_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        events: &P,
    ) -> Result<(), CatalogError> {
        self.ensure_not_deleted(id)?;
        self.repository.trash_dataset(id)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self, events), fields(dataset.id = id))]
    pub(crate) fn restore_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        events: &P,
    ) -> Result<(), CatalogError> {
        self.ensure_not_deleted(id)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        if record.metadata.trashed_at.is_none() {
            return Ok(());
        }
        self.repository.restore_dataset(id)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) fn reconcile_deleted_datasets(&self) -> Result<usize, CatalogError> {
        let records = self.repository.list_all_datasets_including_deleted()?;
        let mut reconciled = 0;

        for record in records {
            if record.metadata.deleted_at.is_some() {
                continue;
            }

            let dataset_path = self.paths.dataset_path_from_uid(record.metadata.uid);
            let graveyard_path = self
                .paths
                .graveyard_dataset_path_from_uid(record.metadata.uid);
            let live_exists = dataset_path.exists();
            let graveyard_exists = graveyard_path.exists();

            if !live_exists && graveyard_exists {
                self.repository.mark_dataset_deleted(record.id)?;
                reconciled += 1;
                continue;
            }

            if live_exists && graveyard_exists {
                warn!(
                    dataset.id = record.id,
                    "Dataset exists in both live storage and graveyard; leaving unchanged"
                );
            }
        }

        if reconciled > 0 {
            info!(count = reconciled, "Reconciled deleted dataset tombstones");
        }

        Ok(reconciled)
    }

    #[instrument(skip(self))]
    pub(crate) fn garbage_collect_deleted_datasets(&self) -> Result<usize, CatalogError> {
        let records = self.repository.list_deleted_datasets()?;
        let mut deleted_count = 0;

        for record in records {
            let graveyard_path = self
                .paths
                .graveyard_dataset_path_from_uid(record.metadata.uid);
            if !graveyard_path.exists() {
                continue;
            }

            match storage::delete_dataset(&graveyard_path) {
                Ok(()) => deleted_count += 1,
                Err(error) => {
                    error!(
                        error = %error,
                        dataset.id = record.id,
                        "Deleted dataset graveyard cleanup failed"
                    );
                }
            }
        }

        if deleted_count > 0 {
            info!(
                count = deleted_count,
                "Garbage collected deleted dataset payloads"
            );
        }

        Ok(deleted_count)
    }

    #[instrument(skip(self, tag), fields(tag.name = %tag))]
    pub(crate) fn delete_tag(&self, tag: String) -> Result<(), CatalogError> {
        let tag = NormalizedTag::parse(tag)?;
        self.repository.delete_tag(&tag)
    }

    #[instrument(skip(self, old_name, new_name), fields(tag.old = %old_name, tag.new = %new_name))]
    pub(crate) fn rename_tag(
        &self,
        old_name: String,
        new_name: String,
    ) -> Result<(), CatalogError> {
        let old_name = NormalizedTag::parse(old_name)?;
        let new_name = NormalizedTag::parse(new_name)?;
        if old_name == new_name {
            return Err(anyhow::anyhow!("old tag name and new tag name must differ").into());
        }
        self.repository.rename_tag(&old_name, &new_name)
    }

    #[instrument(skip(self, source, target), fields(tag.source = %source, tag.target = %target))]
    pub(crate) fn merge_tag(&self, source: String, target: String) -> Result<(), CatalogError> {
        let source = NormalizedTag::parse(source)?;
        let target = NormalizedTag::parse(target)?;
        if source == target {
            return Err(anyhow::anyhow!("source tag and target tag must differ").into());
        }
        self.repository.merge_tag(&source, &target)
    }

    fn ensure_not_deleted(&self, id: i32) -> Result<(), CatalogError> {
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        Self::ensure_not_deleted_record(&record)
    }

    fn ensure_not_deleted_record(record: &DatasetRecord) -> Result<(), CatalogError> {
        if record.metadata.deleted_at.is_some() {
            return Err(CatalogError::Deleted {
                id: record.id.to_string(),
            });
        }
        Ok(())
    }

    // ── Portability ──────────────────────────────────────────────────────────

    /// Export a dataset to a tar+zstd archive in `output_dir`.
    ///
    /// Returns the path of the created archive.
    #[instrument(skip(self, id, output_dir), fields(dataset.id = ?id))]
    pub(crate) fn export_dataset(
        &self,
        id: DatasetId,
        output_dir: &Path,
    ) -> Result<std::path::PathBuf, CatalogError> {
        let record = self.repository.get_dataset(id)?;
        let dataset_dir = self.paths.dataset_path_from_uid(record.metadata.uid);
        portability::export_dataset(&record.metadata, &dataset_dir, output_dir)
            .map_err(|e| CatalogError::from(anyhow::Error::from(e)))
    }

    /// Inspect an archive and return metadata + optional conflict info without
    /// modifying any state.
    #[instrument(skip(self, archive_path))]
    pub(crate) fn preview_import(
        &self,
        archive_path: &Path,
    ) -> Result<ImportPreview, CatalogError> {
        // Peek at metadata to find the uuid.
        let preview = portability::preview_import(archive_path, None)
            .map_err(|e| CatalogError::from(anyhow::Error::from(e)))?;
        // Check whether that uuid is already in the DB.
        let existing_record = self
            .repository
            .find_dataset_by_uid(preview.metadata.uid)?;
        if let Some(record) = existing_record {
            // Re-run with existing metadata so the diff is populated.
            let preview_with_conflict =
                portability::preview_import(archive_path, Some(&record.metadata))
                    .map_err(|e| CatalogError::from(anyhow::Error::from(e)))?;
            return Ok(preview_with_conflict);
        }
        Ok(preview)
    }

    /// Import a dataset from a tar+zstd archive.
    ///
    /// If `force` is `false` and a dataset with the same uuid already exists,
    /// an error is returned.  Set `force = true` to replace the existing data.
    #[instrument(skip(self, archive_path, events))]
    pub(crate) fn import_dataset<P: DatasetEventPublisher>(
        &self,
        archive_path: &Path,
        force: bool,
        events: &P,
    ) -> Result<DatasetRecord, CatalogError> {
        // Peek metadata from archive to resolve dest dir and check conflict.
        let preview = portability::preview_import(archive_path, None)
            .map_err(|e| CatalogError::from(anyhow::Error::from(e)))?;
        let uid = preview.metadata.uid;
        let dest_dir = self.paths.dataset_path_from_uid(uid);

        // Determine if an existing record matches.
        let existing_record = self.repository.find_dataset_by_uid(uid)?;
        if existing_record.is_some() && !force {
            return Err(CatalogError::from(anyhow::Error::from(
                portability::PortabilityError::UuidConflict { uid },
            )));
        }

        let staged = portability::stage_import(archive_path, &dest_dir)
            .map_err(|e| CatalogError::from(anyhow::Error::from(e)))?;

        let backup_dir = match portability::promote_staged_import(
            &staged.staging_dir,
            &dest_dir,
            force,
            uid,
        ) {
            Ok(backup_dir) => backup_dir,
            Err(error) => {
                let _ = portability::discard_staged_import(&staged.staging_dir);
                return Err(CatalogError::from(anyhow::Error::from(error)));
            }
        };

        let result = match existing_record {
            Some(existing_record) => {
                let record = self
                    .repository
                    .replace_imported_dataset_record(existing_record.id, &staged.metadata);
                match record {
                    Ok(record) => {
                        portability::finalize_promoted_import(backup_dir.as_deref()).map_err(
                            |error| CatalogError::from(anyhow::Error::from(error)),
                        )?;
                        info!(
                            dataset.id = record.id,
                            uid = %uid,
                            "Dataset force-imported from archive"
                        );
                        events.publish(DatasetEvent::Updated(record.clone()));
                        Ok(record)
                    }
                    Err(error) => {
                        if let Err(rollback_error) = portability::rollback_promoted_import(
                            &dest_dir,
                            backup_dir.as_deref(),
                        ) {
                            error!(
                                error = %rollback_error,
                                uid = %uid,
                                "Failed to roll back dataset import after repository error"
                            );
                        }
                        Err(error)
                    }
                }
            }
            None => {
                let record = self
                    .repository
                    .insert_imported_dataset_record(&staged.metadata);
                match record {
                    Ok(record) => {
                        info!(
                            dataset.id = record.id,
                            uid = %uid,
                            "Dataset imported from archive"
                        );
                        events.publish(DatasetEvent::Created(record.clone()));
                        Ok(record)
                    }
                    Err(error) => {
                        if let Err(rollback_error) =
                            portability::rollback_promoted_import(&dest_dir, None)
                        {
                            error!(
                                error = %rollback_error,
                                uid = %uid,
                                "Failed to clean up dataset import after repository error"
                            );
                        }
                        Err(error)
                    }
                }
            }
        };

        if let Err(error) = portability::discard_staged_import(&staged.staging_dir) {
            warn!(
                error = %error,
                uid = %uid,
                "Failed to discard staged dataset import directory"
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        fs,
        sync::{Arc, Mutex},
    };

    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;
    use crate::dataset::{
        catalog::MockDatasetCatalogRepository,
        events::DatasetEvent,
        model::{DatasetId, DatasetMetadata, DatasetStatus},
        portability,
    };

    #[derive(Default)]
    struct CollectEvents {
        events: Mutex<Vec<DatasetEvent>>,
    }

    impl DatasetEventPublisher for CollectEvents {
        fn publish(&self, event: DatasetEvent) {
            self.events
                .lock()
                .expect("events mutex should lock")
                .push(event);
        }
    }

    fn dataset_record(id: i32, uid: Uuid) -> DatasetRecord {
        DatasetRecord {
            id,
            metadata: DatasetMetadata {
                uid,
                name: format!("dataset-{id}"),
                description: String::new(),
                favorite: false,
                status: DatasetStatus::Completed,
                created_at: Utc::now(),
                trashed_at: Some(Utc::now()),
                deleted_at: None,
                tags: Vec::new(),
            },
        }
    }

    fn create_import_archive(root: &Path, uid: Uuid, name: &str) -> std::path::PathBuf {
        let source_dir = root.join("source");
        fs::create_dir_all(&source_dir).expect("source dir");
        fs::write(source_dir.join("data_chunk_0.arrow"), b"NEW").expect("source payload");
        let metadata = DatasetMetadata {
            uid,
            name: name.to_string(),
            description: format!("imported {name}"),
            favorite: true,
            status: DatasetStatus::Completed,
            created_at: Utc::now(),
            trashed_at: None,
            deleted_at: None,
            tags: vec!["alpha".to_string(), "beta".to_string()],
        };
        let output_dir = root.join("exports");
        portability::export_dataset(&metadata, &source_dir, &output_dir).expect("archive export")
    }

    #[test]
    fn reconcile_deleted_datasets_marks_graveyard_entries_as_deleted() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let graveyard_path = paths.graveyard_dataset_path_from_uid(uid);

        fs::create_dir_all(&graveyard_path).expect("graveyard dataset should be created");

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_list_all_datasets_including_deleted()
            .once()
            .return_once(move || Ok(vec![dataset_record(1, uid)]));
        repository
            .expect_mark_dataset_deleted()
            .once()
            .withf(|id| *id == 1)
            .return_once(move |_| Ok(dataset_record(1, uid)));

        let service = DatasetCatalogService::new(Arc::new(repository), paths);

        let reconciled = service
            .reconcile_deleted_datasets()
            .expect("reconciliation should succeed");

        assert_eq!(reconciled, 1);
    }

    #[test]
    fn delete_dataset_marks_record_deleted_when_live_directory_is_missing() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let record = dataset_record(1, uid);
        let deleted_record = DatasetRecord {
            metadata: DatasetMetadata {
                deleted_at: Some(Utc::now()),
                ..record.metadata.clone()
            },
            ..record.clone()
        };

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_get_dataset()
            .once()
            .withf(|id| matches!(id, DatasetId::Id(1)))
            .return_once(move |_| Ok(record));
        repository
            .expect_mark_dataset_deleted()
            .once()
            .withf(|id| *id == 1)
            .return_once(move |_| Ok(deleted_record.clone()));

        let service = DatasetCatalogService::new(Arc::new(repository), paths);
        let events = CollectEvents::default();

        service
            .delete_dataset(1, &events)
            .expect("delete should succeed when live directory is already missing");

        let published = events.events.lock().expect("events mutex should lock");
        assert_eq!(published.len(), 1);
        match &published[0] {
            DatasetEvent::Updated(record) => assert_eq!(record.id, 1),
            DatasetEvent::Created(record) => {
                panic!("unexpected created event for dataset {}", record.id)
            }
        }
    }

    #[test]
    fn import_dataset_without_force_returns_conflict_before_touching_storage() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "conflict");
        let existing_record = dataset_record(7, uid);

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(Some(existing_record)));

        let service = DatasetCatalogService::new(Arc::new(repository), paths.clone());
        let events = CollectEvents::default();

        let result = service.import_dataset(&archive, false, &events);

        assert!(result.is_err(), "import should return a conflict");
        assert!(
            !paths.dataset_path_from_uid(uid).exists(),
            "live dataset path should not be created on conflict"
        );
        assert!(
            events.events.lock().expect("events").is_empty(),
            "no events should be published on conflict"
        );
    }

    #[test]
    fn force_import_reuses_existing_record_and_publishes_updated() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "replacement");
        let live_dir = paths.dataset_path_from_uid(uid);
        fs::create_dir_all(&live_dir).expect("live dir");
        fs::write(live_dir.join("old.arrow"), b"OLD").expect("live payload");

        let existing_record = dataset_record(7, uid);
        let replaced_record = DatasetRecord {
            id: 7,
            metadata: DatasetMetadata {
                uid,
                name: "replacement".to_string(),
                description: "imported replacement".to_string(),
                favorite: true,
                status: DatasetStatus::Completed,
                created_at: Utc::now(),
                trashed_at: None,
                deleted_at: None,
                tags: vec!["alpha".to_string(), "beta".to_string()],
            },
        };

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(Some(existing_record)));
        repository
            .expect_replace_imported_dataset_record()
            .once()
            .withf(move |id, metadata| {
                *id == 7 && metadata.uid == uid && metadata.name == "replacement"
            })
            .return_once(move |_, _| Ok(replaced_record.clone()));

        let service = DatasetCatalogService::new(Arc::new(repository), paths.clone());
        let events = CollectEvents::default();

        let record = service
            .import_dataset(&archive, true, &events)
            .expect("force import should succeed");

        assert_eq!(record.id, 7, "force import should reuse the existing record id");
        assert!(
            live_dir.join("data_chunk_0.arrow").exists(),
            "new payload should be promoted into the live path"
        );
        assert!(
            !live_dir.join("old.arrow").exists(),
            "old payload should be replaced"
        );
        let published = events.events.lock().expect("events");
        assert_eq!(published.len(), 1);
        match &published[0] {
            DatasetEvent::Updated(record) => assert_eq!(record.id, 7),
            DatasetEvent::Created(record) => {
                panic!("unexpected created event for dataset {}", record.id)
            }
        }
    }

    #[test]
    fn force_import_revives_deleted_record_and_publishes_updated() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "revived");

        let mut existing_record = dataset_record(9, uid);
        existing_record.metadata.trashed_at = Some(Utc::now());
        existing_record.metadata.deleted_at = Some(Utc::now());

        let revived_record = DatasetRecord {
            id: 9,
            metadata: DatasetMetadata {
                uid,
                name: "revived".to_string(),
                description: "imported revived".to_string(),
                favorite: true,
                status: DatasetStatus::Completed,
                created_at: Utc::now(),
                trashed_at: None,
                deleted_at: None,
                tags: vec!["alpha".to_string(), "beta".to_string()],
            },
        };

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(Some(existing_record)));
        repository
            .expect_replace_imported_dataset_record()
            .once()
            .withf(move |id, metadata| {
                *id == 9 && metadata.uid == uid && metadata.name == "revived"
            })
            .return_once(move |_, _| Ok(revived_record.clone()));

        let service = DatasetCatalogService::new(Arc::new(repository), paths.clone());
        let events = CollectEvents::default();

        let record = service
            .import_dataset(&archive, true, &events)
            .expect("force import should revive deleted record");

        assert_eq!(record.id, 9);
        assert!(
            paths
                .dataset_path_from_uid(uid)
                .join("data_chunk_0.arrow")
                .exists(),
            "revived dataset should have live payload"
        );
        let published = events.events.lock().expect("events");
        assert_eq!(published.len(), 1);
        match &published[0] {
            DatasetEvent::Updated(record) => assert_eq!(record.id, 9),
            DatasetEvent::Created(record) => {
                panic!("unexpected created event for dataset {}", record.id)
            }
        }
    }

    #[test]
    fn import_dataset_rolls_back_live_dir_when_repository_replace_fails() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "rollback");
        let live_dir = paths.dataset_path_from_uid(uid);
        fs::create_dir_all(&live_dir).expect("live dir");
        fs::write(live_dir.join("old.arrow"), b"OLD").expect("live payload");

        let existing_record = dataset_record(11, uid);
        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(Some(existing_record)));
        repository
            .expect_replace_imported_dataset_record()
            .once()
            .return_once(move |_, _| Err(anyhow::anyhow!("replace failed").into()));

        let service = DatasetCatalogService::new(Arc::new(repository), paths);
        let events = CollectEvents::default();

        let result = service.import_dataset(&archive, true, &events);

        assert!(result.is_err(), "force import should fail when repository replace fails");
        assert!(
            live_dir.join("old.arrow").exists(),
            "old payload should be restored after rollback"
        );
        assert!(
            !live_dir.join("data_chunk_0.arrow").exists(),
            "promoted payload should be removed after rollback"
        );
        assert!(
            events.events.lock().expect("events").is_empty(),
            "no events should be published on rollback"
        );
    }
}

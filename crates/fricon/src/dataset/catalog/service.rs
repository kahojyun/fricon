//! Dataset catalog service - orchestrates repository, filesystem, and event
//! side effects for dataset lifecycle operations.
//!
//! # Ownership
//!
//! This service owns the high-level dataset lifecycle (CRUD, trash/restore,
//! delete, import/export, tag management, reconciliation, and garbage
//! collection). It coordinates three collaborators:
//!
//! - **Repository** ([`DatasetCatalogRepository`]): owns database state.
//! - **Storage** ([`storage`]): owns live and graveyard filesystem layouts.
//! - **Events** ([`DatasetEventPublisher`]): notifies downstream consumers
//!   after successful state changes.
//!
//! # Sequencing & rollback conventions
//!
//! Multi-step workflows (delete, import) follow a stage -> commit -> finalize
//! pattern so that a failure at any point leaves the system in a recoverable
//! state. See individual methods for step-by-step sequencing notes.
//!
//! # Extension notes
//!
//! - Adding a field to [`DatasetRecord`] / [`DatasetMetadata`] may require
//!   updates in [`ExportedMetadata`], [`portability::compute_diffs`], and the
//!   repository adapter in `database::dataset`.
//! - New event variants should be published only after the primary state change
//!   has succeeded.

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

/// Stateless service coordinating dataset catalog operations.
///
/// Holds a shared repository handle and workspace paths. All mutation methods
/// accept an `&P: DatasetEventPublisher` so the caller controls event
/// dispatch lifetime.
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
        let record = self.repository.get_dataset(id)?;
        Self::ensure_not_deleted_record(&record)?;
        Ok(record)
    }

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub(crate) fn get_dataset_including_deleted(
        &self,
        id: DatasetId,
    ) -> Result<DatasetRecord, CatalogError> {
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

    /// Permanently delete a dataset that is already in trash.
    ///
    /// # Preconditions
    ///
    /// The dataset must be trashed (`trashed_at` set) and not yet deleted.
    ///
    /// # Sequencing
    ///
    /// 1. Move live directory -> graveyard (filesystem).
    /// 2. Mark record deleted (database) and publish `Updated` event.
    /// 3. Best-effort graveyard cleanup (filesystem). Failures are logged but
    ///    do not fail the operation; `garbage_collect_deleted_datasets` will
    ///    retry later.
    #[instrument(skip(self, events), fields(dataset.id = id))]
    pub(crate) fn delete_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        events: &P,
    ) -> Result<(), CatalogError> {
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        Self::ensure_not_deleted_record(&record)?;
        if record.metadata.trashed_at.is_none() {
            return Err(CatalogError::NotTrashed);
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

    /// Scan all non-deleted datasets and mark any that only exist in the
    /// graveyard as deleted.
    ///
    /// This repairs inconsistencies where the live directory was removed
    /// (e.g. external filesystem changes) but the database record was not
    /// tombstoned. Datasets present in both live and graveyard are left
    /// untouched and logged as warnings.
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

    /// Remove graveyard directories for datasets already marked deleted in
    /// the database.
    ///
    /// Best-effort: individual failures are logged but do not abort the sweep.
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
        let tag = NormalizedTag::parse(tag).map_err(|_| CatalogError::EmptyTag)?;
        self.repository.delete_tag(&tag)
    }

    #[instrument(skip(self, old_name, new_name), fields(tag.old = %old_name, tag.new = %new_name))]
    pub(crate) fn rename_tag(
        &self,
        old_name: String,
        new_name: String,
    ) -> Result<(), CatalogError> {
        let old_name = NormalizedTag::parse(old_name).map_err(|_| CatalogError::EmptyTag)?;
        let new_name = NormalizedTag::parse(new_name).map_err(|_| CatalogError::EmptyTag)?;
        if old_name == new_name {
            return Err(CatalogError::SameTagName);
        }
        self.repository.rename_tag(&old_name, &new_name)
    }

    #[instrument(skip(self, source, target), fields(tag.source = %source, tag.target = %target))]
    pub(crate) fn merge_tag(&self, source: String, target: String) -> Result<(), CatalogError> {
        let source = NormalizedTag::parse(source).map_err(|_| CatalogError::EmptyTag)?;
        let target = NormalizedTag::parse(target).map_err(|_| CatalogError::EmptyTag)?;
        if source == target {
            return Err(CatalogError::SameSourceTarget);
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

    /// Export a dataset to a tar+zstd archive in `output_dir`.
    ///
    /// This is a read-only catalog operation. It does not change repository
    /// state or publish dataset events.
    #[instrument(skip(self, id, output_dir), fields(dataset.id = ?id))]
    pub(crate) fn export_dataset(
        &self,
        id: DatasetId,
        output_dir: &Path,
    ) -> Result<std::path::PathBuf, CatalogError> {
        let record = self.repository.get_dataset(id)?;
        Self::ensure_not_deleted_record(&record)?;
        let dataset_dir = self.paths.dataset_path_from_uid(record.metadata.uid);
        portability::export_dataset(&record.metadata, &dataset_dir, output_dir)
            .map_err(CatalogError::from)
    }

    /// Inspect an archive and return metadata + optional conflict info without
    /// modifying any state.
    ///
    /// If the archive uuid already exists in the repository, this method
    /// enriches the preview with field-level diffs against the live record.
    #[instrument(skip(self, archive_path))]
    pub(crate) fn preview_import(
        &self,
        archive_path: &Path,
    ) -> Result<ImportPreview, CatalogError> {
        // Peek at metadata to find the uuid.
        let preview =
            portability::preview_import(archive_path, None).map_err(CatalogError::from)?;
        // Check whether that uuid is already in the DB.
        let existing_record = self.repository.find_dataset_by_uid(preview.metadata.uid)?;
        if let Some(record) = existing_record {
            // Re-run with existing metadata so the diff is populated.
            let preview_with_conflict =
                portability::preview_import(archive_path, Some(&record.metadata))
                    .map_err(CatalogError::from)?;
            return Ok(preview_with_conflict);
        }
        Ok(preview)
    }

    /// Import a dataset from a tar+zstd archive.
    ///
    /// If `force` is `false` and a dataset with the same uuid already exists,
    /// an error is returned. Set `force = true` to replace the existing data.
    ///
    /// The workflow is:
    /// 1. preview archive metadata to resolve the destination uid
    /// 2. stage archive files without touching the live directory
    /// 3. promote the staged payload into the live directory
    /// 4. insert or replace the repository record
    /// 5. publish the resulting dataset event
    ///
    /// If repository updates fail after promotion, the live filesystem state is
    /// rolled back before the error is returned.
    #[instrument(skip(self, archive_path, events))]
    pub(crate) fn import_dataset<P: DatasetEventPublisher>(
        &self,
        archive_path: &Path,
        force: bool,
        events: &P,
    ) -> Result<DatasetRecord, CatalogError> {
        // Peek metadata from archive to resolve dest dir and check conflict.
        let preview =
            portability::preview_import(archive_path, None).map_err(CatalogError::from)?;
        let uid = preview.metadata.uid;
        let dest_dir = self.paths.dataset_path_from_uid(uid);

        // Determine if an existing record matches.
        let existing_record = self.repository.find_dataset_by_uid(uid)?;
        if existing_record.is_some() && !force {
            return Err(CatalogError::from(
                portability::PortabilityError::UuidConflict { uid },
            ));
        }

        let staged =
            portability::stage_import(archive_path, &dest_dir).map_err(CatalogError::from)?;

        let backup_dir =
            match portability::promote_staged_import(&staged.staging_dir, &dest_dir, force, uid) {
                Ok(backup_dir) => backup_dir,
                Err(error) => {
                    let _ = portability::discard_staged_import(&staged.staging_dir);
                    return Err(CatalogError::from(error));
                }
            };

        let result = if let Some(existing_record) = existing_record {
            self.finish_replaced_import(
                &existing_record,
                &staged.metadata,
                &dest_dir,
                backup_dir.as_ref(),
                uid,
                events,
            )
        } else {
            self.finish_inserted_import(
                &staged.metadata,
                &dest_dir,
                backup_dir.as_ref(),
                uid,
                events,
            )
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

    /// Complete a force-import that replaces an existing dataset record.
    ///
    /// On repository success: finalizes backup, cleans up any stale graveyard
    /// entry (for previously deleted datasets being revived), and publishes
    /// an `Updated` event.
    ///
    /// On repository failure: rolls back filesystem to the pre-import state.
    fn finish_replaced_import<P: DatasetEventPublisher>(
        &self,
        existing_record: &DatasetRecord,
        metadata: &portability::ExportedMetadata,
        dest_dir: &Path,
        backup_dir: Option<&std::path::PathBuf>,
        uid: uuid::Uuid,
        events: &P,
    ) -> Result<DatasetRecord, CatalogError> {
        let revive_graveyard_dir = existing_record
            .metadata
            .deleted_at
            .is_some()
            .then(|| self.paths.graveyard_dataset_path_from_uid(uid));
        let record = self
            .repository
            .replace_imported_dataset_record(existing_record.id, metadata);
        match record {
            Ok(record) => {
                Self::finalize_import_backup(uid, backup_dir.map(std::path::PathBuf::as_path));
                Self::cleanup_revived_graveyard(uid, revive_graveyard_dir.as_deref());
                info!(
                    dataset.id = record.id,
                    uid = %uid,
                    "Dataset force-imported from archive"
                );
                events.publish(DatasetEvent::Updated(record.clone()));
                Ok(record)
            }
            Err(error) => {
                Self::rollback_import(
                    dest_dir,
                    backup_dir.map(std::path::PathBuf::as_path),
                    uid,
                    "Failed to roll back dataset import after repository error",
                );
                Err(error)
            }
        }
    }

    /// Complete an import that inserts a new dataset record.
    ///
    /// On repository success: finalizes backup and publishes a `Created` event.
    /// On repository failure: rolls back filesystem to the pre-import state.
    fn finish_inserted_import<P: DatasetEventPublisher>(
        &self,
        metadata: &portability::ExportedMetadata,
        dest_dir: &Path,
        backup_dir: Option<&std::path::PathBuf>,
        uid: uuid::Uuid,
        events: &P,
    ) -> Result<DatasetRecord, CatalogError> {
        let record = self.repository.insert_imported_dataset_record(metadata);
        match record {
            Ok(record) => {
                Self::finalize_import_backup(uid, backup_dir.map(std::path::PathBuf::as_path));
                info!(
                    dataset.id = record.id,
                    uid = %uid,
                    "Dataset imported from archive"
                );
                events.publish(DatasetEvent::Created(record.clone()));
                Ok(record)
            }
            Err(error) => {
                Self::rollback_import(
                    dest_dir,
                    backup_dir.map(std::path::PathBuf::as_path),
                    uid,
                    "Failed to clean up dataset import after repository error",
                );
                Err(error)
            }
        }
    }

    /// Best-effort removal of the displaced backup directory after a
    /// successful import. Failures are logged but do not fail the import.
    fn finalize_import_backup(uid: uuid::Uuid, backup_dir: Option<&Path>) {
        if let Err(error) = portability::finalize_promoted_import(backup_dir) {
            warn!(
                error = %error,
                uid = %uid,
                "Failed to finalize import backup cleanup; import succeeded"
            );
        }
    }

    /// Best-effort removal of graveyard data for a dataset that was
    /// previously deleted and is now being revived by an import.
    fn cleanup_revived_graveyard(uid: uuid::Uuid, graveyard_dir: Option<&Path>) {
        if let Some(graveyard_dir) = graveyard_dir
            && let Err(error) = storage::delete_dataset(graveyard_dir)
        {
            warn!(
                error = %error,
                uid = %uid,
                "Failed to remove stale graveyard payload after reviving dataset"
            );
        }
    }

    /// Roll back a promoted import after a repository error by restoring the
    /// previous live directory (if backed up) or removing the newly promoted
    /// directory. Rollback failures are logged as errors.
    fn rollback_import(dest_dir: &Path, backup_dir: Option<&Path>, uid: uuid::Uuid, message: &str) {
        if let Err(rollback_error) = portability::rollback_promoted_import(dest_dir, backup_dir) {
            error!(
                error = %rollback_error,
                uid = %uid,
                "{message}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
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
    fn get_dataset_rejects_deleted_dataset() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let mut record = dataset_record(5, uid);
        record.metadata.deleted_at = Some(Utc::now());

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_get_dataset()
            .once()
            .withf(|id| matches!(id, DatasetId::Id(5)))
            .return_once(move |_| Ok(record));

        let service = DatasetCatalogService::new(Arc::new(repository), paths);

        let result = service.get_dataset(DatasetId::Id(5));

        assert!(
            matches!(result, Err(CatalogError::Deleted { .. })),
            "deleted datasets should not be retrievable"
        );
    }

    #[test]
    fn export_dataset_rejects_deleted_dataset() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let mut record = dataset_record(5, uid);
        record.metadata.deleted_at = Some(Utc::now());

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_get_dataset()
            .once()
            .withf(|id| matches!(id, DatasetId::Id(5)))
            .return_once(move |_| Ok(record));

        let service = DatasetCatalogService::new(Arc::new(repository), paths.clone());
        let export_dir = temp_dir.path().join("exports");

        let result = service.export_dataset(DatasetId::Id(5), &export_dir);

        assert!(
            matches!(result, Err(CatalogError::Deleted { .. })),
            "deleted datasets should not be exportable"
        );
        assert!(
            !export_dir.exists(),
            "export directory should not be created when export is rejected"
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

        assert_eq!(
            record.id, 7,
            "force import should reuse the existing record id"
        );
        assert!(
            live_dir.join("data_chunk_0.arrow").exists(),
            "new payload should be promoted into the live path"
        );
        assert!(
            !live_dir.join("old.arrow").exists(),
            "old payload should be replaced"
        );
        assert!(
            !live_dir.join("metadata.json").exists(),
            "live payload should not retain archive metadata sidecar files"
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
        let graveyard_dir = paths.graveyard_dataset_path_from_uid(uid);
        fs::create_dir_all(&graveyard_dir).expect("graveyard dir");
        fs::write(graveyard_dir.join("old.arrow"), b"OLD").expect("graveyard payload");

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
        assert!(
            !graveyard_dir.exists(),
            "revived dataset should clean up any stale graveyard payload"
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
            .return_once(move |_, _| {
                Err(CatalogError::NotFound {
                    id: "mock".to_string(),
                })
            });

        let service = DatasetCatalogService::new(Arc::new(repository), paths);
        let events = CollectEvents::default();

        let result = service.import_dataset(&archive, true, &events);

        assert!(
            result.is_err(),
            "force import should fail when repository replace fails"
        );
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

    #[test]
    fn import_dataset_cleans_orphaned_live_dir_when_forced_insert_succeeds() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "imported");
        let live_dir = paths.dataset_path_from_uid(uid);
        fs::create_dir_all(&live_dir).expect("live dir");
        fs::write(live_dir.join("orphaned.arrow"), b"OLD").expect("orphaned payload");

        let inserted_record = DatasetRecord {
            id: 13,
            metadata: DatasetMetadata {
                uid,
                name: "imported".to_string(),
                description: "imported imported".to_string(),
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
            .return_once(move |_| Ok(None));
        repository
            .expect_insert_imported_dataset_record()
            .once()
            .withf(move |metadata| metadata.uid == uid && metadata.name == "imported")
            .return_once(move |_| Ok(inserted_record.clone()));

        let service = DatasetCatalogService::new(Arc::new(repository), paths);
        let events = CollectEvents::default();

        let record = service
            .import_dataset(&archive, true, &events)
            .expect("forced import should succeed");

        assert_eq!(record.id, 13);
        assert!(
            live_dir.join("data_chunk_0.arrow").exists(),
            "imported payload should be promoted into the live path"
        );
        assert!(
            !live_dir.join("orphaned.arrow").exists(),
            "orphaned live payload should be removed after successful import"
        );
        let siblings = fs::read_dir(
            live_dir
                .parent()
                .expect("dataset path should have a parent directory"),
        )
        .expect("data shard directory should be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("data shard directory entries");
        assert!(
            siblings.iter().all(|entry| {
                !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("import-backup")
            }),
            "forced import should clean up any temporary backup directory"
        );
        let published = events.events.lock().expect("events");
        assert_eq!(published.len(), 1);
        match &published[0] {
            DatasetEvent::Created(record) => assert_eq!(record.id, 13),
            DatasetEvent::Updated(record) => {
                panic!("unexpected updated event for dataset {}", record.id)
            }
        }
    }

    #[test]
    fn import_dataset_without_force_returns_filesystem_conflict_for_orphaned_live_dir() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "orphaned");
        let live_dir = paths.dataset_path_from_uid(uid);
        fs::create_dir_all(&live_dir).expect("live dir");
        fs::write(live_dir.join("orphaned.arrow"), b"OLD").expect("orphaned payload");

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(None));

        let service = DatasetCatalogService::new(Arc::new(repository), paths.clone());
        let events = CollectEvents::default();

        let result = service.import_dataset(&archive, false, &events);
        let error = result.expect_err("orphaned live dir should block non-force import");
        let error_text = error.to_string();

        assert!(
            error_text.contains("storage directory already exists"),
            "error should describe the on-disk conflict: {error_text}"
        );
        assert!(
            live_dir.join("orphaned.arrow").exists(),
            "existing on-disk payload should remain untouched"
        );
        assert!(
            events.events.lock().expect("events").is_empty(),
            "no events should be published on filesystem conflict"
        );
    }

    #[test]
    fn import_dataset_restores_orphaned_live_dir_when_forced_insert_fails() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let uid = Uuid::new_v4();
        let archive = create_import_archive(temp_dir.path(), uid, "rollback");
        let live_dir = paths.dataset_path_from_uid(uid);
        fs::create_dir_all(&live_dir).expect("live dir");
        fs::write(live_dir.join("orphaned.arrow"), b"OLD").expect("orphaned payload");

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_find_dataset_by_uid()
            .once()
            .withf(move |candidate| *candidate == uid)
            .return_once(move |_| Ok(None));
        repository
            .expect_insert_imported_dataset_record()
            .once()
            .return_once(move |_| {
                Err(CatalogError::NotFound {
                    id: "mock".to_string(),
                })
            });

        let service = DatasetCatalogService::new(Arc::new(repository), paths);
        let events = CollectEvents::default();

        let result = service.import_dataset(&archive, true, &events);

        assert!(
            result.is_err(),
            "forced import should fail when repository insert fails"
        );
        assert!(
            live_dir.join("orphaned.arrow").exists(),
            "orphaned live payload should be restored after rollback"
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

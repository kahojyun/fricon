use std::sync::Arc;

use tracing::{error, info, instrument, warn};

use crate::{
    dataset::{
        NormalizedTag,
        catalog::{CatalogError, DatasetCatalogRepository},
        events::{DatasetEvent, DatasetEventPublisher},
        model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
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
        self.ensure_not_deleted(id)?;
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
        self.ensure_not_deleted(id)?;
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
        self.ensure_not_deleted(id)?;
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
        storage::move_dataset(&dataset_path, &graveyard_path).inspect_err(|e| {
            error!(error = %e, dataset.id = id, "Dataset graveyard staging failed");
        })?;

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
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc};

    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;
    use crate::dataset::{
        catalog::MockDatasetCatalogRepository,
        model::{DatasetMetadata, DatasetStatus},
    };

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
}

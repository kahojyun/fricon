use std::sync::Arc;

use tracing::{error, instrument};

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

    #[instrument(skip(self), fields(dataset.id = id))]
    pub(crate) fn delete_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        self.repository.delete_dataset(id)?;
        let dataset_path = self.paths.dataset_path_from_uid(record.metadata.uid);
        storage::delete_dataset(&dataset_path).inspect_err(|e| {
            error!(error = %e, dataset.id = id, "Dataset deletion failed");
        })?;
        Ok(())
    }

    #[instrument(skip(self, events), fields(dataset.id = id))]
    pub(crate) fn trash_dataset<P: DatasetEventPublisher>(
        &self,
        id: i32,
        events: &P,
    ) -> Result<(), CatalogError> {
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
        self.repository.restore_dataset(id)?;
        let record = self.repository.get_dataset(DatasetId::Id(id))?;
        events.publish(DatasetEvent::Updated(record));
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) fn empty_trash(&self) -> Result<usize, CatalogError> {
        let records = self.repository.purge_trashed_datasets()?;
        let count = records.len();
        let mut failed_deletions = 0;
        for record in records {
            let dataset_path = self.paths.dataset_path_from_uid(record.metadata.uid);
            if let Err(error) = storage::delete_dataset(&dataset_path) {
                failed_deletions += 1;
                error!(error = %error, dataset.id = record.id, "Dataset purge failed");
            }
        }
        if failed_deletions > 0 {
            error!(
                failed.deletions = failed_deletions,
                "Dataset purge completed with file cleanup failures"
            );
        }
        Ok(count)
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
                tags: Vec::new(),
            },
        }
    }

    #[test]
    fn empty_trash_continues_after_file_cleanup_failure() {
        let temp_dir = tempdir().expect("temp dir should be created");
        let paths = WorkspacePaths::new(temp_dir.path());
        let bad_uid = Uuid::new_v4();
        let good_uid = Uuid::new_v4();
        let bad_path = paths.dataset_path_from_uid(bad_uid);
        let good_path = paths.dataset_path_from_uid(good_uid);

        fs::create_dir_all(
            bad_path
                .parent()
                .expect("dataset path should have a parent directory"),
        )
        .expect("bad dataset parent directory should be created");
        fs::write(&bad_path, b"not a directory")
            .expect("bad dataset path should be created as a file");
        fs::create_dir_all(&good_path).expect("good dataset directory should be created");

        let mut repository = MockDatasetCatalogRepository::new();
        repository
            .expect_purge_trashed_datasets()
            .once()
            .return_once(move || {
                Ok(vec![
                    dataset_record(1, bad_uid),
                    dataset_record(2, good_uid),
                ])
            });

        let service = DatasetCatalogService::new(Arc::new(repository), paths);

        let deleted_count = service.empty_trash().expect("empty trash should succeed");

        assert_eq!(deleted_count, 2);
        assert!(
            bad_path.exists(),
            "failed cleanup should leave the bad path behind"
        );
        assert!(
            !good_path.exists(),
            "cleanup should continue and remove later dataset directories"
        );
    }
}

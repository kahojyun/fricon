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
        for record in records {
            let dataset_path = self.paths.dataset_path_from_uid(record.metadata.uid);
            storage::delete_dataset(&dataset_path).inspect_err(|e| {
                error!(error = %e, dataset.id = record.id, "Dataset purge failed");
            })?;
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

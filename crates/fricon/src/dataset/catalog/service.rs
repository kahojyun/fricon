use std::sync::Arc;

use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::task::TaskTracker;
use tracing::{error, instrument};

use crate::{
    dataset::{
        NormalizedTag,
        catalog::{CatalogError, DatasetCatalogRepository},
        events::{AppEvent, dataset_updated_event},
        model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
        storage,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub struct DatasetCatalogService {
    repository: Arc<dyn DatasetCatalogRepository>,
    paths: WorkspacePaths,
    event_sender: broadcast::Sender<AppEvent>,
    tracker: TaskTracker,
}

impl DatasetCatalogService {
    #[must_use]
    pub(crate) fn new(
        repository: Arc<dyn DatasetCatalogRepository>,
        paths: WorkspacePaths,
        event_sender: broadcast::Sender<AppEvent>,
        tracker: TaskTracker,
    ) -> Self {
        Self {
            repository,
            paths,
            event_sender,
            tracker,
        }
    }

    fn spawn_blocking<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.tracker.spawn_blocking(f)
    }

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError> {
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.get_dataset(id))
            .await?
    }

    #[instrument(skip(self, query_options))]
    pub async fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError> {
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.list_datasets(query_options))
            .await?
    }

    #[instrument(skip(self))]
    pub async fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError> {
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.list_dataset_tags())
            .await?
    }

    #[instrument(skip(self, update_payload), fields(dataset.id = id))]
    pub async fn update_dataset(
        &self,
        id: i32,
        update_payload: DatasetUpdate,
    ) -> Result<(), CatalogError> {
        let repository = Arc::clone(&self.repository);
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            repository.update_dataset(id, update_payload)?;
            let record = repository.get_dataset(DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), CatalogError> {
        let tags = NormalizedTag::parse_many(tags);
        if tags.is_empty() {
            return Ok(());
        }
        let repository = Arc::clone(&self.repository);
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            repository.add_tags(id, &tags)?;
            let record = repository.get_dataset(DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), CatalogError> {
        let tags = NormalizedTag::parse_many(tags);
        if tags.is_empty() {
            return Ok(());
        }
        let repository = Arc::clone(&self.repository);
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            repository.remove_tags(id, &tags)?;
            let record = repository.get_dataset(DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self), fields(dataset.id = id))]
    pub async fn delete_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let repository = Arc::clone(&self.repository);
        let paths = self.paths.clone();
        self.spawn_blocking(move || {
            let record = repository.get_dataset(DatasetId::Id(id))?;
            repository.delete_dataset(id)?;
            let dataset_path = paths.dataset_path_from_uid(record.metadata.uid);
            storage::delete_dataset(&dataset_path).inspect_err(|e| {
                error!(error = %e, dataset.id = id, "Dataset deletion failed");
            })?;
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self, tag), fields(tag.name = %tag))]
    pub async fn delete_tag(&self, tag: String) -> Result<(), CatalogError> {
        let tag = NormalizedTag::parse(tag)?;
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.delete_tag(&tag))
            .await?
    }

    #[instrument(skip(self, old_name, new_name), fields(tag.old = %old_name, tag.new = %new_name))]
    pub async fn rename_tag(&self, old_name: String, new_name: String) -> Result<(), CatalogError> {
        let old_name = NormalizedTag::parse(old_name)?;
        let new_name = NormalizedTag::parse(new_name)?;
        if old_name == new_name {
            return Err(anyhow::anyhow!("old tag name and new tag name must differ").into());
        }
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.rename_tag(&old_name, &new_name))
            .await?
    }

    #[instrument(skip(self, source, target), fields(tag.source = %source, tag.target = %target))]
    pub async fn merge_tag(&self, source: String, target: String) -> Result<(), CatalogError> {
        let source = NormalizedTag::parse(source)?;
        let target = NormalizedTag::parse(target)?;
        if source == target {
            return Err(anyhow::anyhow!("source tag and target tag must differ").into());
        }
        let repository = Arc::clone(&self.repository);
        self.spawn_blocking(move || repository.merge_tag(&source, &target))
            .await?
    }
}

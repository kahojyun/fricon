use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::task::TaskTracker;
use tracing::{error, instrument};

use crate::{
    dataset::{
        catalog::{CatalogError, mutate, query},
        events::{AppEvent, dataset_updated_event},
        model::{DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate},
        sqlite::Pool,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub struct DatasetCatalogService {
    database: Pool,
    paths: WorkspacePaths,
    event_sender: broadcast::Sender<AppEvent>,
    tracker: TaskTracker,
}

impl DatasetCatalogService {
    #[must_use]
    pub(crate) fn new(
        database: Pool,
        paths: WorkspacePaths,
        event_sender: broadcast::Sender<AppEvent>,
        tracker: TaskTracker,
    ) -> Self {
        Self {
            database,
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
        let database = self.database.clone();
        self.spawn_blocking(move || query::do_get_dataset(&mut *database.get()?, id))
            .await?
    }

    #[instrument(skip(self, query_options))]
    pub async fn list_datasets(
        &self,
        query_options: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError> {
        let database = self.database.clone();
        self.spawn_blocking(move || query::do_list_datasets(&mut *database.get()?, &query_options))
            .await?
    }

    #[instrument(skip(self))]
    pub async fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError> {
        let database = self.database.clone();
        self.spawn_blocking(move || query::do_list_dataset_tags(&mut *database.get()?))
            .await?
    }

    #[instrument(skip(self, update_payload), fields(dataset.id = id))]
    pub async fn update_dataset(
        &self,
        id: i32,
        update_payload: DatasetUpdate,
    ) -> Result<(), CatalogError> {
        let database = self.database.clone();
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            let mut conn = database.get()?;
            mutate::do_update_dataset(&mut conn, id, update_payload)?;
            let record = query::do_get_dataset(&mut conn, DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), CatalogError> {
        let database = self.database.clone();
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            let mut conn = database.get()?;
            mutate::do_add_tags(&mut conn, id, &tags)?;
            let record = query::do_get_dataset(&mut conn, DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), CatalogError> {
        let database = self.database.clone();
        let event_sender = self.event_sender.clone();
        self.spawn_blocking(move || {
            let mut conn = database.get()?;
            mutate::do_remove_tags(&mut conn, id, &tags)?;
            let record = query::do_get_dataset(&mut conn, DatasetId::Id(id))?;
            let _ = event_sender.send(dataset_updated_event(record));
            Ok(())
        })
        .await?
    }

    #[instrument(skip(self), fields(dataset.id = id))]
    pub async fn delete_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let database = self.database.clone();
        let paths = self.paths.clone();
        self.spawn_blocking(move || {
            mutate::do_delete_dataset(&database, &paths, id).inspect_err(|e| {
                error!(error = %e, dataset.id = id, "Dataset deletion failed");
            })
        })
        .await?
    }
}

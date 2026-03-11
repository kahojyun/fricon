use tracing::instrument;

use crate::{
    dataset_catalog::{
        DatasetCatalogError, DatasetId, DatasetListQuery, DatasetRecord, DatasetUpdate,
        events::emit_dataset_updated, query, update,
    },
    runtime::app::AppHandle,
};

#[derive(Clone)]
pub struct DatasetCatalogService {
    app: AppHandle,
}

impl DatasetCatalogService {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| query::get_dataset(&mut *state.database.get()?, id))?
            .await?
    }

    #[instrument(skip(self, query))]
    pub async fn list_datasets(
        &self,
        query: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| query::list_datasets(&mut *state.database.get()?, &query))?
            .await?
    }

    #[instrument(skip(self))]
    pub async fn list_dataset_tags(&self) -> Result<Vec<String>, DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| query::list_dataset_tags(&mut *state.database.get()?))?
            .await?
    }

    #[instrument(skip(self, update_payload), fields(dataset.id = id))]
    pub async fn update_dataset(
        &self,
        id: i32,
        update_payload: DatasetUpdate,
    ) -> Result<(), DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                update::update_dataset(&mut conn, id, update_payload)?;
                let record = update::reload_dataset(&mut conn, id)?;
                emit_dataset_updated(&state, record);
                Ok(())
            })?
            .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                update::add_tags(&mut conn, id, &tags)?;
                let record = update::reload_dataset(&mut conn, id)?;
                emit_dataset_updated(&state, record);
                Ok(())
            })?
            .await?
    }

    #[instrument(skip(self, tags), fields(dataset.id = id, tags.count = tags.len()))]
    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                update::remove_tags(&mut conn, id, &tags)?;
                let record = update::reload_dataset(&mut conn, id)?;
                emit_dataset_updated(&state, record);
                Ok(())
            })?
            .await?
    }

    #[instrument(skip(self), fields(dataset.id = id))]
    pub async fn delete_dataset(&self, id: i32) -> Result<(), DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| update::delete_dataset(&state.database, &state.root, id))?
            .await?
    }
}

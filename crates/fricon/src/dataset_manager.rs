//! Dataset Manager - Central hub for all dataset operations
//!
//! The `DatasetManager` centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

use std::{
    error::Error as StdError,
    io::{Error as IoError, ErrorKind},
    path::Path,
};

use arrow_array::RecordBatch;
use chrono::{DateTime, Utc};
use derive_more::From;
use diesel::result::Error as DieselError;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;
use tracing::error;
use uuid::Uuid;

use crate::{
    app::{AppError, AppHandle},
    database::{self, DatabaseError, DatasetStatus},
    dataset, dataset_fs, dataset_tasks,
};

#[derive(Debug, thiserror::Error)]
pub enum DatasetManagerError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Schema validation failed: {message}")]
    SchemaError { message: String },
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Dataset(#[from] dataset::Error),
    #[error(transparent)]
    DatasetFs(#[from] dataset_fs::Error),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DieselError> for DatasetManagerError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound {
                id: "unknown".to_string(),
            },
            other => Self::Database(other.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: DatasetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
}

#[derive(Debug, Clone, Copy, From)]
pub enum DatasetId {
    Id(i32),
    Uid(Uuid),
}

#[derive(Clone)]
pub struct DatasetManager {
    app: AppHandle,
}

impl DatasetManager {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn create_dataset<S, E>(
        &self,
        request: CreateDatasetRequest,
        stream: S,
    ) -> Result<DatasetRecord, DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: StdError + Send + Sync + 'static,
    {
        let stream: Box<
            dyn Stream<Item = Result<RecordBatch, Box<dyn StdError + Send + Sync>>> + Send + Unpin,
        > = Box::new(stream.map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>));

        let join_handle = self.app.spawn(move |state| async move {
            let result = dataset_tasks::do_create_dataset(
                &state.database,
                &state.root,
                &state.event_sender,
                &state.write_sessions,
                &state.tracker,
                request,
                stream,
            )
            .await;
            if let Err(e) = &result {
                error!("Dataset creation failed: {}", e);
            }
            result
        })?;

        join_handle.await?
    }

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_get_dataset(&state.database, id).await
        })?;

        join_handle.await?
    }

    pub async fn list_datasets(&self) -> Result<Vec<DatasetRecord>, DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_list_datasets(&state.database).await
        })?;

        join_handle.await?
    }

    pub async fn update_dataset(
        &self,
        id: i32,
        update: DatasetUpdate,
    ) -> Result<(), DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_update_dataset(&state.database, id, update).await
        })?;

        join_handle.await?
    }

    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_add_tags(&state.database, id, tags).await
        })?;

        join_handle.await?
    }

    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_remove_tags(&state.database, id, tags).await
        })?;

        join_handle.await?
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            let result = dataset_tasks::do_delete_dataset(&state.database, &state.root, id).await;
            if let Err(e) = &result {
                error!("Dataset deletion failed: {}", e);
            }
            result
        })?;

        join_handle.await?
    }

    /// Return a unified dataset reader (Completed or Live/Writing).
    pub async fn get_dataset_reader(
        &self,
        id: DatasetId,
    ) -> Result<DatasetReader, DatasetManagerError> {
        let join_handle = self.app.spawn(move |state| async move {
            dataset_tasks::do_get_dataset_reader(
                &state.database,
                &state.root,
                &state.write_sessions,
                id,
            )
            .await
        })?;

        join_handle.await?
    }
}

impl DatasetRecord {
    #[must_use]
    pub fn from_database_models(dataset: database::Dataset, tags: Vec<database::Tag>) -> Self {
        let metadata = DatasetMetadata {
            uid: dataset.uid.0,
            name: dataset.name,
            description: dataset.description,
            favorite: dataset.favorite,
            status: dataset.status,
            created_at: dataset.created_at.and_utc(),
            tags: tags.into_iter().map(|tag| tag.name).collect(),
        };

        Self {
            id: dataset.id,
            metadata,
        }
    }
}

//! Dataset Manager - Central hub for all dataset operations
//!
//! The `DatasetManager` centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

mod in_progress;
mod tasks;
mod write_registry;
mod write_session;

use arrow_array::{RecordBatch, RecordBatchReader};
use chrono::{DateTime, Utc};
use derive_more::From;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;
use tracing::error;
use uuid::Uuid;

pub use self::write_registry::WriteSessionRegistry;
use crate::{
    app::{AppError, AppHandle},
    database::{self, DatabaseError, DatasetStatus},
    dataset, dataset_fs,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Schema validation failed: {message}")]
    SchemaError { message: String },
    #[error("Dataset write stream error: {message}")]
    BatchStreamError { message: String },
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

impl From<DieselError> for Error {
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

    pub async fn create_dataset<F, I>(
        &self,
        request: CreateDatasetRequest,
        reader: F,
    ) -> Result<DatasetRecord, Error>
    where
        F: FnOnce() -> Result<I, Error> + Send + 'static,
        I: RecordBatchReader,
    {
        self.app
            .spawn_blocking(move |state| {
                reader()
                    .and_then(|batches| {
                        tasks::do_create_dataset(
                            &state.database,
                            &state.root,
                            &state.event_sender,
                            &state.write_sessions,
                            request,
                            batches,
                        )
                    })
                    .inspect_err(|e| {
                        error!("Dataset creation failed: {e}");
                    })
            })?
            .await?
    }

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, Error> {
        self.app
            .spawn_blocking(move |state| tasks::do_get_dataset(&mut *state.database.get()?, id))?
            .await?
    }

    pub async fn list_datasets(&self) -> Result<Vec<DatasetRecord>, Error> {
        self.app
            .spawn_blocking(move |state| tasks::do_list_datasets(&mut *state.database.get()?))?
            .await?
    }

    pub async fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_update_dataset(&mut *state.database.get()?, id, update)
            })?
            .await?
    }

    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_add_tags(&mut *state.database.get()?, id, &tags)
            })?
            .await?
    }

    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_remove_tags(&mut *state.database.get()?, id, &tags)
            })?
            .await?
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_delete_dataset(&state.database, &state.root, id).inspect_err(|e| {
                    error!("Dataset deletion failed: {e}");
                })
            })?
            .await?
    }

    /// Return a unified dataset reader (Completed or Live/Writing).
    pub async fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_get_dataset_reader(
                    &state.database,
                    &state.root,
                    &state.write_sessions,
                    id,
                )
            })?
            .await?
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

// TODO: implement
pub struct DatasetReader;

impl DatasetReader {
    pub fn batches(&self) -> Result<Vec<RecordBatch>, Error> {
        todo!()
    }
}

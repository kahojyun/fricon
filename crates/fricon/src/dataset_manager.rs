//! Dataset Manager - Central hub for all dataset operations
//!
//! The `DatasetManager` centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

// live / reader / write_session moved to crate root modules (see lib.rs re-exports)

use std::{fs, path::Path};

use arrow::array::RecordBatch;
use arrow::error::ArrowError;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    app::{AppEvent, AppHandle},
    database::{self, DatabaseError, DatasetStatus, NewDataset, PoolExt, SimpleUuid, schema},
};

#[derive(Debug, thiserror::Error)]
pub enum DatasetManagerError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },

    #[error("Schema validation failed: {message}")]
    SchemaError { message: String },

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] ArrowError),

    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl DatasetManagerError {
    pub(crate) fn io_invalid_data(message: impl Into<String>) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message.into(),
        ))
    }

    fn stream_error(error: impl std::error::Error) -> Self {
        Self::io_invalid_data(format!("Stream error: {error}"))
    }

    fn empty_stream() -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Stream is empty",
        ))
    }

    fn path_already_exists(path: &std::path::Path) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Dataset path already exists: {}", path.display()),
        ))
    }
}

impl From<diesel::result::Error> for DatasetManagerError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => Self::NotFound {
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
    pub uuid: Uuid,
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

#[derive(Debug, Clone, Copy)]
pub enum DatasetId {
    Id(i32),
    Uuid(Uuid),
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

    #[must_use]
    pub fn app(&self) -> &AppHandle {
        &self.app
    }

    pub async fn create_dataset<S, E>(
        &self,
        request: CreateDatasetRequest,
        stream: S,
    ) -> Result<DatasetRecord, DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let app = self.app.clone();
        let tracker = app.tracker().clone();

        let dataset_record = tracker
            .spawn(async move {
                let result = app
                    .dataset_manager()
                    .create_dataset_tracked(request, stream)
                    .await;
                if let Err(e) = &result {
                    error!("Dataset creation failed: {}", e);
                }
                result
            })
            .await??;

        Ok(dataset_record)
    }

    async fn create_dataset_tracked<S, E>(
        &self,
        request: CreateDatasetRequest,
        stream: S,
    ) -> Result<DatasetRecord, DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let uuid = Uuid::new_v4();
        let dataset_path = self.app.root().paths().dataset_path_from_uuid(uuid);

        if dataset_path.exists() {
            warn!("Dataset path already exists: {}", dataset_path.display());
            return Err(DatasetManagerError::path_already_exists(&dataset_path));
        }

        info!(
            "Creating new dataset '{}' with UUID: {}",
            request.name, uuid
        );

        let (dataset, tags) = self.create_db_record(&request, uuid).await?;

        fs::create_dir_all(&dataset_path)?;

        let event = AppEvent::DatasetCreated {
            id: dataset.id,
            uuid: uuid.to_string(),
            name: request.name.clone(),
            description: request.description.clone(),
            tags: request.tags.clone(),
        };
        self.app.send_event(event);

        info!(
            "Created dataset with UUID: {} at path: {:?}",
            uuid, dataset_path
        );

        let dataset_record = DatasetRecord::from_database_models(dataset, tags);

        let result = self
            .perform_write_async(dataset_record.id, &dataset_path, stream)
            .await;
        match result {
            Ok(()) => {
                self.update_status(dataset_record.id, DatasetStatus::Completed)
                    .await?;

                let updated_record = self.get_dataset(DatasetId::Id(dataset_record.id)).await?;
                Ok(updated_record)
            }
            Err(e) => {
                let _ = self
                    .update_status(dataset_record.id, DatasetStatus::Aborted)
                    .await;
                Err(e)
            }
        }
    }

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetManagerError> {
        let result = self
            .db(move |conn| {
                let dataset = match id {
                    DatasetId::Id(dataset_id) => database::Dataset::find_by_id(conn, dataset_id)?,
                    DatasetId::Uuid(uuid) => database::Dataset::find_by_uuid(conn, uuid)?,
                };

                let Some(dataset) = dataset else {
                    let id_str = match id {
                        DatasetId::Id(i) => i.to_string(),
                        DatasetId::Uuid(u) => u.to_string(),
                    };
                    return Err(DatasetManagerError::NotFound { id: id_str });
                };

                let tags = dataset.load_tags(conn)?;
                Ok((dataset, tags))
            })
            .await?;

        Ok(DatasetRecord::from_database_models(result.0, result.1))
    }

    pub async fn list_datasets(&self) -> Result<Vec<DatasetRecord>, DatasetManagerError> {
        let datasets_with_tags = self
            .db(|conn| {
                let all_datasets = database::Dataset::list_all_ordered(conn)?;

                let dataset_tags = database::DatasetTag::belonging_to(&all_datasets)
                    .inner_join(schema::tags::table)
                    .select((
                        database::DatasetTag::as_select(),
                        database::Tag::as_select(),
                    ))
                    .load::<(database::DatasetTag, database::Tag)>(conn)?;

                let datasets_with_tags: Vec<(database::Dataset, Vec<database::Tag>)> = dataset_tags
                    .grouped_by(&all_datasets)
                    .into_iter()
                    .zip(all_datasets)
                    .map(|(dataset_tags, dataset)| {
                        (
                            dataset,
                            dataset_tags.into_iter().map(|(_, tag)| tag).collect(),
                        )
                    })
                    .collect();

                Ok(datasets_with_tags)
            })
            .await?;

        Ok(datasets_with_tags
            .into_iter()
            .map(|(dataset, tags)| DatasetRecord::from_database_models(dataset, tags))
            .collect())
    }

    pub async fn update_dataset(
        &self,
        id: i32,
        update: DatasetUpdate,
    ) -> Result<(), DatasetManagerError> {
        let db_update = database::DatasetUpdate {
            name: update.name,
            description: update.description,
            favorite: update.favorite,
            status: None,
        };

        self.db(move |conn| {
            database::Dataset::update_metadata(conn, id, &db_update)?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        self.db(move |conn| {
            conn.immediate_transaction(|conn| {
                let created_tags = database::Tag::find_or_create_batch(conn, &tags)?;
                let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

                database::DatasetTag::create_associations(conn, id, &tag_ids)?;
                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        self.db(move |conn| {
            conn.immediate_transaction(|conn| {
                let tag_ids_to_delete = schema::tags::table
                    .filter(schema::tags::name.eq_any(&tags))
                    .select(schema::tags::id)
                    .load::<i32>(conn)?;

                database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), DatasetManagerError> {
        let app = self.app.clone();
        let tracker = app.tracker().clone();

        tracker
            .spawn(async move {
                let result = app.dataset_manager().delete_dataset_tracked(id).await;
                if let Err(e) = &result {
                    error!("Dataset deletion failed: {}", e);
                }
                result
            })
            .await??;

        Ok(())
    }

    async fn delete_dataset_tracked(&self, id: i32) -> Result<(), DatasetManagerError> {
        let record = self.get_dataset(DatasetId::Id(id)).await?;
        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid);

        self.db(move |conn| {
            database::Dataset::delete_from_db(conn, id)?;
            Ok(())
        })
        .await?;

        if dataset_path.exists() {
            fs::remove_dir_all(&dataset_path)?;
        }

        Ok(())
    }

    async fn create_db_record(
        &self,
        request: &CreateDatasetRequest,
        uuid: Uuid,
    ) -> Result<(database::Dataset, Vec<database::Tag>), DatasetManagerError> {
        let request = request.clone();
        let res = self
            .db(move |conn| {
                conn.immediate_transaction(|conn| {
                    let new_dataset = NewDataset {
                        uuid: SimpleUuid(uuid),
                        name: &request.name,
                        description: &request.description,
                        status: DatasetStatus::Writing,
                    };

                    let dataset = diesel::insert_into(schema::datasets::table)
                        .values(new_dataset)
                        .returning(database::Dataset::as_returning())
                        .get_result(conn)?;

                    let tags = if request.tags.is_empty() {
                        vec![]
                    } else {
                        let created_tags =
                            database::Tag::find_or_create_batch(conn, &request.tags)?;
                        let tag_ids: Vec<i32> = created_tags.iter().map(|tag| tag.id).collect();
                        database::DatasetTag::create_associations(conn, dataset.id, &tag_ids)?;
                        created_tags
                    };

                    Ok((dataset, tags))
                })
            })
            .await?;

        Ok(res)
    }

    async fn update_status(
        &self,
        id: i32,
        status: DatasetStatus,
    ) -> Result<(), DatasetManagerError> {
        self.db(move |conn| {
            database::Dataset::update_status(conn, id, status)?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn db<F, T>(&self, f: F) -> Result<T, DatasetManagerError>
    where
        F: FnOnce(&mut diesel::SqliteConnection) -> Result<T, DatasetManagerError> + Send + 'static,
        T: Send + 'static,
    {
        let res = self.app.database().interact(f).await??;
        Ok(res)
    }

    async fn perform_write_async<S, E>(
        &self,
        dataset_id: i32,
        path: &Path,
        mut stream: S,
    ) -> Result<(), DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let first_batch = match stream.next().await {
            Some(Ok(batch)) => batch,
            Some(Err(e)) => return Err(DatasetManagerError::stream_error(e)),
            None => return Err(DatasetManagerError::empty_stream()),
        };

        let session_guard = self.app.write_sessions().create(
            dataset_id,
            self.app.tracker(),
            path,
            first_batch.schema(),
        );

        // Accumulate potential error but always drive finish() to guarantee ordering:
        // (flush + Closed) -> remove registry -> caller updates status.
        let mut write_error: Option<DatasetManagerError> = None;

        if let Err(e) = session_guard
            .write(first_batch)
            .await
            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))
        {
            write_error = Some(e);
        } else {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(batch) => {
                        if let Err(e) = session_guard
                            .write(batch)
                            .await
                            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))
                        {
                            write_error = Some(e);
                            break;
                        }
                    }
                    Err(e) => {
                        write_error = Some(DatasetManagerError::stream_error(e));
                        break;
                    }
                }
            }
        }

        let finish_result = session_guard.finish().await;
        if let Err(e) = finish_result {
            let finish_err = DatasetManagerError::io_invalid_data(e.to_string());
            if write_error.is_none() {
                write_error = Some(finish_err);
            }
        }

        if let Some(e) = write_error {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Return a unified dataset reader (Completed or Live/Writing).
    pub async fn get_dataset_reader(
        &self,
        id: DatasetId,
    ) -> Result<crate::reader::DatasetReader, DatasetManagerError> {
        let record = self.get_dataset(id).await?;
        match record.metadata.status {
            DatasetStatus::Completed | DatasetStatus::Aborted => {
                // Aborted datasets may still have partially written chunk files (valid up to last flush).
                let dataset_path = self
                    .app
                    .root()
                    .paths()
                    .dataset_path_from_uuid(record.metadata.uuid);
                let completed = crate::reader::CompletedDataset::open(&dataset_path)?;
                Ok(crate::reader::DatasetReader::Completed(completed))
            }
            DatasetStatus::Writing => {
                if let Some(session) = self.app.write_sessions().get(record.id) {
                    return Ok(crate::reader::DatasetReader::Live(session.live().clone()));
                }
                // Fallback: if writer already dropped but directory exists, expose as Completed view.
                let dataset_path = self
                    .app
                    .root()
                    .paths()
                    .dataset_path_from_uuid(record.metadata.uuid);
                if dataset_path.exists() {
                    let completed = crate::reader::CompletedDataset::open(&dataset_path)?;
                    return Ok(crate::reader::DatasetReader::Completed(completed));
                }
                Err(DatasetManagerError::io_invalid_data(
                    "Dataset in Writing state has no active session and no file yet",
                ))
            }
        }
    }
}

impl DatasetRecord {
    #[must_use]
    pub fn from_database_models(dataset: database::Dataset, tags: Vec<database::Tag>) -> Self {
        let metadata = DatasetMetadata {
            uuid: dataset.uuid.0,
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

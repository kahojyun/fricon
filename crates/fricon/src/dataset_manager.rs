//! Dataset Manager - Central hub for all dataset operations
//!
//! The `DatasetManager` centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

mod batch_writer;

use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use anyhow::{Context, Result, bail};
use arrow::array::RecordBatch;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use self::batch_writer::BatchWriter;
use crate::{
    app::{AppEvent, AppHandle},
    database::{self, DatasetStatus, JsonValue, NewDataset, PoolExt, SimpleUuid, schema},
};

pub const DATASET_NAME: &str = "dataset.arrow";
pub const METADATA_NAME: &str = "metadata.json";

/// Errors that can occur during dataset manager operations
#[derive(Debug, thiserror::Error)]
pub enum DatasetManagerError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },

    #[error("Invalid write token")]
    InvalidToken,

    #[error("Dataset is not in writable state: {status:?}")]
    NotWritable { status: DatasetStatus },

    #[error("Schema validation failed: {message}")]
    SchemaError { message: String },

    #[error("Database error: {source}")]
    Database {
        #[from]
        source: anyhow::Error,
    },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

impl DatasetManagerError {
    /// Create an IO error from a string message with `InvalidData` kind
    fn io_invalid_data(message: impl Into<String>) -> Self {
        Self::Io {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()),
        }
    }

    /// Create an IO error for stream errors
    fn stream_error(error: impl std::error::Error) -> Self {
        Self::io_invalid_data(format!("Stream error: {error}"))
    }

    /// Create an IO error for empty stream
    fn empty_stream() -> Self {
        Self::Io {
            source: std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Stream is empty"),
        }
    }

    /// Create an IO error for already existing path
    fn path_already_exists(path: &std::path::Path) -> Self {
        Self::Io {
            source: std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Dataset path already exists: {}", path.display()),
            ),
        }
    }
}

// Convert diesel NotFound errors to DatasetManagerError::NotFound
impl From<diesel::result::Error> for DatasetManagerError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => Self::NotFound {
                id: "unknown".to_string(),
            },
            other => Self::Database {
                source: other.into(),
            },
        }
    }
}

/// Pure data structure representing a dataset record
#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: DatasetMetadata,
}

/// Pure data structure for dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub index_columns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Request structure for creating a new dataset
#[derive(Debug, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub index_columns: Vec<String>,
}

/// Update structure for modifying dataset information
#[derive(Debug, Clone, Default)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
}

/// Identifier for dataset lookup operations
#[derive(Debug, Clone, Copy)]
pub enum DatasetId {
    Id(i32),
    Uuid(Uuid),
}

/// Errors that can occur during the dataset write transaction
#[derive(Debug, thiserror::Error)]
enum WriteDatasetTxError {
    #[error("Dataset not found")]
    NotFound,
    #[error("Dataset is not in writable state: {0:?}")]
    NotWritable(DatasetStatus),
}

/// Central manager for all dataset operations
#[derive(Clone)]
pub struct DatasetManager {
    app: AppHandle,
}

impl DatasetManager {
    /// Create a new `DatasetManager` instance
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Get access to the app handle
    pub fn app(&self) -> &AppHandle {
        &self.app
    }

    /// Create a new dataset and return a write token for data upload
    pub async fn create_dataset(
        &self,
        request: CreateDatasetRequest,
    ) -> Result<Uuid, DatasetManagerError> {
        let uuid = Uuid::new_v4();
        let dataset_path = self.app.root().paths().dataset_path_from_uuid(uuid);

        // Ensure path doesn't already exist
        if dataset_path.exists() {
            return Err(DatasetManagerError::path_already_exists(&dataset_path));
        }

        // Create database record
        let dataset_id = self.create_db_record(&request, uuid).await?;

        // Create filesystem directory
        fs::create_dir_all(&dataset_path).with_context(|| {
            format!(
                "Failed to create dataset directory at {}",
                dataset_path.display()
            )
        })?;

        // Send dataset created event to notify UI
        let event = AppEvent::DatasetCreated {
            id: dataset_id,
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
        Ok(uuid)
    }

    /// Write data to a dataset using the provided write token
    pub async fn write_dataset<S, E>(
        &self,
        uuid: Uuid,
        stream: S,
    ) -> Result<DatasetRecord, DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Atomically check status and update to Writing to prevent race conditions
        let (dataset, tags) = self
            .app
            .database()
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    // Get dataset by UUID within transaction
                    let dataset = database::Dataset::find_by_uuid(conn, uuid)?
                        .ok_or_else(|| anyhow::Error::new(WriteDatasetTxError::NotFound))?;

                    // Check if dataset is in pending status
                    if dataset.status != DatasetStatus::Pending {
                        return Err(anyhow::Error::new(WriteDatasetTxError::NotWritable(
                            dataset.status,
                        )));
                    }

                    // Update status to Writing atomically
                    database::Dataset::update_status(conn, dataset.id, DatasetStatus::Writing)?;

                    // Load tags for the complete record
                    let tags = dataset.load_tags(conn)?;
                    Ok((dataset, tags))
                })
            })
            .await
            .map_err(|e| {
                if let Some(tx_err) = e.downcast_ref::<WriteDatasetTxError>() {
                    match tx_err {
                        WriteDatasetTxError::NotFound => DatasetManagerError::InvalidToken,
                        WriteDatasetTxError::NotWritable(status) => {
                            DatasetManagerError::NotWritable { status: *status }
                        }
                    }
                } else {
                    e.into()
                }
            })?;

        let dataset_record = DatasetRecord::from_database_models(dataset, tags);
        let dataset_path = self.app.root().paths().dataset_path_from_uuid(uuid);

        // Perform the actual write operation
        let result = self
            .perform_write_async(dataset_record.id, &dataset_path, stream)
            .await;

        match result {
            Ok(()) => {
                // Update status to Completed and save metadata
                self.update_status(dataset_record.id, DatasetStatus::Completed)
                    .await?;

                // Get updated dataset record and save metadata file
                let updated_record = self.get_dataset(DatasetId::Id(dataset_record.id)).await?;
                updated_record
                    .metadata
                    .save(&dataset_path.join(METADATA_NAME))?;

                // Return the completed dataset record
                Ok(updated_record)
            }
            Err(e) => {
                // Update status to Aborted on failure
                let _ = self
                    .update_status(dataset_record.id, DatasetStatus::Aborted)
                    .await;
                Err(e)
            }
        }
    }

    /// Get a dataset by ID or UUID
    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetManagerError> {
        let result = self
            .app
            .database()
            .interact(move |conn| {
                let dataset = match id {
                    DatasetId::Id(dataset_id) => database::Dataset::find_by_id(conn, dataset_id)?,
                    DatasetId::Uuid(uuid) => database::Dataset::find_by_uuid(conn, uuid)?,
                };

                let Some(dataset) = dataset else {
                    return Err(diesel::result::Error::NotFound.into());
                };

                let tags = dataset.load_tags(conn)?;
                Ok((dataset, tags))
            })
            .await;

        match result {
            Ok((dataset, tags)) => Ok(DatasetRecord::from_database_models(dataset, tags)),
            Err(e) => {
                if let Some(diesel::result::Error::NotFound) = e.downcast_ref() {
                    Err(DatasetManagerError::NotFound {
                        id: format!("{id:?}"),
                    })
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// List all datasets with optional filtering
    pub async fn list_datasets(&self) -> Result<Vec<DatasetRecord>, DatasetManagerError> {
        let datasets_with_tags = self
            .app
            .database()
            .interact(|conn| {
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

    /// Update dataset information
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

        self.app
            .database()
            .interact(move |conn| {
                database::Dataset::update_metadata(conn, id, &db_update)
                    .context("Failed to update dataset in database")
            })
            .await?;

        // Update metadata file
        let record = self.get_dataset(DatasetId::Id(id)).await?;
        let metadata_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid)
            .join(METADATA_NAME);
        record.metadata.save(&metadata_path)?;

        Ok(())
    }

    /// Add tags to a dataset
    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        self.app
            .database()
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    // Create or find tags and get their IDs
                    let created_tags = database::Tag::find_or_create_batch(conn, &tags)?;
                    let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

                    // Create associations
                    database::DatasetTag::create_associations(conn, id, &tag_ids)?;
                    Ok(())
                })
            })
            .await?;

        // Update metadata file
        let record = self.get_dataset(DatasetId::Id(id)).await?;
        let metadata_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid)
            .join(METADATA_NAME);
        record.metadata.save(&metadata_path)?;

        Ok(())
    }

    /// Remove tags from a dataset
    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), DatasetManagerError> {
        self.app
            .database()
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    // Get the IDs of the tags to be deleted
                    let tag_ids_to_delete = schema::tags::table
                        .filter(schema::tags::name.eq_any(&tags))
                        .select(schema::tags::id)
                        .load::<i32>(conn)?;

                    // Remove associations
                    database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
                    Ok(())
                })
            })
            .await?;

        // Update metadata file
        let record = self.get_dataset(DatasetId::Id(id)).await?;
        let metadata_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid)
            .join(METADATA_NAME);
        record.metadata.save(&metadata_path)?;

        Ok(())
    }

    /// Delete a dataset
    pub async fn delete_dataset(&self, id: i32) -> Result<(), DatasetManagerError> {
        // Get dataset info before deletion
        let record = self.get_dataset(DatasetId::Id(id)).await?;
        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid);

        // Delete from database
        self.app
            .database()
            .interact(move |conn| {
                database::Dataset::delete_from_db(conn, id)
                    .context("Failed to delete dataset from database")
            })
            .await?;

        // Delete filesystem directory
        if dataset_path.exists() {
            fs::remove_dir_all(&dataset_path).with_context(|| {
                format!(
                    "Failed to delete dataset directory: {}",
                    dataset_path.display()
                )
            })?;
        }

        Ok(())
    }

    // Helper methods

    /// Create database record for a new dataset
    async fn create_db_record(
        &self,
        request: &CreateDatasetRequest,
        uuid: Uuid,
    ) -> Result<i32, DatasetManagerError> {
        let request = request.clone();
        let dataset_id = self
            .app
            .database()
            .interact(move |conn| {
                conn.immediate_transaction(|conn| {
                    let new_dataset = NewDataset {
                        uuid: SimpleUuid(uuid),
                        name: &request.name,
                        description: &request.description,
                        status: DatasetStatus::Pending,
                        index_columns: JsonValue(&request.index_columns),
                    };

                    let dataset = diesel::insert_into(schema::datasets::table)
                        .values(new_dataset)
                        .returning(database::Dataset::as_returning())
                        .get_result(conn)?;

                    // Handle tags creation and association
                    if !request.tags.is_empty() {
                        let created_tags =
                            database::Tag::find_or_create_batch(conn, &request.tags)?;
                        let tag_ids: Vec<i32> =
                            created_tags.into_iter().map(|tag| tag.id).collect();
                        database::DatasetTag::create_associations(conn, dataset.id, &tag_ids)?;
                    }

                    Ok(dataset.id)
                })
            })
            .await?;

        Ok(dataset_id)
    }

    /// Update dataset status in database
    async fn update_status(
        &self,
        id: i32,
        status: DatasetStatus,
    ) -> Result<(), DatasetManagerError> {
        self.app
            .database()
            .interact(move |conn| {
                database::Dataset::update_status(conn, id, status)
                    .context("Failed to update dataset status")
            })
            .await?;

        Ok(())
    }

    async fn perform_write_async<S, E>(
        &self,
        _dataset_id: i32,
        path: &Path,
        mut stream: S,
    ) -> Result<(), DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let dataset_path = path.join(DATASET_NAME);

        // Create the Arrow file
        let file = File::create_new(&dataset_path).with_context(|| {
            format!(
                "Failed to create dataset file at {}",
                dataset_path.display()
            )
        })?;

        // Spawn blocking task to write batches using BatchWriter
        let write_result = self
            .app
            .tracker()
            .spawn_blocking(move || -> Result<(), DatasetManagerError> {
                let rt_handle = tokio::runtime::Handle::current();

                // Get the first batch to initialize the writer
                let mut batch = match rt_handle.block_on(stream.next()) {
                    Some(Ok(batch)) => batch,
                    Some(Err(e)) => return Err(DatasetManagerError::stream_error(e)),
                    None => return Err(DatasetManagerError::empty_stream()),
                };

                let buf_writer = BufWriter::new(file);
                let mut batch_writer = BatchWriter::new(buf_writer, &batch.schema())
                    .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;

                loop {
                    batch_writer
                        .write(batch)
                        .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;
                    batch = match rt_handle.block_on(stream.next()) {
                        Some(Ok(batch)) => batch,
                        Some(Err(e)) => return Err(DatasetManagerError::stream_error(e)),
                        None => break, // End of stream
                    };
                }

                batch_writer
                    .finish()
                    .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;

                Ok(())
            })
            .await;

        match write_result {
            Ok(result) => result,
            Err(e) => Err(std::io::Error::other(format!("Write task failed: {e}")).into()),
        }
    }
}

impl DatasetRecord {
    /// Create `DatasetRecord` from database models
    #[must_use]
    pub fn from_database_models(dataset: database::Dataset, tags: Vec<database::Tag>) -> Self {
        let metadata = DatasetMetadata {
            uuid: dataset.uuid.0,
            name: dataset.name,
            description: dataset.description,
            favorite: dataset.favorite,
            status: dataset.status,
            index_columns: dataset.index_columns.0,
            created_at: dataset.created_at.and_utc(),
            tags: tags.into_iter().map(|tag| tag.name).collect(),
        };

        Self {
            id: dataset.id,
            metadata,
        }
    }
}

// Conversion to protobuf types
impl From<DatasetRecord> for crate::proto::Dataset {
    fn from(record: DatasetRecord) -> Self {
        Self {
            id: record.id,
            metadata: Some(record.metadata.into()),
        }
    }
}

impl TryFrom<crate::proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(dataset: crate::proto::Dataset) -> Result<Self, Self::Error> {
        use anyhow::Context;
        Ok(Self {
            id: dataset.id,
            metadata: dataset
                .metadata
                .context("metadata field is required")?
                .try_into()?,
        })
    }
}

impl From<DatasetMetadata> for crate::proto::DatasetMetadata {
    fn from(metadata: DatasetMetadata) -> Self {
        use prost_types::Timestamp;
        let created_at = Timestamp {
            seconds: metadata.created_at.timestamp(),
            #[expect(clippy::cast_possible_wrap, reason = "Nanos are always less than 2e9.")]
            nanos: metadata.created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            uuid: metadata.uuid.simple().to_string(),
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            index_columns: metadata.index_columns,
            created_at: Some(created_at),
            tags: metadata.tags,
            status: crate::proto::DatasetStatus::from(metadata.status) as i32,
        }
    }
}

impl TryFrom<crate::proto::DatasetMetadata> for DatasetMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: crate::proto::DatasetMetadata) -> Result<Self, Self::Error> {
        use anyhow::{Context, bail};
        use chrono::DateTime;

        let uuid = metadata.uuid.parse()?;
        let created_at = metadata.created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(clippy::cast_sign_loss)]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
        let proto_status = crate::proto::DatasetStatus::try_from(metadata.status)
            .context("Invalid dataset status")?;
        let status = DatasetStatus::try_from(proto_status)?;

        Ok(Self {
            uuid,
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            status,
            index_columns: metadata.index_columns,
            created_at,
            tags: metadata.tags,
        })
    }
}

impl DatasetMetadata {
    /// Save metadata to file
    pub fn save(&self, path: &std::path::Path) -> Result<(), DatasetManagerError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)
            .with_context(|| format!("Failed to write metadata to {}", path.display()))?;
        Ok(())
    }
}

//! Dataset Manager - Central hub for all dataset operations
//!
//! The DatasetManager centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use arrow::array::RecordBatch;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    app::AppHandle,
    database::{
        self, DatasetStatus, DatasetTag, JsonValue, NewDataset, NewTag, PoolExt, SimpleUuid, schema,
    },
    dataset::{DATASET_NAME, METADATA_NAME, batch_writer::BatchWriter},
};

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
#[derive(Debug, Clone)]
pub enum DatasetId {
    Id(i32),
    Uuid(Uuid),
}

/// Information about a pending write operation
#[derive(Debug)]
struct PendingWrite {
    dataset_id: i32,
    path: PathBuf,
    request: CreateDatasetRequest,
}

/// Central manager for all dataset operations
pub struct DatasetManager {
    app: AppHandle,
    pending_writers: Arc<Mutex<HashMap<Uuid, PendingWrite>>>,
}

impl DatasetManager {
    /// Create a new DatasetManager instance
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            pending_writers: Arc::new(Mutex::new(HashMap::new())),
        }
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
            return Err(DatasetManagerError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Dataset path already exists: {}", dataset_path.display()),
                ),
            });
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

        // Store pending write information
        let pending_write = PendingWrite {
            dataset_id,
            path: dataset_path.clone(),
            request: request.clone(),
        };

        {
            let mut pending = self.pending_writers.lock().unwrap();
            pending.insert(uuid, pending_write);
        }

        info!(
            "Created dataset with UUID: {} at path: {:?}",
            uuid, dataset_path
        );
        Ok(uuid)
    }

    /// Write data to a dataset using the provided write token
    pub async fn write_dataset<S, E>(
        &self,
        token: Uuid,
        stream: S,
    ) -> Result<DatasetRecord, DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Get pending write info and remove from pending map
        let pending_write = {
            let mut pending = self.pending_writers.lock().unwrap();
            pending
                .remove(&token)
                .ok_or(DatasetManagerError::InvalidToken)?
        };

        // Update status to Writing
        self.update_status(pending_write.dataset_id, DatasetStatus::Writing)
            .await?;

        // Perform the actual write operation
        let result = self
            .perform_write_async(pending_write.dataset_id, &pending_write.path, stream)
            .await;

        match result {
            Ok(_) => {
                // Update status to Completed and save metadata
                self.update_status(pending_write.dataset_id, DatasetStatus::Completed)
                    .await?;

                // Save metadata file
                let metadata = self.create_metadata(&pending_write).await?;
                metadata.save(&pending_write.path.join(METADATA_NAME))?;

                // Return the completed dataset record
                self.get_dataset(DatasetId::Id(pending_write.dataset_id))
                    .await
            }
            Err(e) => {
                // Update status to Aborted on failure
                let _ = self
                    .update_status(pending_write.dataset_id, DatasetStatus::Aborted)
                    .await;
                Err(e)
            }
        }
    }

    /// Get a dataset by ID or UUID
    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetManagerError> {
        let (dataset, tags) = self
            .app
            .database()
            .interact(move |conn| {
                let dataset = match id {
                    DatasetId::Id(dataset_id) => schema::datasets::table
                        .find(dataset_id)
                        .select(database::Dataset::as_select())
                        .first(conn)?,
                    DatasetId::Uuid(uuid) => schema::datasets::table
                        .filter(schema::datasets::uuid.eq(SimpleUuid(uuid)))
                        .select(database::Dataset::as_select())
                        .first(conn)?,
                };

                let tags = database::DatasetTag::belonging_to(&dataset)
                    .inner_join(schema::tags::table)
                    .select(database::Tag::as_select())
                    .load(conn)?;

                Ok((dataset, tags))
            })
            .await?;

        Ok(DatasetRecord::from_database_models(dataset, tags))
    }

    /// List all datasets with optional filtering
    pub async fn list_datasets(&self) -> Result<Vec<DatasetRecord>, DatasetManagerError> {
        let datasets_with_tags = self
            .app
            .database()
            .interact(|conn| {
                let all_datasets = schema::datasets::table
                    .order(schema::datasets::id.desc())
                    .select(database::Dataset::as_select())
                    .load(conn)?;

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
                use database::schema::datasets::dsl::datasets;
                diesel::update(datasets.find(id))
                    .set(&db_update)
                    .execute(conn)
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
                    // Insert or ignore new tags into the tags table
                    let new_tags: Vec<_> = tags.iter().map(|name| NewTag { name }).collect();
                    diesel::insert_or_ignore_into(schema::tags::table)
                        .values(new_tags)
                        .execute(conn)?;

                    // Get the IDs of the tags that were requested to be added
                    let tag_ids = schema::tags::table
                        .filter(schema::tags::name.eq_any(tags))
                        .select(schema::tags::id)
                        .load::<i32>(conn)?;

                    // Insert or ignore new entries into the datasets_tags table
                    let new_dataset_tags: Vec<_> = tag_ids
                        .into_iter()
                        .map(|tag_id| DatasetTag {
                            dataset_id: id,
                            tag_id,
                        })
                        .collect();
                    diesel::insert_or_ignore_into(schema::datasets_tags::table)
                        .values(new_dataset_tags)
                        .execute(conn)?;

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
                        .filter(schema::tags::name.eq_any(tags))
                        .select(schema::tags::id)
                        .load::<i32>(conn)?;

                    // Delete the entries from datasets_tags table
                    diesel::delete(schema::datasets_tags::table)
                        .filter(schema::datasets_tags::dataset_id.eq(id))
                        .filter(schema::datasets_tags::tag_id.eq_any(tag_ids_to_delete))
                        .execute(conn)?;

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
                use database::schema::datasets::dsl::datasets;
                diesel::delete(datasets.find(id))
                    .execute(conn)
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
                        let new_tags: Vec<_> =
                            request.tags.iter().map(|name| NewTag { name }).collect();
                        diesel::insert_or_ignore_into(schema::tags::table)
                            .values(new_tags)
                            .execute(conn)?;

                        let tag_ids = schema::tags::table
                            .filter(schema::tags::name.eq_any(&request.tags))
                            .select(schema::tags::id)
                            .load::<i32>(conn)?;

                        let dataset_tags: Vec<_> = tag_ids
                            .into_iter()
                            .map(|tag_id| DatasetTag {
                                dataset_id: dataset.id,
                                tag_id,
                            })
                            .collect();

                        diesel::insert_into(schema::datasets_tags::table)
                            .values(dataset_tags)
                            .execute(conn)?;
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
                use database::schema::datasets::dsl::datasets;
                diesel::update(datasets.find(id))
                    .set(database::schema::datasets::status.eq(status))
                    .execute(conn)
                    .context("Failed to update dataset status")
            })
            .await?;

        Ok(())
    }

    /// Create metadata from pending write info and current database state
    async fn create_metadata(
        &self,
        pending_write: &PendingWrite,
    ) -> Result<DatasetMetadata, DatasetManagerError> {
        let record = self
            .get_dataset(DatasetId::Id(pending_write.dataset_id))
            .await?;
        Ok(record.metadata)
    }

    /// Perform the actual write operation using BatchWriter
    async fn perform_write_async<S, E>(
        &self,
        _dataset_id: i32,
        path: &PathBuf,
        mut stream: S,
    ) -> Result<(), DatasetManagerError>
    where
        S: Stream<Item = Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        use futures::StreamExt;

        let dataset_path = path.join(DATASET_NAME);

        // Create the Arrow file
        let file = File::create_new(&dataset_path).with_context(|| {
            format!(
                "Failed to create dataset file at {}",
                dataset_path.display()
            )
        })?;

        // Collect all batches and process them in a blocking task
        let mut batches = Vec::new();
        let mut schema_opt = None;

        while let Some(result) = stream.next().await {
            let batch = result.map_err(|e| DatasetManagerError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Stream error: {}", e),
                ),
            })?;

            if batch.num_rows() == 0 {
                continue; // Skip empty batches
            }

            // Initialize schema from first batch
            if schema_opt.is_none() {
                schema_opt = Some(batch.schema());
            }

            batches.push(batch);
        }

        if batches.is_empty() {
            return Err(DatasetManagerError::SchemaError {
                message: "No data written to the dataset".to_string(),
            });
        }

        let schema = schema_opt.unwrap();

        // Perform the actual writing in a blocking task
        let app_clone = self.app.clone();
        let write_result: Result<Result<(), DatasetManagerError>, _> = app_clone
            .tracker()
            .spawn_blocking(move || -> Result<(), DatasetManagerError> {
                let buf_writer = BufWriter::new(file);
                let mut batch_writer =
                    BatchWriter::new(buf_writer, &schema).map_err(|e| DatasetManagerError::Io {
                        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
                    })?;

                for batch in batches {
                    batch_writer
                        .write(batch)
                        .map_err(|e| DatasetManagerError::Io {
                            source: std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                e.to_string(),
                            ),
                        })?;
                }

                batch_writer.finish().map_err(|e| DatasetManagerError::Io {
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
                })?;

                Ok(())
            })
            .await;

        match write_result {
            Ok(result) => result,
            Err(e) => {
                return Err(DatasetManagerError::Io {
                    source: std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Write task failed: {}", e),
                    ),
                });
            }
        }?;

        Ok(())
    }
}

impl DatasetRecord {
    /// Create DatasetRecord from database models
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

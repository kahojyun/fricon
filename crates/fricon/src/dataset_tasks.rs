//! Dataset task functions - Core business logic for dataset operations
//!
//! This module contains standalone functions that perform the actual dataset
//! operations. Each function takes only the required resources as parameters
//! instead of a broad `AppState`, implementing the core business logic for
//! dataset management with minimal dependencies.

use std::{error::Error as StdError, fs, path::Path};

use arrow_array::RecordBatch;
use deadpool_diesel::sqlite::Pool;
use diesel::prelude::*;
use futures::prelude::*;
use tokio::sync::broadcast;
use tokio_util::task::TaskTracker;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    WorkspaceRoot,
    app::AppEvent,
    database::{self, DatasetStatus, NewDataset, PoolExt, SimpleUuid, schema},
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetManagerError, DatasetRecord, DatasetUpdate,
    },
    reader::{CompletedDataset, DatasetReader},
    write_registry::WriteSessionRegistry,
};

/// Create a new dataset with the given request and data stream
pub async fn do_create_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    event_sender: &broadcast::Sender<AppEvent>,
    write_sessions: &WriteSessionRegistry,
    tracker: &TaskTracker,
    request: CreateDatasetRequest,
    mut stream: Box<
        dyn Stream<Item = Result<RecordBatch, Box<dyn StdError + Send + Sync>>> + Send + Unpin,
    >,
) -> Result<DatasetRecord, DatasetManagerError> {
    let uuid = Uuid::new_v4();
    let dataset_path = root.paths().dataset_path_from_uuid(uuid);

    if dataset_path.exists() {
        warn!("Dataset path already exists: {}", dataset_path.display());
        return Err(DatasetManagerError::path_already_exists(&dataset_path));
    }

    info!(
        "Creating new dataset '{}' with UUID: {}",
        request.name, uuid
    );

    let (dataset, tags) = create_dataset_db_record(database, &request, uuid).await?;

    fs::create_dir_all(&dataset_path)?;

    let event = AppEvent::DatasetCreated {
        id: dataset.id,
        uuid: uuid.to_string(),
        name: request.name.clone(),
        description: request.description.clone(),
        tags: request.tags.clone(),
    };
    let _ = event_sender.send(event);

    info!(
        "Created dataset with UUID: {} at path: {:?}",
        uuid, dataset_path
    );

    let dataset_record = DatasetRecord::from_database_models(dataset, tags);

    let result = perform_write_async(
        write_sessions,
        tracker,
        dataset_record.id,
        &dataset_path,
        &mut stream,
    )
    .await;
    match result {
        Ok(()) => {
            update_dataset_status(database, dataset_record.id, DatasetStatus::Completed).await?;

            let updated_record = do_get_dataset(database, DatasetId::Id(dataset_record.id)).await?;
            Ok(updated_record)
        }
        Err(e) => {
            let _ =
                update_dataset_status(database, dataset_record.id, DatasetStatus::Aborted).await;
            Err(e)
        }
    }
}

/// Delete a dataset by ID
pub async fn do_delete_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    id: i32,
) -> Result<(), DatasetManagerError> {
    let record = do_get_dataset(database, DatasetId::Id(id)).await?;
    let dataset_path = root.paths().dataset_path_from_uuid(record.metadata.uuid);

    delete_dataset_from_db(database, id).await?;

    if dataset_path.exists() {
        fs::remove_dir_all(&dataset_path)?;
    }

    Ok(())
}

/// Get a dataset by ID or UUID
pub async fn do_get_dataset(
    database: &Pool,
    id: DatasetId,
) -> Result<DatasetRecord, DatasetManagerError> {
    let result = database
        .interact(move |conn| {
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
        .await??;

    Ok(DatasetRecord::from_database_models(result.0, result.1))
}

/// List all datasets
pub async fn do_list_datasets(database: &Pool) -> Result<Vec<DatasetRecord>, DatasetManagerError> {
    let datasets_with_tags = database
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

            Ok::<Vec<(database::Dataset, Vec<database::Tag>)>, DatasetManagerError>(
                datasets_with_tags,
            )
        })
        .await??;

    Ok(datasets_with_tags
        .into_iter()
        .map(|(dataset, tags)| DatasetRecord::from_database_models(dataset, tags))
        .collect())
}

/// Update dataset metadata
pub async fn do_update_dataset(
    database: &Pool,
    id: i32,
    update: DatasetUpdate,
) -> Result<(), DatasetManagerError> {
    let db_update = database::DatasetUpdate {
        name: update.name,
        description: update.description,
        favorite: update.favorite,
        status: None,
    };

    database
        .interact(move |conn| {
            database::Dataset::update_metadata(conn, id, &db_update)?;
            Ok::<(), DatasetManagerError>(())
        })
        .await??;

    Ok(())
}

/// Add tags to a dataset
pub async fn do_add_tags(
    database: &Pool,
    id: i32,
    tags: Vec<String>,
) -> Result<(), DatasetManagerError> {
    database
        .interact(move |conn| {
            conn.immediate_transaction::<_, DatasetManagerError, _>(|conn| {
                let created_tags = database::Tag::find_or_create_batch(conn, &tags)?;
                let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

                database::DatasetTag::create_associations(conn, id, &tag_ids)?;
                Ok(())
            })
        })
        .await??;

    Ok(())
}

/// Remove tags from a dataset
pub async fn do_remove_tags(
    database: &Pool,
    id: i32,
    tags: Vec<String>,
) -> Result<(), DatasetManagerError> {
    database
        .interact(move |conn| {
            conn.immediate_transaction::<_, DatasetManagerError, _>(|conn| {
                let tag_ids_to_delete = schema::tags::table
                    .filter(schema::tags::name.eq_any(&tags))
                    .select(schema::tags::id)
                    .load::<i32>(conn)?;

                database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
                Ok(())
            })
        })
        .await??;

    Ok(())
}

/// Get a dataset reader for the specified dataset
pub async fn do_get_dataset_reader(
    database: &Pool,
    root: &WorkspaceRoot,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, DatasetManagerError> {
    let record = do_get_dataset(database, id).await?;
    match record.metadata.status {
        DatasetStatus::Completed | DatasetStatus::Aborted => {
            // Aborted datasets may still have partially written chunk files (valid up to
            // last flush).
            let dataset_path = root.paths().dataset_path_from_uuid(record.metadata.uuid);
            let completed = CompletedDataset::open(&dataset_path)?;
            Ok(DatasetReader::Completed(completed))
        }
        DatasetStatus::Writing => {
            if let Some(session) = write_sessions.get(record.id) {
                return Ok(DatasetReader::Live(session.live().clone()));
            }
            // Fallback: if writer already dropped but directory exists, expose as Completed
            // view.
            let dataset_path = root.paths().dataset_path_from_uuid(record.metadata.uuid);
            if dataset_path.exists() {
                let completed = CompletedDataset::open(&dataset_path)?;
                return Ok(DatasetReader::Completed(completed));
            }
            Err(DatasetManagerError::io_invalid_data(
                "Dataset in Writing state has no active session and no file yet",
            ))
        }
    }
}

// Helper functions

async fn create_dataset_db_record(
    database: &Pool,
    request: &CreateDatasetRequest,
    uuid: Uuid,
) -> Result<(database::Dataset, Vec<database::Tag>), DatasetManagerError> {
    let request = request.clone();
    let res = database
        .interact(move |conn| {
            conn.immediate_transaction::<_, DatasetManagerError, _>(|conn| {
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
                    let created_tags = database::Tag::find_or_create_batch(conn, &request.tags)?;
                    let tag_ids: Vec<i32> = created_tags.iter().map(|tag| tag.id).collect();
                    database::DatasetTag::create_associations(conn, dataset.id, &tag_ids)?;
                    created_tags
                };

                Ok((dataset, tags))
            })
        })
        .await??;

    Ok(res)
}

async fn update_dataset_status(
    database: &Pool,
    id: i32,
    status: DatasetStatus,
) -> Result<(), DatasetManagerError> {
    database
        .interact(move |conn| {
            database::Dataset::update_status(conn, id, status)?;
            Ok::<(), DatasetManagerError>(())
        })
        .await??;

    Ok(())
}

async fn delete_dataset_from_db(database: &Pool, id: i32) -> Result<(), DatasetManagerError> {
    database
        .interact(move |conn| {
            database::Dataset::delete_from_db(conn, id)?;
            Ok::<(), DatasetManagerError>(())
        })
        .await??;

    Ok(())
}

async fn perform_write_async(
    write_sessions: &WriteSessionRegistry,
    tracker: &TaskTracker,
    dataset_id: i32,
    path: &Path,
    stream: &mut Box<
        dyn Stream<Item = Result<RecordBatch, Box<dyn StdError + Send + Sync>>> + Send + Unpin,
    >,
) -> Result<(), DatasetManagerError> {
    let first_batch = match stream.next().await {
        Some(Ok(batch)) => batch,
        Some(Err(e)) => {
            return Err(DatasetManagerError::io_invalid_data(format!(
                "Stream error: {e}"
            )));
        }
        None => return Err(DatasetManagerError::empty_stream()),
    };

    let session_guard = write_sessions.create(dataset_id, tracker, path, first_batch.schema());

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
                    write_error = Some(DatasetManagerError::io_invalid_data(format!(
                        "Stream error: {e}"
                    )));
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

//! Dataset task functions - Core business logic for dataset operations
//!
//! This module contains standalone functions that perform the actual dataset
//! operations. Each function takes only the required resources as parameters
//! instead of a broad `AppState`, implementing the core business logic for
//! dataset management with minimal dependencies.

use arrow_array::RecordBatchReader;
use diesel::prelude::*;
use tokio::sync::broadcast;
use tracing::info;
use uuid::Uuid;

use crate::{
    DEFAULT_DATASET_LIST_LIMIT, WorkspaceRoot,
    app::AppEvent,
    database::{self, DatasetStatus, NewDataset, Pool, SimpleUuid, schema},
    dataset_fs,
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetReader, DatasetRecord, DatasetUpdate, Error,
        write_registry::WriteSessionRegistry,
    },
};

/// Create a new dataset with the given request and data stream
pub fn do_create_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    event_sender: &broadcast::Sender<AppEvent>,
    write_sessions: &WriteSessionRegistry,
    request: CreateDatasetRequest,
    batches: impl RecordBatchReader,
) -> Result<DatasetRecord, Error> {
    let uid = Uuid::new_v4();
    let dataset_path = root.paths().dataset_path_from_uid(uid);

    info!("Creating new dataset '{}' with uid: {}", request.name, uid);
    dataset_fs::create_dataset(&dataset_path)?;
    let (dataset, tags) = create_dataset_db_record(&mut *database.get()?, &request, uid)?;

    let event = AppEvent::DatasetCreated {
        id: dataset.id,
        name: request.name,
        description: request.description,
        favorite: dataset.favorite,
        tags: request.tags,
        status: dataset.status,
        created_at: dataset.created_at.and_utc(),
    };
    let _ = event_sender.send(event);

    info!(
        "Created dataset with UUID: {} at path: {:?}",
        uid, dataset_path
    );

    let dataset_record = DatasetRecord::from_database_models(dataset, tags);

    write_sessions.with_session(
        dataset_record.id,
        dataset_path,
        batches.schema(),
        move |session| {
            let write_result = batches.into_iter().try_for_each(|batch| {
                let batch = batch.map_err(|e| Error::BatchStream {
                    message: e.to_string(),
                })?;
                session.write(batch)
            });
            let mut conn = database.get()?;
            match write_result {
                Ok(()) => {
                    database::Dataset::update_status(
                        &mut conn,
                        dataset_record.id,
                        DatasetStatus::Completed,
                    )?;
                    let updated_record =
                        do_get_dataset(&mut conn, DatasetId::Id(dataset_record.id))?;
                    Ok(updated_record)
                }
                Err(e) => {
                    let _ = database::Dataset::update_status(
                        &mut conn,
                        dataset_record.id,
                        DatasetStatus::Aborted,
                    );
                    Err(e)
                }
            }
        },
    )
}

/// Delete a dataset by ID
pub fn do_delete_dataset(database: &Pool, root: &WorkspaceRoot, id: i32) -> Result<(), Error> {
    let mut conn = database.get()?;
    let record = do_get_dataset(&mut conn, DatasetId::Id(id))?;
    let dataset_path = root.paths().dataset_path_from_uid(record.metadata.uid);
    database::Dataset::delete_from_db(&mut conn, id)?;
    drop(conn);

    dataset_fs::delete_dataset(&dataset_path)?;

    Ok(())
}

/// Get a dataset by ID or UUID
pub fn do_get_dataset(conn: &mut SqliteConnection, id: DatasetId) -> Result<DatasetRecord, Error> {
    let dataset = match id {
        DatasetId::Id(dataset_id) => database::Dataset::find_by_id(conn, dataset_id)?,
        DatasetId::Uid(uid) => database::Dataset::find_by_uid(conn, uid)?,
    };

    let Some(dataset) = dataset else {
        let id_str = match id {
            DatasetId::Id(i) => i.to_string(),
            DatasetId::Uid(u) => u.to_string(),
        };
        return Err(Error::NotFound { id: id_str });
    };

    let tags = dataset.load_tags(conn)?;

    Ok(DatasetRecord::from_database_models(dataset, tags))
}

/// List datasets, optionally filtered by name and tags
pub fn do_list_datasets(
    conn: &mut SqliteConnection,
    search: Option<&str>,
    tags: Option<&[String]>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<DatasetRecord>, Error> {
    let search = search.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let tag_filters = tags.and_then(|tags| {
        let cleaned: Vec<String> = tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    });
    let tagged_dataset_ids = if let Some(tag_filters) = tag_filters.as_ref() {
        let ids = schema::datasets_tags::table
            .inner_join(schema::tags::table)
            .filter(schema::tags::name.eq_any(tag_filters))
            .select(schema::datasets_tags::dataset_id)
            .distinct()
            .load::<i32>(conn)?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        Some(ids)
    } else {
        None
    };

    let mut query = schema::datasets::table.into_boxed();
    if let Some(search) = search {
        let pattern = format!("%{search}%");
        query = query.filter(schema::datasets::name.like(pattern));
    }
    if let Some(ids) = tagged_dataset_ids {
        query = query.filter(schema::datasets::id.eq_any(ids));
    }

    let limit = limit.unwrap_or(DEFAULT_DATASET_LIST_LIMIT).max(0);
    let offset = offset.unwrap_or(0).max(0);
    let all_datasets = query
        .order(schema::datasets::id.desc())
        .limit(limit)
        .offset(offset)
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

    Ok(datasets_with_tags
        .into_iter()
        .map(|(dataset, tags)| DatasetRecord::from_database_models(dataset, tags))
        .collect())
}

/// Update dataset metadata
pub fn do_update_dataset(
    conn: &mut SqliteConnection,
    id: i32,
    update: DatasetUpdate,
) -> Result<(), Error> {
    let db_update = database::DatasetUpdate {
        name: update.name,
        description: update.description,
        favorite: update.favorite,
        status: None,
    };
    database::Dataset::update_metadata(conn, id, &db_update)?;
    Ok(())
}

/// Add tags to a dataset
pub fn do_add_tags(conn: &mut SqliteConnection, id: i32, tags: &[String]) -> Result<(), Error> {
    conn.immediate_transaction(|conn| {
        let created_tags = database::Tag::find_or_create_batch(conn, tags)?;
        let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

        database::DatasetTag::create_associations(conn, id, &tag_ids)?;
        Ok(())
    })
}

/// Remove tags from a dataset
pub fn do_remove_tags(conn: &mut SqliteConnection, id: i32, tags: &[String]) -> Result<(), Error> {
    conn.immediate_transaction(|conn| {
        let tag_ids_to_delete = schema::tags::table
            .filter(schema::tags::name.eq_any(tags))
            .select(schema::tags::id)
            .load::<i32>(conn)?;

        database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
        Ok(())
    })
}

/// Get a dataset reader for the specified dataset
pub fn do_get_dataset_reader(
    database: &Pool,
    root: &WorkspaceRoot,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, Error> {
    let mut conn = database.get()?;
    let dataset = do_get_dataset(&mut conn, id)?;
    if let Some(handle) = write_sessions.get(dataset.id) {
        Ok(DatasetReader::from_handle(handle)?)
    } else {
        let path = root.paths().dataset_path_from_uid(dataset.metadata.uid);
        Ok(DatasetReader::open_dir(path)?)
    }
}

// Helper functions

fn create_dataset_db_record(
    conn: &mut SqliteConnection,
    request: &CreateDatasetRequest,
    uid: Uuid,
) -> Result<(database::Dataset, Vec<database::Tag>), Error> {
    conn.immediate_transaction(|conn| {
        let new_dataset = NewDataset {
            uid: SimpleUuid(uid),
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
}

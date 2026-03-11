//! Dataset task functions - Core business logic for dataset operations
//!
//! This module contains standalone functions that perform the actual dataset
//! operations. Each function takes only the required resources as parameters
//! instead of a broad `AppState`, implementing the core business logic for
//! dataset management with minimal dependencies.

use std::path::PathBuf;

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use diesel::prelude::*;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    DEFAULT_DATASET_LIST_LIMIT,
    database::{self, DatasetStatus, NewDataset, Pool, SimpleUuid, schema},
    dataset_catalog::{
        DatasetCatalogError, DatasetId, DatasetListQuery, DatasetRecord, DatasetSortBy,
        DatasetUpdate, SortDirection,
    },
    dataset_ingest::{
        CreateDatasetRequest, CreateIngestEvent, CreateTerminal, WriteSessionGuard,
        WriteSessionRegistry,
    },
    dataset_read::DatasetReader,
    runtime::app::AppEvent,
    storage,
    workspace::WorkspaceRoot,
};

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetRepo {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(database::Dataset, Vec<database::Tag>), DatasetCatalogError>;
    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), DatasetCatalogError>;
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetCatalogError>;
}

impl DatasetRepo for Pool {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(database::Dataset, Vec<database::Tag>), DatasetCatalogError> {
        create_dataset_db_record(&mut *self.get()?, request, uid)
    }

    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), DatasetCatalogError> {
        let mut conn = self.get()?;
        database::Dataset::update_status(&mut conn, id, status)?;
        Ok(())
    }

    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetCatalogError> {
        let mut conn = self.get()?;
        do_get_dataset(&mut conn, id)
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetStore {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, DatasetCatalogError>;
}

impl DatasetStore for WorkspaceRoot {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, DatasetCatalogError> {
        let path = self.paths().dataset_path_from_uid(uid);
        storage::create_dataset(&path)?;
        Ok(path)
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetEvents {
    fn send_dataset_created(&self, event: AppEvent);
}

impl DatasetEvents for broadcast::Sender<AppEvent> {
    fn send_dataset_created(&self, event: AppEvent) {
        let _ = self.send(event);
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait WriteSessionGuardOps {
    fn write(&mut self, batch: RecordBatch) -> Result<(), DatasetCatalogError>;
    fn commit(self) -> Result<(), DatasetCatalogError>;
    fn abort(self) -> Result<(), DatasetCatalogError>;
}

impl WriteSessionGuardOps for WriteSessionGuard {
    fn write(&mut self, batch: RecordBatch) -> Result<(), DatasetCatalogError> {
        self.write_batch(batch)
    }

    fn commit(self) -> Result<(), DatasetCatalogError> {
        self.commit_session()
    }

    fn abort(self) -> Result<(), DatasetCatalogError> {
        self.abort_session()
    }
}

pub(super) trait WriteSessions {
    type Guard: WriteSessionGuardOps;
    fn start_session(&self, id: i32, path: PathBuf, schema: SchemaRef) -> Self::Guard;
}

impl WriteSessions for WriteSessionRegistry {
    type Guard = WriteSessionGuard;

    fn start_session(&self, id: i32, path: PathBuf, schema: SchemaRef) -> Self::Guard {
        WriteSessionRegistry::start_session(self, id, path, schema)
    }
}

#[instrument(
    skip(repo, store, events, write_sessions, events_rx, request),
    fields(dataset.name = %request.name, tags.count = request.tags.len())
)]
fn create_dataset_with<R, S, E, W>(
    repo: &R,
    store: &S,
    events: &E,
    write_sessions: &W,
    request: CreateDatasetRequest,
    mut events_rx: mpsc::Receiver<CreateIngestEvent>,
) -> Result<DatasetRecord, DatasetCatalogError>
where
    R: DatasetRepo,
    S: DatasetStore,
    E: DatasetEvents,
    W: WriteSessions,
{
    let uid = Uuid::new_v4();
    let dataset_path = store.create_dataset_dir(uid)?;

    let (dataset, tags) = repo.create_dataset_record(&request, uid)?;
    info!(dataset.id = dataset.id, %uid, name = %request.name, "Dataset record created");

    let event = AppEvent::DatasetCreated {
        id: dataset.id,
        name: request.name,
        description: request.description,
        favorite: dataset.favorite,
        tags: request.tags,
        status: dataset.status,
        created_at: dataset.created_at.and_utc(),
    };
    events.send_dataset_created(event);

    let dataset_record = DatasetRecord::from_database_models(dataset, tags);

    let mut session = None;
    let terminal = loop {
        let Some(event) = events_rx.blocking_recv() else {
            break CreateTerminal::Abort;
        };

        match event {
            CreateIngestEvent::Batch(batch) => {
                let session_ref = session.get_or_insert_with(|| {
                    write_sessions.start_session(
                        dataset_record.id,
                        dataset_path.clone(),
                        batch.schema(),
                    )
                });
                if let Err(error) = session_ref.write(batch) {
                    debug!(error = %error, "Failed to write batch into dataset session");
                    break CreateTerminal::Abort;
                }
            }
            CreateIngestEvent::Terminal(terminal) => break terminal,
        }
    };

    match terminal {
        CreateTerminal::Finish => {
            if let Some(session) = session.take()
                && let Err(error) = session.commit()
            {
                debug!(error = %error, "Failed to commit dataset session, switching to aborted");
                let _ = repo.update_status(dataset_record.id, DatasetStatus::Aborted);
                return Err(error);
            }
            repo.update_status(dataset_record.id, DatasetStatus::Completed)?;
            info!(dataset.id = dataset_record.id, %uid, "Dataset write completed");
            repo.get_dataset(DatasetId::Id(dataset_record.id))
        }
        CreateTerminal::Abort => {
            if let Some(session) = session.take()
                && let Err(error) = session.abort()
            {
                debug!(error = %error, "Failed to abort dataset session, keeping aborted status");
            }
            repo.update_status(dataset_record.id, DatasetStatus::Aborted)?;
            info!(dataset.id = dataset_record.id, "Dataset write aborted");
            repo.get_dataset(DatasetId::Id(dataset_record.id))
        }
    }
}

/// Create a new dataset with the given request and data stream
#[instrument(
    skip(database, root, event_sender, write_sessions, events_rx, request),
    fields(dataset.name = %request.name, tags.count = request.tags.len())
)]
pub(crate) fn do_create_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    event_sender: &broadcast::Sender<AppEvent>,
    write_sessions: &WriteSessionRegistry,
    request: CreateDatasetRequest,
    events_rx: mpsc::Receiver<CreateIngestEvent>,
) -> Result<DatasetRecord, DatasetCatalogError> {
    create_dataset_with(
        database,
        root,
        event_sender,
        write_sessions,
        request,
        events_rx,
    )
}

/// Delete a dataset by ID
#[instrument(skip(database, root), fields(dataset.id = id))]
pub(crate) fn do_delete_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    id: i32,
) -> Result<(), DatasetCatalogError> {
    let mut conn = database.get()?;
    let record = do_get_dataset(&mut conn, DatasetId::Id(id))?;
    let uid = record.metadata.uid;
    let dataset_path = root.paths().dataset_path_from_uid(uid);
    database::Dataset::delete_from_db(&mut conn, id)?;
    drop(conn);

    storage::delete_dataset(&dataset_path)?;
    info!(dataset.id = id, %uid, "Dataset deleted");

    Ok(())
}

/// Get a dataset by ID or UUID
#[instrument(skip(conn, id), fields(dataset.id = ?id))]
pub(crate) fn do_get_dataset(
    conn: &mut SqliteConnection,
    id: DatasetId,
) -> Result<DatasetRecord, DatasetCatalogError> {
    let dataset = match id {
        DatasetId::Id(dataset_id) => database::Dataset::find_by_id(conn, dataset_id)?,
        DatasetId::Uid(uid) => database::Dataset::find_by_uid(conn, uid)?,
    };

    let Some(dataset) = dataset else {
        let id_str = match id {
            DatasetId::Id(i) => i.to_string(),
            DatasetId::Uid(u) => u.to_string(),
        };
        return Err(DatasetCatalogError::NotFound { id: id_str });
    };

    let tags = dataset.load_tags(conn)?;

    Ok(DatasetRecord::from_database_models(dataset, tags))
}

fn normalize_search(search: Option<&str>) -> Option<&str> {
    search.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_tag_filters(tags: Option<&[String]>) -> Option<Vec<String>> {
    tags.and_then(|tags| {
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
    })
}

fn resolve_tagged_dataset_ids(
    conn: &mut SqliteConnection,
    tag_filters: Option<&[String]>,
) -> Result<Option<Vec<i32>>, DatasetCatalogError> {
    let Some(tag_filters) = tag_filters else {
        return Ok(None);
    };

    let ids = schema::datasets_tags::table
        .inner_join(schema::tags::table)
        .filter(schema::tags::name.eq_any(tag_filters))
        .select(schema::datasets_tags::dataset_id)
        .distinct()
        .load::<i32>(conn)?;

    if ids.is_empty() {
        Ok(Some(Vec::new()))
    } else {
        Ok(Some(ids))
    }
}

fn normalize_statuses(statuses: Option<&[DatasetStatus]>) -> Option<Vec<DatasetStatus>> {
    statuses.and_then(|statuses| {
        let mut deduped = statuses.to_vec();
        deduped.sort_unstable_by_key(|status| *status as u8);
        deduped.dedup();
        if deduped.is_empty() {
            None
        } else {
            Some(deduped)
        }
    })
}

fn map_datasets_with_tags(
    conn: &mut SqliteConnection,
    all_datasets: Vec<database::Dataset>,
) -> Result<Vec<DatasetRecord>, DatasetCatalogError> {
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

/// List datasets with filtering, sorting, and pagination options.
#[instrument(skip(conn, query_options))]
pub(crate) fn do_list_datasets(
    conn: &mut SqliteConnection,
    query_options: &DatasetListQuery,
) -> Result<Vec<DatasetRecord>, DatasetCatalogError> {
    let search = normalize_search(query_options.search.as_deref());
    let tag_filters = normalize_tag_filters(query_options.tags.as_deref());
    let tagged_dataset_ids = resolve_tagged_dataset_ids(conn, tag_filters.as_deref())?;
    if tagged_dataset_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }
    let statuses = normalize_statuses(query_options.statuses.as_deref());

    let mut query = schema::datasets::table.into_boxed();
    if let Some(search) = search {
        let pattern = format!("%{search}%");
        query = query.filter(schema::datasets::name.like(pattern));
    }
    if let Some(ids) = tagged_dataset_ids {
        query = query.filter(schema::datasets::id.eq_any(ids));
    }
    if query_options.favorite_only {
        query = query.filter(schema::datasets::favorite.eq(true));
    }
    if let Some(statuses) = statuses {
        query = query.filter(schema::datasets::status.eq_any(statuses));
    }

    query = match (query_options.sort_by, query_options.sort_direction) {
        (DatasetSortBy::Id, SortDirection::Asc) => query.order(schema::datasets::id.asc()),
        (DatasetSortBy::Id, SortDirection::Desc) => query.order(schema::datasets::id.desc()),
        (DatasetSortBy::Name, SortDirection::Asc) => {
            query.order((schema::datasets::name.asc(), schema::datasets::id.desc()))
        }
        (DatasetSortBy::Name, SortDirection::Desc) => {
            query.order((schema::datasets::name.desc(), schema::datasets::id.desc()))
        }
        (DatasetSortBy::CreatedAt, SortDirection::Asc) => query.order((
            schema::datasets::created_at.asc(),
            schema::datasets::id.desc(),
        )),
        (DatasetSortBy::CreatedAt, SortDirection::Desc) => query.order((
            schema::datasets::created_at.desc(),
            schema::datasets::id.desc(),
        )),
    };

    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_DATASET_LIST_LIMIT)
        .max(0);
    let offset = query_options.offset.unwrap_or(0).max(0);
    let all_datasets: Vec<database::Dataset> = query
        .limit(limit)
        .offset(offset)
        .select(database::Dataset::as_select())
        .load(conn)?;
    map_datasets_with_tags(conn, all_datasets)
}

/// List all known dataset tags in ascending name order.
#[instrument(skip(conn))]
pub(crate) fn do_list_dataset_tags(
    conn: &mut SqliteConnection,
) -> Result<Vec<String>, DatasetCatalogError> {
    let tags = schema::tags::table
        .select(schema::tags::name)
        .order(schema::tags::name.asc())
        .load(conn)?;
    Ok(tags)
}

/// Update dataset metadata
#[instrument(skip(conn, update), fields(dataset.id = id))]
pub(crate) fn do_update_dataset(
    conn: &mut SqliteConnection,
    id: i32,
    update: DatasetUpdate,
) -> Result<(), DatasetCatalogError> {
    let db_update = database::DatasetUpdate {
        name: update.name,
        description: update.description,
        favorite: update.favorite,
        status: None,
    };
    database::Dataset::update_metadata(conn, id, &db_update)?;
    debug!(dataset.id = id, "Dataset metadata updated");
    Ok(())
}

/// Add tags to a dataset
#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_add_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    conn.immediate_transaction(|conn| {
        let created_tags = database::Tag::find_or_create_batch(conn, tags)?;
        let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

        database::DatasetTag::create_associations(conn, id, &tag_ids)?;
        Ok::<(), DatasetCatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags added to dataset");
    Ok(())
}

/// Remove tags from a dataset
#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_remove_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    conn.immediate_transaction(|conn| {
        let tag_ids_to_delete = schema::tags::table
            .filter(schema::tags::name.eq_any(tags))
            .select(schema::tags::id)
            .load::<i32>(conn)?;

        database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
        Ok::<(), DatasetCatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags removed from dataset");
    Ok(())
}

/// Get a dataset reader for the specified dataset
#[instrument(skip(database, root, write_sessions, id), fields(dataset.id = ?id))]
pub(crate) fn do_get_dataset_reader(
    database: &Pool,
    root: &WorkspaceRoot,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, DatasetCatalogError> {
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
) -> Result<(database::Dataset, Vec<database::Tag>), DatasetCatalogError> {
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, sync::Arc};

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_schema::{DataType, Field, Schema};
    use chrono::{NaiveDate, NaiveDateTime, Utc};
    use diesel::{
        Connection, ExpressionMethods, RunQueryDsl, connection::SimpleConnection,
        sqlite::SqliteConnection,
    };
    use mockall::{Sequence, predicate::eq};
    use tokio::sync::mpsc;

    use super::*;
    struct FakeWriteSessions {
        guard: RefCell<Option<MockWriteSessionGuardOps>>,
    }

    impl FakeWriteSessions {
        fn new(guard: MockWriteSessionGuardOps) -> Self {
            Self {
                guard: RefCell::new(Some(guard)),
            }
        }
    }

    impl WriteSessions for FakeWriteSessions {
        type Guard = MockWriteSessionGuardOps;

        fn start_session(&self, _id: i32, _path: PathBuf, _schema: SchemaRef) -> Self::Guard {
            self.guard.borrow_mut().take().expect("guard")
        }
    }

    fn dataset_from_request(
        request: &CreateDatasetRequest,
        uid: Uuid,
        id: i32,
        status: DatasetStatus,
    ) -> database::Dataset {
        database::Dataset {
            id,
            uid: database::SimpleUuid(uid),
            name: request.name.clone(),
            description: request.description.clone(),
            favorite: false,
            status,
            created_at: Utc::now().naive_utc(),
        }
    }

    fn setup_common_mocks(
        seq: &mut Sequence,
        dataset_id: i32,
    ) -> (MockDatasetStore, MockDatasetRepo, MockDatasetEvents) {
        let mut store = MockDatasetStore::new();
        store
            .expect_create_dataset_dir()
            .times(1)
            .in_sequence(seq)
            .returning(|_| Ok(PathBuf::from("/tmp/fricon_test_dataset")));

        let mut repo = MockDatasetRepo::new();
        repo.expect_create_dataset_record()
            .times(1)
            .in_sequence(seq)
            .returning(move |request, uid| {
                let dataset =
                    dataset_from_request(request, uid, dataset_id, DatasetStatus::Writing);
                Ok((dataset, vec![]))
            });

        let mut events = MockDatasetEvents::new();
        events
            .expect_send_dataset_created()
            .times(1)
            .in_sequence(seq)
            .returning(|_| ());

        (store, repo, events)
    }

    fn sample_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let array = Arc::new(Int32Array::from(vec![1, 2, 3]));
        RecordBatch::try_new(schema, vec![array]).expect("batch")
    }

    fn events_rx(events: Vec<CreateIngestEvent>) -> mpsc::Receiver<CreateIngestEvent> {
        let (tx, rx) = mpsc::channel(16);
        for event in events {
            tx.try_send(event).expect("send event");
        }
        drop(tx);
        rx
    }

    #[test]
    fn create_success_commits_before_completed() {
        let mut seq = Sequence::new();
        let dataset_id = 1;

        let (store, mut repo, events) = setup_common_mocks(&mut seq, dataset_id);

        let mut guard = MockWriteSessionGuardOps::new();
        guard
            .expect_write()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        guard
            .expect_commit()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));

        repo.expect_update_status()
            .with(eq(dataset_id), eq(DatasetStatus::Completed))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(()));
        repo.expect_get_dataset()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| {
                let dataset = database::Dataset {
                    id: dataset_id,
                    uid: database::SimpleUuid(Uuid::new_v4()),
                    name: "name".to_string(),
                    description: "desc".to_string(),
                    favorite: false,
                    status: DatasetStatus::Completed,
                    created_at: Utc::now().naive_utc(),
                };
                Ok(DatasetRecord::from_database_models(dataset, vec![]))
            });

        let sessions = FakeWriteSessions::new(guard);

        let batch = sample_batch();
        let request = CreateDatasetRequest {
            name: "name".to_string(),
            description: "desc".to_string(),
            tags: vec!["t1".to_string()],
        };
        let events_rx = events_rx(vec![
            CreateIngestEvent::Batch(batch),
            CreateIngestEvent::Terminal(CreateTerminal::Finish),
        ]);

        let result = create_dataset_with(&repo, &store, &events, &sessions, request, events_rx);
        assert!(result.is_ok());
    }

    #[test]
    fn create_commit_failure_returns_error() {
        let mut seq = Sequence::new();
        let dataset_id = 1;

        let (store, mut repo, events) = setup_common_mocks(&mut seq, dataset_id);

        let mut guard = MockWriteSessionGuardOps::new();
        guard
            .expect_write()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        guard
            .expect_commit()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| {
                Err(DatasetCatalogError::NotFound {
                    id: "commit failed".to_string(),
                })
            });

        repo.expect_update_status()
            .with(eq(dataset_id), eq(DatasetStatus::Aborted))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(()));

        let sessions = FakeWriteSessions::new(guard);

        let batch = sample_batch();
        let request = CreateDatasetRequest {
            name: "name".to_string(),
            description: "desc".to_string(),
            tags: vec![],
        };
        let events_rx = events_rx(vec![
            CreateIngestEvent::Batch(batch),
            CreateIngestEvent::Terminal(CreateTerminal::Finish),
        ]);

        let result = create_dataset_with(&repo, &store, &events, &sessions, request, events_rx);
        assert!(matches!(result, Err(DatasetCatalogError::NotFound { .. })));
    }

    #[test]
    fn create_abort_returns_aborted_dataset() {
        let mut seq = Sequence::new();
        let dataset_id = 1;

        let (store, mut repo, events) = setup_common_mocks(&mut seq, dataset_id);

        let mut guard = MockWriteSessionGuardOps::new();
        guard
            .expect_write()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        guard
            .expect_abort()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));

        repo.expect_update_status()
            .with(eq(dataset_id), eq(DatasetStatus::Aborted))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(()));
        repo.expect_get_dataset()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| {
                let dataset = database::Dataset {
                    id: dataset_id,
                    uid: database::SimpleUuid(Uuid::new_v4()),
                    name: "name".to_string(),
                    description: "desc".to_string(),
                    favorite: false,
                    status: DatasetStatus::Aborted,
                    created_at: Utc::now().naive_utc(),
                };
                Ok(DatasetRecord::from_database_models(dataset, vec![]))
            });

        let sessions = FakeWriteSessions::new(guard);

        let batch = sample_batch();
        let request = CreateDatasetRequest {
            name: "name".to_string(),
            description: "desc".to_string(),
            tags: vec![],
        };
        let events_rx = events_rx(vec![
            CreateIngestEvent::Batch(batch),
            CreateIngestEvent::Terminal(CreateTerminal::Abort),
        ]);

        let result = create_dataset_with(&repo, &store, &events, &sessions, request, events_rx);
        assert_eq!(
            result.expect("aborted dataset").metadata.status,
            DatasetStatus::Aborted
        );
    }

    #[test]
    fn create_channel_closed_without_terminal_returns_aborted_dataset() {
        let mut seq = Sequence::new();
        let dataset_id = 1;

        let (store, mut repo, events) = setup_common_mocks(&mut seq, dataset_id);

        let mut guard = MockWriteSessionGuardOps::new();
        guard
            .expect_write()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        guard
            .expect_abort()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));

        repo.expect_update_status()
            .with(eq(dataset_id), eq(DatasetStatus::Aborted))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(()));
        repo.expect_get_dataset()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| {
                let dataset = database::Dataset {
                    id: dataset_id,
                    uid: database::SimpleUuid(Uuid::new_v4()),
                    name: "name".to_string(),
                    description: "desc".to_string(),
                    favorite: false,
                    status: DatasetStatus::Aborted,
                    created_at: Utc::now().naive_utc(),
                };
                Ok(DatasetRecord::from_database_models(dataset, vec![]))
            });

        let sessions = FakeWriteSessions::new(guard);

        let batch = sample_batch();
        let request = CreateDatasetRequest {
            name: "name".to_string(),
            description: "desc".to_string(),
            tags: vec![],
        };
        let events_rx = events_rx(vec![CreateIngestEvent::Batch(batch)]);

        let result = create_dataset_with(&repo, &store, &events, &sessions, request, events_rx);
        assert_eq!(
            result.expect("aborted dataset").metadata.status,
            DatasetStatus::Aborted
        );
    }

    fn setup_list_query_db() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:").expect("in-memory sqlite");
        conn.batch_execute(
            r"
            CREATE TABLE datasets (
                id INTEGER PRIMARY KEY NOT NULL,
                uid TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                favorite BOOLEAN NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL
            );
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE datasets_tags (
                dataset_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (dataset_id, tag_id)
            );
            ",
        )
        .expect("create schema");
        conn
    }

    fn date(day: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 1, day)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
    }

    fn insert_dataset(
        conn: &mut SqliteConnection,
        id: i32,
        name: &str,
        favorite: bool,
        status: DatasetStatus,
        created_at: NaiveDateTime,
    ) {
        diesel::insert_into(schema::datasets::table)
            .values((
                schema::datasets::id.eq(id),
                schema::datasets::uid.eq(SimpleUuid(Uuid::new_v4())),
                schema::datasets::name.eq(name),
                schema::datasets::description.eq("desc"),
                schema::datasets::favorite.eq(favorite),
                schema::datasets::status.eq(status),
                schema::datasets::created_at.eq(created_at),
            ))
            .execute(conn)
            .expect("insert dataset");
    }

    fn insert_tag(conn: &mut SqliteConnection, id: i32, name: &str) {
        diesel::insert_into(schema::tags::table)
            .values((schema::tags::id.eq(id), schema::tags::name.eq(name)))
            .execute(conn)
            .expect("insert tag");
    }

    fn link_dataset_tag(conn: &mut SqliteConnection, dataset_id: i32, tag_id: i32) {
        diesel::insert_into(schema::datasets_tags::table)
            .values((
                schema::datasets_tags::dataset_id.eq(dataset_id),
                schema::datasets_tags::tag_id.eq(tag_id),
            ))
            .execute(conn)
            .expect("link dataset tag");
    }

    #[test]
    fn list_datasets_filters_by_favorite_status_and_sorts_by_name() {
        let mut conn = setup_list_query_db();
        insert_dataset(
            &mut conn,
            1,
            "beta",
            false,
            DatasetStatus::Completed,
            date(1),
        );
        insert_dataset(
            &mut conn,
            2,
            "alpha",
            true,
            DatasetStatus::Completed,
            date(2),
        );
        insert_dataset(&mut conn, 3, "gamma", true, DatasetStatus::Writing, date(3));

        let datasets = do_list_datasets(
            &mut conn,
            &DatasetListQuery {
                favorite_only: true,
                statuses: Some(vec![DatasetStatus::Completed]),
                sort_by: DatasetSortBy::Name,
                sort_direction: SortDirection::Asc,
                ..DatasetListQuery::default()
            },
        )
        .expect("list datasets");

        let ids: Vec<i32> = datasets.into_iter().map(|dataset| dataset.id).collect();
        assert_eq!(ids, vec![2]);
    }

    #[test]
    fn list_datasets_tag_filter_matches_any_selected_tag() {
        let mut conn = setup_list_query_db();
        insert_dataset(
            &mut conn,
            1,
            "one",
            false,
            DatasetStatus::Completed,
            date(1),
        );
        insert_dataset(
            &mut conn,
            2,
            "two",
            false,
            DatasetStatus::Completed,
            date(2),
        );
        insert_tag(&mut conn, 10, "vision");
        insert_tag(&mut conn, 11, "nlp");
        link_dataset_tag(&mut conn, 1, 10);
        link_dataset_tag(&mut conn, 2, 11);

        let datasets = do_list_datasets(
            &mut conn,
            &DatasetListQuery {
                tags: Some(vec!["vision".to_string(), "missing".to_string()]),
                ..DatasetListQuery::default()
            },
        )
        .expect("list datasets");

        let ids: Vec<i32> = datasets.into_iter().map(|dataset| dataset.id).collect();
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn list_datasets_default_sort_and_pagination() {
        let mut conn = setup_list_query_db();
        insert_dataset(
            &mut conn,
            1,
            "one",
            false,
            DatasetStatus::Completed,
            date(1),
        );
        insert_dataset(
            &mut conn,
            2,
            "two",
            false,
            DatasetStatus::Completed,
            date(2),
        );
        insert_dataset(
            &mut conn,
            3,
            "three",
            false,
            DatasetStatus::Completed,
            date(3),
        );

        let first_page = do_list_datasets(
            &mut conn,
            &DatasetListQuery {
                limit: Some(2),
                offset: Some(0),
                ..DatasetListQuery::default()
            },
        )
        .expect("first page");
        let second_page = do_list_datasets(
            &mut conn,
            &DatasetListQuery {
                limit: Some(2),
                offset: Some(2),
                ..DatasetListQuery::default()
            },
        )
        .expect("second page");

        let first_ids: Vec<i32> = first_page.into_iter().map(|dataset| dataset.id).collect();
        let second_ids: Vec<i32> = second_page.into_iter().map(|dataset| dataset.id).collect();
        assert_eq!(first_ids, vec![3, 2]);
        assert_eq!(second_ids, vec![1]);
    }

    #[test]
    fn list_dataset_tags_returns_sorted_names() {
        let mut conn = setup_list_query_db();
        insert_tag(&mut conn, 1, "zeta");
        insert_tag(&mut conn, 2, "alpha");
        insert_tag(&mut conn, 3, "vision");

        let tags = do_list_dataset_tags(&mut conn).expect("list tags");

        assert_eq!(
            tags,
            vec![
                "alpha".to_string(),
                "vision".to_string(),
                "zeta".to_string()
            ]
        );
    }
}

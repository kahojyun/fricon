use std::{cell::RefCell, path::PathBuf, sync::Arc};

use arrow_array::{Int32Array, RecordBatch};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::{
    Connection, ExpressionMethods, RunQueryDsl, connection::SimpleConnection,
    sqlite::SqliteConnection,
};
use mockall::{Sequence, predicate::eq};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{
    MockDatasetEvents, MockDatasetRepo, MockDatasetStore, MockWriteSessionGuardOps, WriteSessions,
    create::create_dataset_with, do_list_dataset_tags, do_list_datasets,
};
use crate::{
    database::{self, DatasetStatus, SimpleUuid, schema},
    dataset_catalog::{DatasetListQuery, DatasetRecord, DatasetSortBy, SortDirection},
    dataset_ingest::{CreateDatasetRequest, CreateIngestEvent, CreateTerminal},
};

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
            let dataset = dataset_from_request(request, uid, dataset_id, DatasetStatus::Writing);
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
            Err(crate::dataset_catalog::DatasetCatalogError::NotFound {
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
    assert!(matches!(
        result,
        Err(crate::dataset_catalog::DatasetCatalogError::NotFound { .. })
    ));
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

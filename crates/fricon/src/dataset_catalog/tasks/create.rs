use diesel::{SqliteConnection, prelude::*};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use super::{DatasetEvents, DatasetRepo, DatasetStore, WriteSessionGuardOps, WriteSessions};
use crate::{
    database::{self, DatasetStatus, NewDataset, Pool, SimpleUuid, schema},
    dataset_catalog::{DatasetCatalogError, DatasetId, DatasetRecord},
    dataset_ingest::{
        CreateDatasetRequest, CreateIngestEvent, CreateTerminal, WriteSessionRegistry,
    },
    runtime::app::AppEvent,
    workspace::WorkspaceRoot,
};

#[instrument(
    skip(repo, store, events, write_sessions, events_rx, request),
    fields(dataset.name = %request.name, tags.count = request.tags.len())
)]
pub(super) fn create_dataset_with<R, S, E, W>(
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

pub(super) fn create_dataset_db_record(
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

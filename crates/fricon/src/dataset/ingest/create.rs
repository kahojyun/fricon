use std::path::PathBuf;

use diesel::{SqliteConnection, prelude::*};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    dataset::{
        events::AppEvent,
        ingest::{
            CreateDatasetRequest, CreateIngestEvent, CreateTerminal, IngestError,
            WriteSessionGuard, WriteSessionRegistry,
        },
        model::{DatasetId, DatasetRecord, DatasetStatus},
        sqlite::{self, NewDataset, Pool, SimpleUuid, schema},
        storage,
    },
    workspace::WorkspacePaths,
};

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetRepo {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(sqlite::Dataset, Vec<sqlite::Tag>), IngestError>;
    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), IngestError>;
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, IngestError>;
}

impl DatasetRepo for Pool {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(sqlite::Dataset, Vec<sqlite::Tag>), IngestError> {
        create_dataset_db_record(&mut *self.get()?, request, uid)
    }

    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), IngestError> {
        let mut conn = self.get()?;
        sqlite::Dataset::update_status(&mut conn, id, status)?;
        Ok(())
    }

    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, IngestError> {
        let mut conn = self.get()?;
        let dataset = match id {
            DatasetId::Id(dataset_id) => sqlite::Dataset::find_by_id(&mut conn, dataset_id)?,
            DatasetId::Uid(uid) => sqlite::Dataset::find_by_uid(&mut conn, uid)?,
        }
        .ok_or_else(|| IngestError::NotFound {
            id: match id {
                DatasetId::Id(value) => value.to_string(),
                DatasetId::Uid(value) => value.to_string(),
            },
        })?;
        let tags = dataset.load_tags(&mut conn)?;
        Ok(sqlite::dataset_record_from_models(dataset, tags))
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetStore {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, IngestError>;
}

impl DatasetStore for WorkspacePaths {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, IngestError> {
        let path = self.dataset_path_from_uid(uid);
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
    fn write(&mut self, batch: arrow_array::RecordBatch) -> Result<(), IngestError>;
    fn commit(self) -> Result<(), IngestError>;
    fn abort(self) -> Result<(), IngestError>;
}

impl WriteSessionGuardOps for WriteSessionGuard {
    fn write(&mut self, batch: arrow_array::RecordBatch) -> Result<(), IngestError> {
        self.write_batch(batch)
    }

    fn commit(self) -> Result<(), IngestError> {
        self.commit_session()
    }

    fn abort(self) -> Result<(), IngestError> {
        self.abort_session()
    }
}

pub(super) trait WriteSessions {
    type Guard: WriteSessionGuardOps;

    fn start_session(&self, id: i32, path: PathBuf, schema: arrow_schema::SchemaRef)
    -> Self::Guard;
}

impl WriteSessions for WriteSessionRegistry {
    type Guard = WriteSessionGuard;

    fn start_session(
        &self,
        id: i32,
        path: PathBuf,
        schema: arrow_schema::SchemaRef,
    ) -> Self::Guard {
        WriteSessionRegistry::start_session(self, id, path, schema)
    }
}

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
) -> Result<DatasetRecord, IngestError>
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

    events.send_dataset_created(AppEvent::DatasetCreated {
        id: dataset.id,
        name: request.name,
        description: request.description,
        favorite: dataset.favorite,
        tags: request.tags,
        status: dataset.status,
        created_at: dataset.created_at.and_utc(),
    });

    let dataset_record = sqlite::dataset_record_from_models(dataset, tags);

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

pub(super) fn create_dataset_db_record(
    conn: &mut SqliteConnection,
    request: &CreateDatasetRequest,
    uid: Uuid,
) -> Result<(sqlite::Dataset, Vec<sqlite::Tag>), IngestError> {
    conn.immediate_transaction(|conn| {
        let new_dataset = NewDataset {
            uid: SimpleUuid(uid),
            name: &request.name,
            description: &request.description,
            status: DatasetStatus::Writing,
        };

        let dataset = diesel::insert_into(schema::datasets::table)
            .values(new_dataset)
            .returning(sqlite::Dataset::as_returning())
            .get_result(conn)?;

        let tags = if request.tags.is_empty() {
            vec![]
        } else {
            let created_tags = sqlite::Tag::find_or_create_batch(conn, &request.tags)?;
            let tag_ids: Vec<i32> = created_tags.iter().map(|tag| tag.id).collect();
            sqlite::DatasetTag::create_associations(conn, dataset.id, &tag_ids)?;
            created_tags
        };

        Ok((dataset, tags))
    })
}

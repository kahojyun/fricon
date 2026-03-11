use std::path::PathBuf;

use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    dataset::{
        events::AppEvent,
        ingest::{
            CreateDatasetRequest, CreateIngestEvent, CreateTerminal, DatasetIngestRepository,
            IngestError, WriteSessionGuard, WriteSessionRegistry,
        },
        model::{DatasetId, DatasetRecord, DatasetStatus},
        storage,
    },
    workspace::WorkspacePaths,
};

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
    request: &CreateDatasetRequest,
    mut events_rx: mpsc::Receiver<CreateIngestEvent>,
) -> Result<DatasetRecord, IngestError>
where
    R: DatasetIngestRepository + ?Sized,
    S: DatasetStore,
    E: DatasetEvents,
    W: WriteSessions,
{
    let uid = Uuid::new_v4();
    let dataset_path = store.create_dataset_dir(uid)?;

    let dataset_record = repo.create_dataset_record(request, uid)?;
    info!(dataset.id = dataset_record.id, %uid, name = %request.name, "Dataset record created");

    events.send_dataset_created(AppEvent::DatasetCreated {
        id: dataset_record.id,
        name: dataset_record.metadata.name.clone(),
        description: dataset_record.metadata.description.clone(),
        favorite: dataset_record.metadata.favorite,
        tags: dataset_record.metadata.tags.clone(),
        status: dataset_record.metadata.status,
        created_at: dataset_record.metadata.created_at,
    });

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

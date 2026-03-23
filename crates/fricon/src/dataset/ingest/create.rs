//! Dataset ingest workflow helper.
//!
//! This module owns the stage -> stream -> finalize sequencing for dataset
//! creation. It creates the dataset directory and record, publishes the
//! `Created` event after the record exists, then drives the write session to a
//! terminal `Completed` or `Aborted` state.

use std::path::PathBuf;

use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::{
    dataset::{
        events::{DatasetEvent, DatasetEventPublisher},
        ingest::{
            CreateDatasetInput, CreateDatasetRequest, DatasetIngestRepository, IngestError,
            WriteSessionRegistry,
        },
        model::{DatasetId, DatasetRecord, DatasetStatus},
        storage,
    },
    workspace::WorkspacePaths,
};

/// Create a dataset record and drive its write session to a terminal status.
///
/// # Sequencing
///
/// 1. Create the dataset directory.
/// 2. Insert the dataset record in `Writing` status.
/// 3. Publish `DatasetEvent::Created` for the new record.
/// 4. Consume streamed inputs into a write session until `Finish`, `Abort`, or
///    end-of-stream.
/// 5. Commit to `Completed` or abort to `Aborted`, then re-read the final
///    stored record.
///
/// End-of-stream is treated as `Abort`. If session commit fails after the
/// record exists, the workflow best-effort marks the dataset `Aborted`
/// before returning the error.
#[instrument(
    skip(repo, paths, events, write_sessions, next_input, request),
    fields(dataset.name = %request.name, tags.count = request.tags.len())
)]
pub(super) fn create_dataset_with<R, E, F>(
    repo: &R,
    paths: &WorkspacePaths,
    events: &E,
    write_sessions: &WriteSessionRegistry,
    request: &CreateDatasetRequest,
    mut next_input: F,
) -> Result<DatasetRecord, IngestError>
where
    R: DatasetIngestRepository + ?Sized,
    E: DatasetEventPublisher,
    F: FnMut() -> Option<CreateDatasetInput>,
{
    let uid = Uuid::new_v4();
    let dataset_path = create_dataset_dir(paths, uid)?;

    let dataset_record = repo.create_dataset_record(request, uid)?;
    info!(dataset.id = dataset_record.id, %uid, name = %request.name, "Dataset record created");

    events.publish(DatasetEvent::Created(dataset_record.clone()));

    let mut session = None;
    let terminal = loop {
        let Some(event) = next_input() else {
            break CreateDatasetInput::Abort;
        };

        match event {
            CreateDatasetInput::Batch(batch) => {
                let session_ref = session.get_or_insert_with(|| {
                    write_sessions.start_session(
                        dataset_record.id,
                        dataset_path.clone(),
                        batch.schema(),
                    )
                });
                if let Err(error) = session_ref.write_batch(batch) {
                    debug!(error = %error, "Failed to write batch into dataset session");
                    break CreateDatasetInput::Abort;
                }
            }
            CreateDatasetInput::Finish | CreateDatasetInput::Abort => break event,
        }
    };

    match terminal {
        CreateDatasetInput::Finish => {
            if let Some(session) = session.take()
                && let Err(error) = session.commit_session()
            {
                debug!(error = %error, "Failed to commit dataset session, switching to aborted");
                let _ = repo.update_status(dataset_record.id, DatasetStatus::Aborted);
                return Err(error);
            }
            repo.update_status(dataset_record.id, DatasetStatus::Completed)?;
            info!(dataset.id = dataset_record.id, %uid, "Dataset write completed");
            repo.get_dataset(DatasetId::Id(dataset_record.id))
        }
        CreateDatasetInput::Abort => {
            if let Some(session) = session.take()
                && let Err(error) = session.abort_session()
            {
                debug!(error = %error, "Failed to abort dataset session, keeping aborted status");
            }
            repo.update_status(dataset_record.id, DatasetStatus::Aborted)?;
            info!(dataset.id = dataset_record.id, "Dataset write aborted");
            repo.get_dataset(DatasetId::Id(dataset_record.id))
        }
        CreateDatasetInput::Batch(_) => unreachable!("batch cannot terminate dataset creation"),
    }
}

/// Create the dataset directory before the ingest record is inserted.
///
/// The workflow stages filesystem creation first so later write-session work
/// has a concrete destination. Failures abort ingest before any database
/// record is created.
fn create_dataset_dir(paths: &WorkspacePaths, uid: Uuid) -> Result<PathBuf, IngestError> {
    let path = paths.dataset_path_from_uid(uid);
    storage::create_dataset(&path)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, fs, sync::Mutex};

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_schema::{DataType, Field, Schema};
    use chrono::Utc;
    use tempfile::TempDir;

    use super::*;
    use crate::{
        dataset::{
            events::DatasetEvent,
            model::{DatasetMetadata, DatasetStatus},
        },
        workspace::WorkspaceRoot,
    };

    struct FakeRepo {
        state: Mutex<FakeRepoState>,
    }

    struct FakeRepoState {
        record: DatasetRecord,
        created_uid: Option<Uuid>,
        updated_statuses: Vec<DatasetStatus>,
    }

    impl FakeRepo {
        fn new() -> Self {
            Self {
                state: Mutex::new(FakeRepoState {
                    record: DatasetRecord {
                        id: 7,
                        metadata: DatasetMetadata {
                            uid: Uuid::nil(),
                            name: "dataset".to_string(),
                            description: "desc".to_string(),
                            favorite: false,
                            status: DatasetStatus::Writing,
                            created_at: Utc::now(),
                            trashed_at: None,
                            deleted_at: None,
                            tags: vec!["tag".to_string()],
                        },
                    },
                    created_uid: None,
                    updated_statuses: Vec::new(),
                }),
            }
        }

        fn created_uid(&self) -> Uuid {
            self.state
                .lock()
                .expect("repo state")
                .created_uid
                .expect("uid captured")
        }

        fn updated_statuses(&self) -> Vec<DatasetStatus> {
            self.state
                .lock()
                .expect("repo state")
                .updated_statuses
                .clone()
        }
    }

    impl DatasetIngestRepository for FakeRepo {
        fn create_dataset_record(
            &self,
            request: &CreateDatasetRequest,
            uid: Uuid,
        ) -> Result<DatasetRecord, IngestError> {
            let mut state = self.state.lock().expect("repo state");
            state.created_uid = Some(uid);
            state.record.metadata.uid = uid;
            state.record.metadata.name = request.name.clone();
            state.record.metadata.description = request.description.clone();
            state.record.metadata.tags = request.tags.clone();
            state.record.metadata.status = DatasetStatus::Writing;
            Ok(state.record.clone())
        }

        fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), IngestError> {
            let mut state = self.state.lock().expect("repo state");
            assert_eq!(id, state.record.id);
            state.record.metadata.status = status;
            state.updated_statuses.push(status);
            Ok(())
        }

        fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, IngestError> {
            let state = self.state.lock().expect("repo state");
            assert!(matches!(id, DatasetId::Id(7)));
            Ok(state.record.clone())
        }
    }

    #[derive(Default)]
    struct CollectEvents {
        events: Mutex<Vec<DatasetEvent>>,
    }

    impl CollectEvents {
        fn snapshot(&self) -> Vec<DatasetEvent> {
            self.events.lock().expect("events").clone()
        }
    }

    impl DatasetEventPublisher for CollectEvents {
        fn publish(&self, event: DatasetEvent) {
            self.events.lock().expect("events").push(event);
        }
    }

    fn create_request() -> CreateDatasetRequest {
        CreateDatasetRequest {
            name: "dataset".to_string(),
            description: "desc".to_string(),
            tags: vec!["tag".to_string()],
        }
    }

    fn one_col_batch() -> RecordBatch {
        let schema =
            std::sync::Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        RecordBatch::try_new(schema, vec![std::sync::Arc::new(Int32Array::from(vec![1]))])
            .expect("batch")
    }

    #[test]
    fn finish_without_batches_marks_dataset_completed() {
        let temp_dir = TempDir::new().expect("temp dir");
        let workspace = WorkspaceRoot::create_new(temp_dir.path()).expect("workspace");
        let paths = workspace.paths().clone();
        let repo = FakeRepo::new();
        let events = CollectEvents::default();
        let write_sessions = WriteSessionRegistry::new();
        let mut inputs = VecDeque::from(vec![CreateDatasetInput::Finish]);

        let record = create_dataset_with(
            &repo,
            &paths,
            &events,
            &write_sessions,
            &create_request(),
            || inputs.pop_front(),
        )
        .expect("create dataset");

        assert_eq!(record.metadata.status, DatasetStatus::Completed);
        assert_eq!(repo.updated_statuses(), vec![DatasetStatus::Completed]);
        assert!(matches!(
            events.snapshot().as_slice(),
            [DatasetEvent::Created(created)] if created.id == record.id
        ));
    }

    #[test]
    fn abort_marks_dataset_aborted() {
        let temp_dir = TempDir::new().expect("temp dir");
        let workspace = WorkspaceRoot::create_new(temp_dir.path()).expect("workspace");
        let paths = workspace.paths().clone();
        let repo = FakeRepo::new();
        let events = CollectEvents::default();
        let write_sessions = WriteSessionRegistry::new();
        let mut inputs = VecDeque::from(vec![CreateDatasetInput::Abort]);

        let record = create_dataset_with(
            &repo,
            &paths,
            &events,
            &write_sessions,
            &create_request(),
            || inputs.pop_front(),
        )
        .expect("abort dataset");

        assert_eq!(record.metadata.status, DatasetStatus::Aborted);
        assert_eq!(repo.updated_statuses(), vec![DatasetStatus::Aborted]);
        assert!(matches!(
            events.snapshot().as_slice(),
            [DatasetEvent::Created(created)] if created.id == record.id
        ));
    }

    #[test]
    fn commit_failure_marks_dataset_aborted_and_returns_error() {
        let temp_dir = TempDir::new().expect("temp dir");
        let workspace = WorkspaceRoot::create_new(temp_dir.path()).expect("workspace");
        let paths = workspace.paths().clone();
        let repo = FakeRepo::new();
        let events = CollectEvents::default();
        let write_sessions = WriteSessionRegistry::new();
        let mut sent_batch = false;
        let mut removed_dir = false;

        let error = create_dataset_with(
            &repo,
            &paths,
            &events,
            &write_sessions,
            &create_request(),
            || {
                if !sent_batch {
                    sent_batch = true;
                    Some(CreateDatasetInput::Batch(one_col_batch()))
                } else if !removed_dir {
                    removed_dir = true;
                    let dataset_path = paths.dataset_path_from_uid(repo.created_uid());
                    fs::remove_dir_all(dataset_path).expect("remove dataset dir");
                    Some(CreateDatasetInput::Finish)
                } else {
                    None
                }
            },
        )
        .expect_err("commit should fail");

        assert_eq!(repo.updated_statuses(), vec![DatasetStatus::Aborted]);
        assert!(matches!(
            events.snapshot().as_slice(),
            [DatasetEvent::Created(created)] if created.id == 7
        ));
        assert!(matches!(error, IngestError::DatasetFs(_)));
    }
}

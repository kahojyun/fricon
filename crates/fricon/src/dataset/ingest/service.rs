use tokio::{sync::broadcast, task::JoinHandle};
use tokio_util::task::TaskTracker;
use tracing::error;

use crate::{
    dataset::{
        events::AppEvent,
        ingest::{
            CreateDatasetRequest, CreateIngestEvent, IngestError, WriteSessionRegistry, create,
        },
        model::DatasetRecord,
        sqlite::Pool,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub struct DatasetIngestService {
    database: Pool,
    paths: WorkspacePaths,
    event_sender: broadcast::Sender<AppEvent>,
    write_sessions: WriteSessionRegistry,
    tracker: TaskTracker,
}

impl DatasetIngestService {
    #[must_use]
    pub(crate) fn new(
        database: Pool,
        paths: WorkspacePaths,
        event_sender: broadcast::Sender<AppEvent>,
        write_sessions: WriteSessionRegistry,
        tracker: TaskTracker,
    ) -> Self {
        Self {
            database,
            paths,
            event_sender,
            write_sessions,
            tracker,
        }
    }

    fn spawn_blocking<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.tracker.spawn_blocking(f)
    }

    pub async fn create_dataset(
        &self,
        request: CreateDatasetRequest,
        events_rx: tokio::sync::mpsc::Receiver<CreateIngestEvent>,
    ) -> Result<DatasetRecord, IngestError> {
        let database = self.database.clone();
        let paths = self.paths.clone();
        let event_sender = self.event_sender.clone();
        let write_sessions = self.write_sessions.clone();
        let dataset_name = request.name.clone();

        self.spawn_blocking(move || {
            create::create_dataset_with(
                &database,
                &paths,
                &event_sender,
                &write_sessions,
                request,
                events_rx,
            )
            .inspect_err(|e| {
                error!(error = %e, dataset.name = %dataset_name, "Dataset creation failed");
            })
        })
        .await?
    }
}

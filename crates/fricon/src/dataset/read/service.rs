use tokio::task::JoinHandle;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::{
    dataset::{
        ingest::WriteSessionRegistry,
        model::DatasetId,
        read::{DatasetReader, ReadError, access},
        sqlite::Pool,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub struct DatasetReadService {
    database: Pool,
    paths: WorkspacePaths,
    write_sessions: WriteSessionRegistry,
    tracker: TaskTracker,
}

impl DatasetReadService {
    #[must_use]
    pub(crate) fn new(
        database: Pool,
        paths: WorkspacePaths,
        write_sessions: WriteSessionRegistry,
        tracker: TaskTracker,
    ) -> Self {
        Self {
            database,
            paths,
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

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub async fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, ReadError> {
        let database = self.database.clone();
        let paths = self.paths.clone();
        let write_sessions = self.write_sessions.clone();
        self.spawn_blocking(move || {
            access::get_dataset_reader(&database, &paths, &write_sessions, id)
        })
        .await?
    }
}

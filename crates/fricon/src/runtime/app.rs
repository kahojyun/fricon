use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::broadcast, task::JoinHandle, time};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument};

use crate::{
    database::{self, DatasetStatus, Pool},
    dataset_catalog::DatasetCatalogService,
    dataset_ingest::{DatasetIngestService, WriteSessionRegistry},
    dataset_read::DatasetReadService,
    runtime::server,
    workspace::{WorkspacePaths, WorkspaceRoot},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppEvent {
    DatasetCreated {
        id: i32,
        name: String,
        description: String,
        favorite: bool,
        tags: Vec<String>,
        status: DatasetStatus,
        created_at: DateTime<Utc>,
    },
    DatasetUpdated {
        id: i32,
        name: String,
        description: String,
        favorite: bool,
        tags: Vec<String>,
        status: DatasetStatus,
        created_at: DateTime<Utc>,
    },
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppState has been dropped")]
    StateDropped,
}

pub struct AppState {
    pub(crate) root: WorkspaceRoot,
    pub(crate) database: Pool,
    pub(crate) shutdown_token: CancellationToken,
    pub(crate) tracker: TaskTracker,
    pub(crate) event_sender: broadcast::Sender<AppEvent>,
    pub(crate) write_sessions: WriteSessionRegistry,
}

impl AppState {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    fn new(root: WorkspaceRoot) -> Result<Arc<Self>> {
        let db_path = root.paths().database_file();
        let backup_path = root
            .paths()
            .database_backup_file(Local::now().naive_local());
        info!(path = ?root.paths().root(), "Initializing app state");
        let database = database::connect(db_path, backup_path)?;

        // Clean up any datasets that were left in 'writing' state
        if let Err(e) = database::cleanup_writing_datasets(&database) {
            error!(error = %e, "Failed to cleanup writing datasets");
        }

        let shutdown_token = CancellationToken::new();
        let tracker = TaskTracker::new();
        let (event_sender, _) = broadcast::channel(1000);

        let write_sessions = WriteSessionRegistry::new();
        Ok(Arc::new(Self {
            root,
            database,
            shutdown_token,
            tracker,
            event_sender,
            write_sessions,
        }))
    }
}

#[derive(Clone)]
pub struct AppHandle {
    state: Weak<AppState>,
}

impl AppHandle {
    fn new(state: Weak<AppState>) -> Self {
        Self { state }
    }

    fn state(&self) -> Result<Arc<AppState>, AppError> {
        self.state.upgrade().ok_or(AppError::StateDropped)
    }

    pub fn paths(&self) -> Result<WorkspacePaths, AppError> {
        Ok(self.state()?.root.paths().clone())
    }

    pub fn subscribe_to_events(&self) -> Result<broadcast::Receiver<AppEvent>, AppError> {
        Ok(self.state()?.event_sender.subscribe())
    }

    #[must_use]
    pub fn dataset_catalog(&self) -> DatasetCatalogService {
        DatasetCatalogService::new(self.clone())
    }

    #[must_use]
    pub fn dataset_ingest(&self) -> DatasetIngestService {
        DatasetIngestService::new(self.clone())
    }

    #[must_use]
    pub fn dataset_read(&self) -> DatasetReadService {
        DatasetReadService::new(self.clone())
    }

    pub fn spawn<F, Fut, T>(&self, f: F) -> Result<JoinHandle<T>, AppError>
    where
        F: FnOnce(Arc<AppState>) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let state = self.state()?;
        let tracker = state.tracker.clone();
        Ok(tracker.spawn(f(state)))
    }

    pub fn spawn_blocking<F, T>(&self, f: F) -> Result<JoinHandle<T>, AppError>
    where
        F: FnOnce(Arc<AppState>) -> T + Send + 'static,
        T: Send + 'static,
    {
        let state = self.state()?;
        let tracker = state.tracker.clone();
        Ok(tracker.spawn_blocking(move || f(state)))
    }
}

pub struct AppManager {
    state: Arc<AppState>,
    handle: AppHandle,
}

impl AppManager {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    pub fn serve(root: WorkspaceRoot) -> Result<Self> {
        let state = AppState::new(root)?;
        let handle = AppHandle::new(Arc::downgrade(&state));

        let ipc_file = handle.paths()?.ipc_file();
        server::start(
            ipc_file,
            &handle,
            &state.tracker,
            state.shutdown_token.clone(),
        )?;

        info!(path = ?handle.paths()?.root(), "App server started");
        Ok(Self { state, handle })
    }

    /// Creates a new `AppManager` with workspace creation.
    pub fn serve_with_path(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::create(path)?;
        Self::serve(root)
    }

    pub async fn shutdown(self) {
        self.shutdown_with_timeout(Duration::from_secs(10)).await;
    }

    pub async fn shutdown_with_timeout(self, timeout: Duration) {
        info!(timeout_ms = timeout.as_millis(), "Starting server shutdown");

        let result = time::timeout(timeout, async {
            self.state.shutdown_token.cancel();
            self.state.tracker.close();
            self.state.tracker.wait().await;
        })
        .await;

        match result {
            Ok(()) => {
                info!("Server shutdown completed successfully");
            }
            Err(_) => {
                error!(
                    timeout_ms = timeout.as_millis(),
                    "Server shutdown timed out; some resources may not have been cleaned up \
                     properly"
                );
            }
        }
    }

    #[must_use]
    pub fn handle(&self) -> &AppHandle {
        &self.handle
    }
}

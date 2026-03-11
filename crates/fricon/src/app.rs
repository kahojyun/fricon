mod server;

use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use anyhow::Result;
use chrono::Local;
use thiserror::Error;
use tokio::{sync::broadcast, time};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument};

pub use crate::dataset::events::AppEvent;
use crate::{
    dataset::{
        DatasetCatalogService, DatasetIngestService, DatasetReadService,
        ingest::WriteSessionRegistry,
        sqlite::{self, Pool},
    },
    workspace::{WorkspacePaths, WorkspaceRoot},
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppState has been dropped")]
    StateDropped,
}

pub struct AppState {
    root: WorkspaceRoot,
    dataset_catalog: DatasetCatalogService,
    dataset_ingest: DatasetIngestService,
    dataset_read: DatasetReadService,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
    event_sender: broadcast::Sender<AppEvent>,
}

impl AppState {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    fn new(root: WorkspaceRoot) -> Result<Arc<Self>> {
        let database = init_database(&root)?;
        let shutdown_token = CancellationToken::new();
        let tracker = TaskTracker::new();
        let (event_sender, _) = broadcast::channel(1000);
        let write_sessions = WriteSessionRegistry::new();

        let dataset_catalog = DatasetCatalogService::new(
            database.clone(),
            root.paths().clone(),
            event_sender.clone(),
            tracker.clone(),
        );
        let dataset_ingest = DatasetIngestService::new(
            database.clone(),
            root.paths().clone(),
            event_sender.clone(),
            write_sessions.clone(),
            tracker.clone(),
        );
        let dataset_read = DatasetReadService::new(
            database,
            root.paths().clone(),
            write_sessions,
            tracker.clone(),
        );

        Ok(Arc::new(Self {
            root,
            dataset_catalog,
            dataset_ingest,
            dataset_read,
            shutdown_token,
            tracker,
            event_sender,
        }))
    }
}

fn init_database(root: &WorkspaceRoot) -> Result<Pool> {
    let db_path = root.paths().database_file();
    let backup_path = root
        .paths()
        .database_backup_file(Local::now().naive_local());
    info!(path = ?root.paths().root(), "Initializing app state");
    let database = sqlite::connect(db_path, backup_path)?;

    if let Err(e) = sqlite::cleanup_writing_datasets(&database) {
        error!(error = %e, "Failed to cleanup writing datasets");
    }

    Ok(database)
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
        self.state()
            .expect("AppState should be alive while AppHandle is used")
            .dataset_catalog
            .clone()
    }

    #[must_use]
    pub fn dataset_ingest(&self) -> DatasetIngestService {
        self.state()
            .expect("AppState should be alive while AppHandle is used")
            .dataset_ingest
            .clone()
    }

    #[must_use]
    pub fn dataset_read(&self) -> DatasetReadService {
        self.state()
            .expect("AppState should be alive while AppHandle is used")
            .dataset_read
            .clone()
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
            Ok(()) => info!("Server shutdown completed successfully"),
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

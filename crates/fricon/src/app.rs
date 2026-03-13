mod server;

use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use anyhow::{Result, bail};
use chrono::Local;
use thiserror::Error;
use tokio::{runtime::Handle, sync::broadcast, time};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument};

pub use crate::dataset::events::AppEvent;
use crate::{
    database::{core, dataset as database_dataset},
    dataset::{
        DatasetCatalogService, DatasetIngestService, DatasetReadService,
        ingest::WriteSessionRegistry,
    },
    workspace::{WorkspacePaths, WorkspaceRoot},
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppState has been dropped")]
    StateDropped,
    #[error("App event was not delivered to any subscribers")]
    EventUndelivered,
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
        let dataset_repository = Arc::new(database_dataset::DatasetRepository::new(database));

        let dataset_catalog = DatasetCatalogService::new(
            dataset_repository.clone(),
            root.paths().clone(),
            event_sender.clone(),
            tracker.clone(),
        );
        let dataset_ingest = DatasetIngestService::new(
            dataset_repository.clone(),
            root.paths().clone(),
            event_sender.clone(),
            write_sessions.clone(),
            tracker.clone(),
        );
        let dataset_read = DatasetReadService::new(
            dataset_repository,
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

fn init_database(root: &WorkspaceRoot) -> Result<core::Pool> {
    let db_path = root.paths().database_file();
    let backup_path = root
        .paths()
        .database_backup_file(Local::now().naive_local());
    info!(path = ?root.paths().root(), "Initializing app state");
    let database = core::connect(db_path, backup_path)?;

    if let Err(e) = database_dataset::cleanup_writing_datasets(&database) {
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

    pub fn send_event(&self, event: AppEvent) -> Result<usize, AppError> {
        self.state()?
            .event_sender
            .send(event)
            .map_err(|_| AppError::EventUndelivered)
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
    started: bool,
}

impl AppManager {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    pub fn new(root: WorkspaceRoot) -> Result<Self> {
        let state = AppState::new(root)?;
        let handle = AppHandle::new(Arc::downgrade(&state));
        Ok(Self {
            state,
            handle,
            started: false,
        })
    }

    pub fn new_with_path(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::create(path)?;
        Self::new(root)
    }

    #[instrument(skip(self, runtime), fields(workspace.path = ?self.handle.paths()?.root()))]
    pub fn start(mut self, runtime: &Handle) -> Result<Self> {
        if self.started {
            bail!("App server is already started");
        }

        let ipc_file = self.handle.paths()?.ipc_file();
        server::start(
            ipc_file,
            &self.handle,
            &self.state.tracker,
            self.state.shutdown_token.clone(),
            runtime,
        )?;

        self.started = true;
        info!(path = ?self.handle.paths()?.root(), "App server started");
        Ok(self)
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

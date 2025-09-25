use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use anyhow::Result;
use chrono::Local;
use deadpool_diesel::sqlite::Pool;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{sync::broadcast, task::JoinHandle, time::sleep};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use crate::{
    database,
    dataset_manager::DatasetManager,
    server,
    workspace::{WorkspacePaths, WorkspaceRoot},
    write_registry::WriteSessionRegistry,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppEvent {
    DatasetCreated {
        id: i32,
        uuid: String,
        name: String,
        description: String,
        tags: Vec<String>,
    },
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppState has been dropped")]
    StateDropped,
}

pub struct AppState {
    pub root: WorkspaceRoot,
    pub database: Pool,
    pub shutdown_token: CancellationToken,
    pub tracker: TaskTracker,
    pub event_sender: broadcast::Sender<AppEvent>,
    pub write_sessions: WriteSessionRegistry,
}

impl AppState {
    async fn new(root: WorkspaceRoot) -> Result<Arc<Self>> {
        let db_path = root.paths().database_file();
        let backup_path = root
            .paths()
            .database_backup_file(Local::now().naive_local());
        let database = database::connect(db_path, backup_path).await?;
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
    pub(crate) fn new(state: Weak<AppState>) -> Self {
        Self { state }
    }

    pub(crate) fn get_state(&self) -> Result<Arc<AppState>, AppError> {
        self.state.upgrade().ok_or(AppError::StateDropped)
    }

    pub fn paths(&self) -> Result<WorkspacePaths, AppError> {
        Ok(self.get_state()?.root.paths().clone())
    }

    pub fn subscribe_to_events(&self) -> Result<broadcast::Receiver<AppEvent>, AppError> {
        Ok(self.get_state()?.event_sender.subscribe())
    }

    #[must_use]
    pub fn dataset_manager(&self) -> DatasetManager {
        DatasetManager::new(self.clone())
    }

    pub fn spawn<F, Fut, T>(&self, f: F) -> Result<JoinHandle<T>, AppError>
    where
        F: FnOnce(Arc<AppState>) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let state = self.get_state()?;
        let tracker = state.tracker.clone();
        Ok(tracker.spawn(f(state)))
    }

    pub fn spawn_blocking<F, T>(&self, f: F) -> Result<JoinHandle<T>, AppError>
    where
        F: FnOnce(Arc<AppState>) -> T + Send + 'static,
        T: Send + 'static,
    {
        let state = self.get_state()?;
        let tracker = state.tracker.clone();
        Ok(tracker.spawn_blocking(move || f(state)))
    }
}

pub struct AppManager {
    state: Arc<AppState>,
    handle: AppHandle,
}

impl AppManager {
    pub async fn serve(root: WorkspaceRoot) -> Result<Self> {
        let state = AppState::new(root).await?;
        let handle = AppHandle::new(Arc::downgrade(&state));

        let ipc_file = handle.paths()?.ipc_file();
        server::start(
            ipc_file,
            handle.clone(),
            &state.tracker,
            state.shutdown_token.clone(),
        )?;

        Ok(Self { state, handle })
    }

    /// Creates a new `AppManager` with workspace creation.
    pub async fn serve_with_path(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::create(path)?;
        Self::serve(root).await
    }

    pub async fn shutdown(self) {
        self.shutdown_with_timeout(Duration::from_secs(10)).await
    }

    pub async fn shutdown_with_timeout(self, timeout: Duration) {
        info!("Starting server shutdown with timeout: {:?}", timeout);

        let result = tokio::time::timeout(timeout, async {
            self.state.shutdown_token.cancel();
            self.state.tracker.close();
            self.state.tracker.wait().await;
            drop(self.state);
            // Wait for sqlite connection release
            sleep(Duration::from_millis(200)).await;
        })
        .await;

        match result {
            Ok(_) => {
                info!("Server shutdown completed successfully");
            }
            Err(_) => {
                error!(
                    "Server shutdown timed out after {:?}. Some resources may not have been \
                     cleaned up properly.",
                    timeout
                );
            }
        }
    }

    #[must_use]
    pub fn handle(&self) -> &AppHandle {
        &self.handle
    }
}

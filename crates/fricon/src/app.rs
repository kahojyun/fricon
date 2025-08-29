use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use chrono::Local;
use deadpool_diesel::sqlite::Pool;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::info;

use crate::{database, dataset_manager::DatasetManager, server, workspace::WorkspaceRoot};

pub async fn init(path: impl Into<PathBuf>) -> Result<()> {
    let path = path.into();
    info!("Initialize workspace: {}", path.display());
    let root = WorkspaceRoot::init(path)?;
    let db_path = root.paths().database_file();
    let backup_path = root
        .paths()
        .database_backup_file(Local::now().naive_local());
    database::connect(db_path, backup_path).await?;
    Ok(())
}

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

#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    root: WorkspaceRoot,
    database: Pool,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
    event_sender: broadcast::Sender<AppEvent>,
}

impl AppState {
    async fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let root = WorkspaceRoot::open(path)?;
        let db_path = root.paths().database_file();
        let backup_path = root
            .paths()
            .database_backup_file(Local::now().naive_local());
        let database = database::connect(db_path, backup_path).await?;
        let shutdown_token = CancellationToken::new();
        let tracker = TaskTracker::new();
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            inner: Arc::new(AppStateInner {
                root,
                database,
                shutdown_token,
                tracker,
                event_sender,
            }),
        })
    }

    #[must_use]
    fn root(&self) -> &WorkspaceRoot {
        &self.inner.root
    }

    #[must_use]
    fn database(&self) -> &Pool {
        &self.inner.database
    }

    #[must_use]
    fn tracker(&self) -> &TaskTracker {
        &self.inner.tracker
    }

    #[must_use]
    fn shutdown_token(&self) -> &CancellationToken {
        &self.inner.shutdown_token
    }

    #[must_use]
    fn event_sender(&self) -> &broadcast::Sender<AppEvent> {
        &self.inner.event_sender
    }

    fn subscribe_to_events(&self) -> broadcast::Receiver<AppEvent> {
        self.inner.event_sender.subscribe()
    }
}

#[derive(Clone)]
pub struct AppHandle {
    state: AppState,
}

impl AppHandle {
    fn new(state: AppState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn root(&self) -> &WorkspaceRoot {
        self.state.root()
    }

    #[must_use]
    pub fn database(&self) -> &Pool {
        self.state.database()
    }

    #[must_use]
    pub fn tracker(&self) -> &TaskTracker {
        self.state.tracker()
    }

    #[must_use]
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<AppEvent> {
        self.state.subscribe_to_events()
    }

    #[must_use]
    pub fn dataset_manager(&self) -> DatasetManager {
        DatasetManager::new(self.clone())
    }

    pub fn send_event(&self, event: AppEvent) {
        let _ = self.state.event_sender().send(event);
    }
}

pub struct AppManager {
    state: AppState,
    handle: AppHandle,
}

impl AppManager {
    pub async fn serve(path: impl Into<PathBuf>) -> Result<Self> {
        let state = AppState::new(path).await?;
        let handle = AppHandle::new(state.clone());

        state
            .tracker()
            .spawn(server::run(handle.clone(), state.shutdown_token().clone()));

        Ok(Self { state, handle })
    }

    pub async fn shutdown(&self) {
        self.state.shutdown_token().cancel();
        self.state.tracker().close();
        self.state.tracker().wait().await;
    }

    #[must_use]
    pub fn handle(&self) -> &AppHandle {
        &self.handle
    }
}

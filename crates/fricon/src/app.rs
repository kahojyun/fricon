mod server;

use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use anyhow::{anyhow, bail};
use chrono::Local;
use thiserror::Error;
use tokio::{
    runtime::Handle,
    sync::{broadcast, mpsc},
    time,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument};

use crate::{
    database::{core, dataset as database_dataset},
    dataset::{
        CreateDatasetInput, CreateDatasetRequest, DatasetEvent, DatasetId, DatasetListQuery,
        DatasetReader, DatasetRecord, DatasetUpdate,
        catalog::{CatalogError, DatasetCatalogService},
        events::DatasetEventPublisher,
        ingest::{DatasetIngestService, IngestError, WriteSessionRegistry},
        read::{DatasetReadService, ReadError},
    },
    workspace::{WorkspacePaths, WorkspaceRoot},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCommand {
    ShowUi,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppState has been dropped")]
    StateDropped,
    #[error("UI command was not delivered to any subscribers")]
    UiCommandUndelivered,
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("Subscription lagged by {skipped} messages")]
    Lagged { skipped: u64 },
    #[error("Subscription closed")]
    Closed,
}

pub struct DatasetEventSubscription {
    inner: broadcast::Receiver<DatasetEvent>,
}

impl DatasetEventSubscription {
    pub async fn recv(&mut self) -> std::result::Result<DatasetEvent, SubscriptionError> {
        self.inner
            .recv()
            .await
            .map_err(|error| map_recv_error(&error))
    }
}

pub struct UiCommandSubscription {
    inner: broadcast::Receiver<UiCommand>,
}

impl UiCommandSubscription {
    pub async fn recv(&mut self) -> std::result::Result<UiCommand, SubscriptionError> {
        self.inner
            .recv()
            .await
            .map_err(|error| map_recv_error(&error))
    }
}

#[derive(Clone)]
struct BroadcastDatasetEvents {
    sender: broadcast::Sender<DatasetEvent>,
}

impl DatasetEventPublisher for BroadcastDatasetEvents {
    fn publish(&self, event: DatasetEvent) {
        let _ = self.sender.send(event);
    }
}

pub struct AppState {
    root: WorkspaceRoot,
    dataset_catalog: DatasetCatalogService,
    dataset_ingest: DatasetIngestService,
    dataset_read: DatasetReadService,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
    dataset_event_sender: broadcast::Sender<DatasetEvent>,
    ui_command_sender: broadcast::Sender<UiCommand>,
}

impl AppState {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    fn new(root: WorkspaceRoot) -> anyhow::Result<Arc<Self>> {
        let database = init_database(&root)?;
        let shutdown_token = CancellationToken::new();
        let tracker = TaskTracker::new();
        let (dataset_event_sender, _) = broadcast::channel(1000);
        let (ui_command_sender, _) = broadcast::channel(64);
        let write_sessions = WriteSessionRegistry::new();
        let dataset_repository = Arc::new(database_dataset::DatasetRepository::new(database));

        let dataset_catalog =
            DatasetCatalogService::new(dataset_repository.clone(), root.paths().clone());
        let dataset_ingest = DatasetIngestService::new(
            dataset_repository.clone(),
            root.paths().clone(),
            write_sessions.clone(),
        );
        let dataset_read =
            DatasetReadService::new(dataset_repository, root.paths().clone(), write_sessions);

        if let Err(error) = dataset_catalog.reconcile_deleted_datasets() {
            error!(error = %error, "Failed to reconcile deleted datasets");
        }
        if let Err(error) = dataset_catalog.garbage_collect_deleted_datasets() {
            error!(error = %error, "Failed to garbage collect deleted dataset payloads");
        }

        Ok(Arc::new(Self {
            root,
            dataset_catalog,
            dataset_ingest,
            dataset_read,
            shutdown_token,
            tracker,
            dataset_event_sender,
            ui_command_sender,
        }))
    }
}

fn init_database(root: &WorkspaceRoot) -> anyhow::Result<core::Pool> {
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

fn map_recv_error(error: &broadcast::error::RecvError) -> SubscriptionError {
    match error {
        broadcast::error::RecvError::Lagged(skipped) => {
            SubscriptionError::Lagged { skipped: *skipped }
        }
        broadcast::error::RecvError::Closed => SubscriptionError::Closed,
    }
}

fn catalog_state_dropped() -> CatalogError {
    anyhow!("app state has been dropped").into()
}

fn ingest_state_dropped() -> IngestError {
    anyhow!("app state has been dropped").into()
}

fn read_state_dropped() -> ReadError {
    anyhow!("app state has been dropped").into()
}

fn catalog_join_error(error: tokio::task::JoinError, context: &'static str) -> CatalogError {
    anyhow::Error::new(error).context(context).into()
}

fn ingest_join_error(error: tokio::task::JoinError, context: &'static str) -> IngestError {
    anyhow::Error::new(error).context(context).into()
}

fn read_join_error(error: tokio::task::JoinError, context: &'static str) -> ReadError {
    anyhow::Error::new(error).context(context).into()
}

#[derive(Clone)]
pub struct AppHandle {
    state: Weak<AppState>,
}

impl AppHandle {
    fn new(state: Weak<AppState>) -> Self {
        Self { state }
    }

    fn state(&self) -> std::result::Result<Arc<AppState>, AppError> {
        self.state.upgrade().ok_or(AppError::StateDropped)
    }

    pub fn paths(&self) -> std::result::Result<WorkspacePaths, AppError> {
        Ok(self.state()?.root.paths().clone())
    }

    pub fn subscribe_dataset_events(
        &self,
    ) -> std::result::Result<DatasetEventSubscription, AppError> {
        Ok(DatasetEventSubscription {
            inner: self.state()?.dataset_event_sender.subscribe(),
        })
    }

    pub fn subscribe_ui_commands(&self) -> std::result::Result<UiCommandSubscription, AppError> {
        Ok(UiCommandSubscription {
            inner: self.state()?.ui_command_sender.subscribe(),
        })
    }

    pub fn request_show_ui(&self) -> std::result::Result<(), AppError> {
        self.state()?
            .ui_command_sender
            .send(UiCommand::ShowUi)
            .map(|_| ())
            .map_err(|_| AppError::UiCommandUndelivered)
    }

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.get_dataset(id))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset get task"))?
    }

    pub async fn list_datasets(
        &self,
        query: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.list_datasets(query))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset list task"))?
    }

    pub async fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.list_dataset_tags())
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset tag list task"))?
    }

    pub async fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.update_dataset(id, update, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset update task"))?
    }

    pub async fn add_dataset_tags(&self, id: i32, tags: Vec<String>) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.add_tags(id, tags, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset add-tags task"))?
    }

    pub async fn remove_dataset_tags(
        &self,
        id: i32,
        tags: Vec<String>,
    ) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.remove_tags(id, tags, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset remove-tags task"))?
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.delete_dataset(id, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset delete task"))?
    }

    pub async fn trash_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.trash_dataset(id, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset trash task"))?
    }

    pub async fn restore_dataset(&self, id: i32) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || catalog.restore_dataset(id, &events))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join dataset restore task"))?
    }

    pub async fn delete_tag(&self, tag: String) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.delete_tag(tag))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join tag delete task"))?
    }

    pub async fn rename_tag(&self, old_name: String, new_name: String) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.rename_tag(old_name, new_name))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join tag rename task"))?
    }

    pub async fn merge_tag(&self, source: String, target: String) -> Result<(), CatalogError> {
        let state = self.state().map_err(|_| catalog_state_dropped())?;
        let tracker = state.tracker.clone();
        let catalog = state.dataset_catalog.clone();
        tracker
            .spawn_blocking(move || catalog.merge_tag(source, target))
            .await
            .map_err(|error| catalog_join_error(error, "failed to join tag merge task"))?
    }

    pub async fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, ReadError> {
        let state = self.state().map_err(|_| read_state_dropped())?;
        let tracker = state.tracker.clone();
        let read = state.dataset_read.clone();
        tracker
            .spawn_blocking(move || read.get_dataset_reader(id))
            .await
            .map_err(|error| read_join_error(error, "failed to join dataset read task"))?
    }

    pub async fn create_empty_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<DatasetRecord, IngestError> {
        let request = CreateDatasetRequest {
            name,
            description,
            tags,
        };
        let state = self.state().map_err(|_| ingest_state_dropped())?;
        let tracker = state.tracker.clone();
        let ingest = state.dataset_ingest.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || {
                let mut sent_finish = false;
                ingest.create_dataset(
                    &request,
                    || {
                        if sent_finish {
                            None
                        } else {
                            sent_finish = true;
                            Some(CreateDatasetInput::Finish)
                        }
                    },
                    &events,
                )
            })
            .await
            .map_err(|error| ingest_join_error(error, "failed to join dataset create task"))?
    }

    pub(crate) async fn create_dataset_from_receiver(
        &self,
        request: CreateDatasetRequest,
        mut receiver: mpsc::Receiver<CreateDatasetInput>,
    ) -> Result<DatasetRecord, IngestError> {
        let state = self.state().map_err(|_| ingest_state_dropped())?;
        let tracker = state.tracker.clone();
        let ingest = state.dataset_ingest.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        tracker
            .spawn_blocking(move || {
                ingest.create_dataset(&request, || receiver.blocking_recv(), &events)
            })
            .await
            .map_err(|error| ingest_join_error(error, "failed to join dataset create task"))?
    }
}

pub struct AppManager {
    state: Arc<AppState>,
    handle: AppHandle,
    started: bool,
}

impl AppManager {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    pub fn new(root: WorkspaceRoot) -> anyhow::Result<Self> {
        let state = AppState::new(root)?;
        let handle = AppHandle::new(Arc::downgrade(&state));
        Ok(Self {
            state,
            handle,
            started: false,
        })
    }

    pub fn new_with_path(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = WorkspaceRoot::create(path)?;
        Self::new(root)
    }

    #[instrument(skip(self, runtime), fields(workspace.path = ?self.handle.paths()?.root()))]
    pub fn start(mut self, runtime: &Handle) -> anyhow::Result<Self> {
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

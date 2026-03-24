mod server;

use std::{
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

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
    database::{core, core::DatabaseError, dataset as database_dataset},
    dataset::{
        CreateDatasetInput, CreateDatasetRequest, DatasetEvent, DatasetId, DatasetListQuery,
        DatasetReader, DatasetRecord, DatasetUpdate, ImportPreview,
        catalog::{CatalogError, DatasetCatalogService},
        events::DatasetEventPublisher,
        ingest::{DatasetIngestService, IngestError, WriteSessionRegistry},
        read::{DatasetReadService, ReadError},
    },
    workspace::{WorkspaceError, WorkspacePaths, WorkspaceRoot},
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
    #[error("App server is already started")]
    AlreadyStarted,
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("Subscription lagged by {skipped} messages")]
    Lagged { skipped: u64 },
    #[error("Subscription closed")]
    Closed,
}

#[derive(Debug, Error)]
pub enum CatalogAppError {
    #[error(transparent)]
    Domain(#[from] CatalogError),
    #[error("App state has been dropped")]
    StateDropped,
    #[error("Background task panicked while {operation}")]
    TaskPanic { operation: &'static str },
    #[error("Background task was cancelled while {operation}")]
    TaskCancelled { operation: &'static str },
}

#[derive(Debug, Error)]
pub enum ReadAppError {
    #[error(transparent)]
    Domain(#[from] ReadError),
    #[error("App state has been dropped")]
    StateDropped,
    #[error("Background task panicked while {operation}")]
    TaskPanic { operation: &'static str },
    #[error("Background task was cancelled while {operation}")]
    TaskCancelled { operation: &'static str },
}

#[derive(Debug, Error)]
pub enum IngestAppError {
    #[error(transparent)]
    Domain(#[from] IngestError),
    #[error("App state has been dropped")]
    StateDropped,
    #[error("Background task panicked while {operation}")]
    TaskPanic { operation: &'static str },
    #[error("Background task was cancelled while {operation}")]
    TaskCancelled { operation: &'static str },
}

pub struct DatasetEventSubscription {
    inner: broadcast::Receiver<DatasetEvent>,
}

impl DatasetEventSubscription {
    pub async fn recv(&mut self) -> std::result::Result<DatasetEvent, SubscriptionError> {
        self.inner.recv().await.map_err(Into::into)
    }
}

pub struct UiCommandSubscription {
    inner: broadcast::Receiver<UiCommand>,
}

impl UiCommandSubscription {
    pub async fn recv(&mut self) -> std::result::Result<UiCommand, SubscriptionError> {
        self.inner.recv().await.map_err(Into::into)
    }
}

impl From<broadcast::error::RecvError> for SubscriptionError {
    fn from(error: broadcast::error::RecvError) -> Self {
        match error {
            broadcast::error::RecvError::Lagged(skipped) => Self::Lagged { skipped },
            broadcast::error::RecvError::Closed => Self::Closed,
        }
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
    fn new(root: WorkspaceRoot) -> Result<Arc<Self>, DatabaseError> {
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

fn init_database(root: &WorkspaceRoot) -> Result<core::Pool, DatabaseError> {
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

fn catalog_join_error(error: &tokio::task::JoinError, operation: &'static str) -> CatalogAppError {
    if error.is_cancelled() {
        CatalogAppError::TaskCancelled { operation }
    } else {
        CatalogAppError::TaskPanic { operation }
    }
}

fn ingest_join_error(error: &tokio::task::JoinError, operation: &'static str) -> IngestAppError {
    if error.is_cancelled() {
        IngestAppError::TaskCancelled { operation }
    } else {
        IngestAppError::TaskPanic { operation }
    }
}

fn read_join_error(error: &tokio::task::JoinError, operation: &'static str) -> ReadAppError {
    if error.is_cancelled() {
        ReadAppError::TaskCancelled { operation }
    } else {
        ReadAppError::TaskPanic { operation }
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

    fn state(&self) -> std::result::Result<Arc<AppState>, AppError> {
        self.state.upgrade().ok_or(AppError::StateDropped)
    }

    async fn run_catalog_task<T, F>(
        &self,
        operation: &'static str,
        work: F,
    ) -> Result<T, CatalogAppError>
    where
        T: Send + 'static,
        F: FnOnce(DatasetCatalogService) -> Result<T, CatalogError> + Send + 'static,
    {
        let state = self.state.upgrade().ok_or(CatalogAppError::StateDropped)?;
        let catalog = state.dataset_catalog.clone();
        Ok(state
            .tracker
            .spawn_blocking(move || work(catalog))
            .await
            .map_err(|error| catalog_join_error(&error, operation))??)
    }

    async fn run_catalog_task_with_events<T, F>(
        &self,
        operation: &'static str,
        work: F,
    ) -> Result<T, CatalogAppError>
    where
        T: Send + 'static,
        F: FnOnce(DatasetCatalogService, BroadcastDatasetEvents) -> Result<T, CatalogError>
            + Send
            + 'static,
    {
        let state = self.state.upgrade().ok_or(CatalogAppError::StateDropped)?;
        let catalog = state.dataset_catalog.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        Ok(state
            .tracker
            .spawn_blocking(move || work(catalog, events))
            .await
            .map_err(|error| catalog_join_error(&error, operation))??)
    }

    async fn run_read_task<T, F>(&self, operation: &'static str, work: F) -> Result<T, ReadAppError>
    where
        T: Send + 'static,
        F: FnOnce(DatasetReadService) -> Result<T, ReadError> + Send + 'static,
    {
        let state = self.state.upgrade().ok_or(ReadAppError::StateDropped)?;
        let read = state.dataset_read.clone();
        Ok(state
            .tracker
            .spawn_blocking(move || work(read))
            .await
            .map_err(|error| read_join_error(&error, operation))??)
    }

    async fn run_ingest_task<T, F>(
        &self,
        operation: &'static str,
        work: F,
    ) -> Result<T, IngestAppError>
    where
        T: Send + 'static,
        F: FnOnce(DatasetIngestService, BroadcastDatasetEvents) -> Result<T, IngestError>
            + Send
            + 'static,
    {
        let state = self.state.upgrade().ok_or(IngestAppError::StateDropped)?;
        let ingest = state.dataset_ingest.clone();
        let events = BroadcastDatasetEvents {
            sender: state.dataset_event_sender.clone(),
        };
        Ok(state
            .tracker
            .spawn_blocking(move || work(ingest, events))
            .await
            .map_err(|error| ingest_join_error(&error, operation))??)
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

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, CatalogAppError> {
        self.run_catalog_task("failed to join dataset get task", move |catalog| {
            catalog.get_dataset(id)
        })
        .await
    }

    pub async fn get_dataset_including_deleted(
        &self,
        id: DatasetId,
    ) -> Result<DatasetRecord, CatalogAppError> {
        self.run_catalog_task(
            "failed to join dataset get-including-deleted task",
            move |catalog| catalog.get_dataset_including_deleted(id),
        )
        .await
    }

    pub async fn list_datasets(
        &self,
        query: DatasetListQuery,
    ) -> Result<Vec<DatasetRecord>, CatalogAppError> {
        self.run_catalog_task("failed to join dataset list task", move |catalog| {
            catalog.list_datasets(query)
        })
        .await
    }

    pub async fn list_dataset_tags(&self) -> Result<Vec<String>, CatalogAppError> {
        self.run_catalog_task("failed to join dataset tag list task", move |catalog| {
            catalog.list_dataset_tags()
        })
        .await
    }

    pub async fn update_dataset(
        &self,
        id: i32,
        update: DatasetUpdate,
    ) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset update task",
            move |catalog, events| catalog.update_dataset(id, update, &events),
        )
        .await
    }

    pub async fn add_dataset_tags(
        &self,
        id: i32,
        tags: Vec<String>,
    ) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset add-tags task",
            move |catalog, events| catalog.add_tags(id, tags, &events),
        )
        .await
    }

    pub async fn remove_dataset_tags(
        &self,
        id: i32,
        tags: Vec<String>,
    ) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset remove-tags task",
            move |catalog, events| catalog.remove_tags(id, tags, &events),
        )
        .await
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset delete task",
            move |catalog, events| catalog.delete_dataset(id, &events),
        )
        .await
    }

    pub async fn trash_dataset(&self, id: i32) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset trash task",
            move |catalog, events| catalog.trash_dataset(id, &events),
        )
        .await
    }

    pub async fn restore_dataset(&self, id: i32) -> Result<(), CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset restore task",
            move |catalog, events| catalog.restore_dataset(id, &events),
        )
        .await
    }

    pub async fn delete_tag(&self, tag: String) -> Result<(), CatalogAppError> {
        self.run_catalog_task("failed to join tag delete task", move |catalog| {
            catalog.delete_tag(tag)
        })
        .await
    }

    pub async fn rename_tag(
        &self,
        old_name: String,
        new_name: String,
    ) -> Result<(), CatalogAppError> {
        self.run_catalog_task("failed to join tag rename task", move |catalog| {
            catalog.rename_tag(old_name, new_name)
        })
        .await
    }

    pub async fn merge_tag(&self, source: String, target: String) -> Result<(), CatalogAppError> {
        self.run_catalog_task("failed to join tag merge task", move |catalog| {
            catalog.merge_tag(source, target)
        })
        .await
    }

    pub async fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, ReadAppError> {
        self.run_read_task("failed to join dataset read task", move |read| {
            read.get_dataset_reader(id)
        })
        .await
    }

    pub async fn create_empty_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<DatasetRecord, IngestAppError> {
        let request = CreateDatasetRequest {
            name,
            description,
            tags,
        };
        self.run_ingest_task(
            "failed to join dataset create task",
            move |ingest, events| {
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
            },
        )
        .await
    }

    pub(crate) async fn create_dataset_from_receiver(
        &self,
        request: CreateDatasetRequest,
        mut receiver: mpsc::Receiver<CreateDatasetInput>,
    ) -> Result<DatasetRecord, IngestAppError> {
        self.run_ingest_task(
            "failed to join dataset create task",
            move |ingest, events| {
                ingest.create_dataset(&request, || receiver.blocking_recv(), &events)
            },
        )
        .await
    }

    pub async fn export_dataset(
        &self,
        id: DatasetId,
        output_dir: std::path::PathBuf,
    ) -> Result<std::path::PathBuf, CatalogAppError> {
        self.run_catalog_task("failed to join dataset export task", move |catalog| {
            catalog.export_dataset(id, &output_dir)
        })
        .await
    }

    pub async fn preview_import(
        &self,
        archive_path: std::path::PathBuf,
    ) -> Result<ImportPreview, CatalogAppError> {
        self.run_catalog_task("failed to join import preview task", move |catalog| {
            catalog.preview_import(&archive_path)
        })
        .await
    }

    pub async fn import_dataset(
        &self,
        archive_path: std::path::PathBuf,
        force: bool,
    ) -> Result<DatasetRecord, CatalogAppError> {
        self.run_catalog_task_with_events(
            "failed to join dataset import task",
            move |catalog, events| catalog.import_dataset(&archive_path, force, &events),
        )
        .await
    }
}

pub struct AppManager {
    state: Arc<AppState>,
    handle: AppHandle,
    started: bool,
}

impl AppManager {
    #[instrument(skip(root), fields(workspace.path = ?root.paths().root()))]
    pub fn new(root: WorkspaceRoot) -> Result<Self, AppError> {
        let state = AppState::new(root)?;
        let handle = AppHandle::new(Arc::downgrade(&state));
        Ok(Self {
            state,
            handle,
            started: false,
        })
    }

    pub fn new_with_path(path: impl Into<PathBuf>) -> Result<Self, AppError> {
        let root = WorkspaceRoot::create(path)?;
        Self::new(root)
    }

    #[instrument(skip(self, runtime), fields(workspace.path = ?self.handle.paths()?.root()))]
    pub fn start(mut self, runtime: &Handle) -> Result<Self, AppError> {
        if self.started {
            return Err(AppError::AlreadyStarted);
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

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use super::{catalog_join_error, ingest_join_error, read_join_error};
    use crate::app::{CatalogAppError, IngestAppError, ReadAppError};

    fn panic_join_error() -> tokio::task::JoinError {
        Runtime::new()
            .expect("runtime")
            .block_on(async { tokio::spawn(async { panic!("boom") }).await.unwrap_err() })
    }

    fn cancelled_join_error() -> tokio::task::JoinError {
        Runtime::new().expect("runtime").block_on(async {
            let handle = tokio::spawn(async {
                tokio::task::yield_now().await;
            });
            handle.abort();
            handle.await.unwrap_err()
        })
    }

    #[test]
    fn catalog_join_error_preserves_panic_context() {
        let error = catalog_join_error(&panic_join_error(), "joining catalog task");
        assert!(matches!(
            error,
            CatalogAppError::TaskPanic {
                operation: "joining catalog task"
            }
        ));
    }

    #[test]
    fn ingest_join_error_preserves_cancel_context() {
        let error = ingest_join_error(&cancelled_join_error(), "joining ingest task");
        assert!(matches!(
            error,
            IngestAppError::TaskCancelled {
                operation: "joining ingest task"
            }
        ));
    }

    #[test]
    fn read_join_error_preserves_cancel_context() {
        let error = read_join_error(&cancelled_join_error(), "joining read task");
        assert!(matches!(
            error,
            ReadAppError::TaskCancelled {
                operation: "joining read task"
            }
        ));
    }
}

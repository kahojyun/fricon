use std::sync::{Arc, Mutex};

use anyhow::Result;
use fricon::runtime::app::{AppEvent, AppHandle, AppManager};
use tauri::async_runtime;
use tauri_specta::Event;
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, warn};

use crate::dataset_browser::{DatasetCreated, DatasetInfo, DatasetUpdated};

pub(crate) struct AppState {
    manager: Mutex<Option<AppManager>>,
    current_dataset: Mutex<Option<(i32, Arc<fricon::DatasetReader>)>>,
}

impl AppState {
    pub(crate) fn new(workspace_path: std::path::PathBuf) -> Result<Self> {
        let _runtime_guard = async_runtime::handle().inner().enter();
        let app_manager = AppManager::serve_with_path(workspace_path)?;
        Ok(Self {
            manager: Mutex::new(Some(app_manager)),
            current_dataset: Mutex::new(None),
        })
    }

    pub(crate) fn start_event_listener(&self, app_handle: tauri::AppHandle) {
        let app = self.app();
        let mut event_rx = match app.subscribe_to_events() {
            Ok(event_rx) => event_rx,
            Err(err) => {
                error!(error = %err, "Failed to subscribe to app events");
                return;
            }
        };

        async_runtime::spawn(async move {
            loop {
                let event = match event_rx.recv().await {
                    Ok(event) => event,
                    Err(RecvError::Lagged(skipped)) => {
                        warn!(skipped, "App event listener lagged behind");
                        continue;
                    }
                    Err(RecvError::Closed) => {
                        warn!("App event listener stopped (channel closed)");
                        break;
                    }
                };

                match event {
                    AppEvent::DatasetCreated {
                        id,
                        name,
                        description,
                        favorite,
                        tags,
                        status,
                        created_at,
                    } => {
                        if let Err(err) = DatasetCreated(DatasetInfo::new(
                            id,
                            name,
                            description,
                            favorite,
                            tags,
                            status.into(),
                            created_at,
                        ))
                        .emit(&app_handle)
                        {
                            warn!(
                                dataset_id = id,
                                error = %err,
                                "Failed to emit DatasetCreated event"
                            );
                        }
                    }
                    AppEvent::DatasetUpdated {
                        id,
                        name,
                        description,
                        favorite,
                        tags,
                        status,
                        created_at,
                    } => {
                        if let Err(err) = DatasetUpdated(DatasetInfo::new(
                            id,
                            name,
                            description,
                            favorite,
                            tags,
                            status.into(),
                            created_at,
                        ))
                        .emit(&app_handle)
                        {
                            warn!(
                                dataset_id = id,
                                error = %err,
                                "Failed to emit DatasetUpdated event"
                            );
                        }
                    }
                }
            }
        });
    }

    pub(crate) fn app(&self) -> AppHandle {
        self.manager
            .lock()
            .expect("Failed to acquire lock on app state")
            .as_ref()
            .expect("App should be running")
            .handle()
            .clone()
    }

    pub(crate) fn shutdown(&self) {
        async_runtime::block_on(async {
            let app_manager = self
                .manager
                .lock()
                .expect("Failed to acquire lock on app state")
                .take()
                .expect("App should be running");
            app_manager.shutdown().await;
        });
    }

    pub(crate) async fn dataset(&self, id: i32) -> Result<Arc<fricon::DatasetReader>> {
        if let Some((current_id, current_dataset)) = self
            .current_dataset
            .lock()
            .expect("Should not be poisoned.")
            .clone()
            && current_id == id
        {
            Ok(current_dataset)
        } else {
            let dataset = self
                .app()
                .dataset_read()
                .get_dataset_reader(id.into())
                .await?;
            let dataset = Arc::new(dataset);
            *self
                .current_dataset
                .lock()
                .expect("Should not be poisoned.") = Some((id, dataset.clone()));
            Ok(dataset)
        }
    }
}

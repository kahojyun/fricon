use fricon::app::AppEvent;
use tauri::async_runtime;
use tauri_specta::Event;
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, warn};

use crate::{
    application::session::WorkspaceSession,
    tauri_api::dataset::{DatasetCreated, DatasetInfo, DatasetUpdated},
};

pub(crate) fn start_event_forwarder(session: &WorkspaceSession, app_handle: tauri::AppHandle) {
    let mut event_rx = match session.app().subscribe_to_events() {
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

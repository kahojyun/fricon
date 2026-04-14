use fricon::{
    DatasetEvent,
    app::{SubscriptionError, UiCommand},
};
use tauri::async_runtime;
use tauri_specta::Event;
use tracing::{error, warn};

use crate::{
    desktop_runtime::{runtime::show_main_window, session::WorkspaceSession},
    features::datasets::{
        tauri::{DatasetChanged, DatasetWriteProgress},
        types::DatasetInfo,
    },
};

pub(crate) fn start_event_forwarder(session: &WorkspaceSession, app_handle: tauri::AppHandle) {
    let dataset_events = match session.app().subscribe_dataset_events() {
        Ok(event_rx) => event_rx,
        Err(err) => {
            error!(error = %err, "Failed to subscribe to dataset events");
            return;
        }
    };
    let ui_commands = match session.app().subscribe_ui_commands() {
        Ok(command_rx) => command_rx,
        Err(err) => {
            error!(error = %err, "Failed to subscribe to UI commands");
            return;
        }
    };

    async_runtime::spawn(forward_dataset_events(dataset_events, app_handle.clone()));
    async_runtime::spawn(forward_ui_commands(ui_commands, app_handle));
}

async fn forward_dataset_events(
    mut dataset_events: fricon::app::DatasetEventSubscription,
    app_handle: tauri::AppHandle,
) {
    loop {
        let event = match dataset_events.recv().await {
            Ok(event) => event,
            Err(SubscriptionError::Lagged { skipped }) => {
                warn!(skipped, "Dataset event listener lagged behind");
                continue;
            }
            Err(SubscriptionError::Closed) => {
                warn!("Dataset event listener stopped (channel closed)");
                break;
            }
        };

        emit_dataset_event(&event, &app_handle);
    }
}

fn emit_dataset_event(event: &DatasetEvent, app_handle: &tauri::AppHandle) {
    let changed = match event {
        DatasetEvent::Created(r) => DatasetChanged::Created {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::WriteProgress(progress) => DatasetChanged::WriteProgress {
            progress: DatasetWriteProgress {
                id: progress.id,
                row_count: progress.row_count,
            },
        },
        DatasetEvent::StatusChanged(r) => DatasetChanged::StatusChanged {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::MetadataUpdated(r) => DatasetChanged::MetadataUpdated {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::TagsChanged(r) => DatasetChanged::TagsChanged {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::Trashed(r) => DatasetChanged::Trashed {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::Restored(r) => DatasetChanged::Restored {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::Deleted(r) => DatasetChanged::Deleted {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::Imported(r) => DatasetChanged::Imported {
            info: DatasetInfo::from(r),
        },
        DatasetEvent::GlobalTagsChanged => DatasetChanged::GlobalTagsChanged,
    };
    if let Err(err) = changed.emit(app_handle) {
        warn!(error = %err, "Failed to emit dataset event");
    }
}

async fn forward_ui_commands(
    mut ui_commands: fricon::app::UiCommandSubscription,
    app_handle: tauri::AppHandle,
) {
    loop {
        let command = match ui_commands.recv().await {
            Ok(command) => command,
            Err(SubscriptionError::Lagged { skipped }) => {
                warn!(skipped, "UI command listener lagged behind");
                continue;
            }
            Err(SubscriptionError::Closed) => {
                warn!("UI command listener stopped (channel closed)");
                break;
            }
        };

        match command {
            UiCommand::ShowUi => show_main_window(&app_handle),
        }
    }
}

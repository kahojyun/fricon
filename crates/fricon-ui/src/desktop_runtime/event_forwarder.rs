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
        tauri::{DatasetChangeKind, DatasetChanged},
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
    let info = dataset_info_from_event(event);
    let kind = match event {
        DatasetEvent::Created(_) => DatasetChangeKind::Created,
        DatasetEvent::StatusChanged(_) => DatasetChangeKind::StatusChanged,
        DatasetEvent::MetadataUpdated(_) => DatasetChangeKind::MetadataUpdated,
        DatasetEvent::TagsChanged(_) => DatasetChangeKind::TagsChanged,
        DatasetEvent::Trashed(_) => DatasetChangeKind::Trashed,
        DatasetEvent::Restored(_) => DatasetChangeKind::Restored,
        DatasetEvent::Deleted(_) => DatasetChangeKind::Deleted,
        DatasetEvent::Imported(_) => DatasetChangeKind::Imported,
        DatasetEvent::GlobalTagsChanged => DatasetChangeKind::GlobalTagsChanged,
    };
    let dataset_id = info.as_ref().map(|i| i.id);
    let result = DatasetChanged { info, kind }.emit(app_handle);

    if let Err(err) = result {
        warn!(
            dataset_id = ?dataset_id,
            error = %err,
            "Failed to emit dataset event"
        );
    }
}

fn dataset_info_from_event(event: &DatasetEvent) -> Option<DatasetInfo> {
    let record = match event {
        DatasetEvent::Created(record)
        | DatasetEvent::StatusChanged(record)
        | DatasetEvent::MetadataUpdated(record)
        | DatasetEvent::TagsChanged(record)
        | DatasetEvent::Trashed(record)
        | DatasetEvent::Restored(record)
        | DatasetEvent::Deleted(record)
        | DatasetEvent::Imported(record) => record,
        DatasetEvent::GlobalTagsChanged => return None,
    };
    Some(DatasetInfo::from(record))
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

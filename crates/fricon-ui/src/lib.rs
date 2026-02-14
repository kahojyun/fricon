mod commands;
mod models;

use std::{
    any::Any,
    fs, io, panic,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context as _, Result};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri::{
    Manager, RunEvent, WindowEvent, async_runtime,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_specta::Event;
use tokio::signal;
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::commands::{DatasetCreated, DatasetInfo, DatasetUpdated};

struct AppState {
    manager: Mutex<Option<(fricon::AppManager, WorkerGuard)>>,
    current_dataset: Mutex<Option<(i32, Arc<fricon::DatasetReader>)>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchSource {
    Standalone,
    Cli {
        command_name: String,
        cli_help: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Dialog,
    Terminal,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceLaunchError {
    #[error("workspace path is required")]
    WorkspacePathMissing,
    #[error(
        "invalid workspace path '{}': {reason}",
        .path
            .as_ref()
            .map_or_else(|| "<none>".to_string(), |p| p.display().to_string())
    )]
    WorkspacePathInvalid {
        path: Option<PathBuf>,
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct LaunchContext {
    pub launch_source: LaunchSource,
    pub workspace_path: Option<PathBuf>,
    pub interaction_mode: InteractionMode,
}

impl AppState {
    fn new(workspace_path: PathBuf) -> Result<Self> {
        let _runtime_guard = async_runtime::handle().inner().enter();
        let log_guard = setup_logging(workspace_path.clone())?;
        let app_manager = fricon::AppManager::serve_with_path(workspace_path)?;
        Ok(Self {
            manager: Mutex::new(Some((app_manager, log_guard))),
            current_dataset: Mutex::new(None),
        })
    }

    fn start_event_listener(&self, app_handle: tauri::AppHandle) {
        let app = self.app();
        let mut event_rx = app
            .subscribe_to_events()
            .expect("Failed to subscribe to events");

        async_runtime::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    fricon::AppEvent::DatasetCreated {
                        id,
                        name,
                        description,
                        favorite,
                        tags,
                        status,
                        created_at,
                    } => {
                        let _ = DatasetCreated(DatasetInfo {
                            id,
                            name,
                            description,
                            favorite,
                            tags,
                            status: status.into(),
                            created_at,
                        })
                        .emit(&app_handle);
                    }
                    fricon::AppEvent::DatasetUpdated {
                        id,
                        name,
                        description,
                        favorite,
                        tags,
                        status,
                        created_at,
                    } => {
                        let _ = DatasetUpdated(DatasetInfo {
                            id,
                            name,
                            description,
                            favorite,
                            tags,
                            status: status.into(),
                            created_at,
                        })
                        .emit(&app_handle);
                    }
                }
            }
        });
    }

    fn app(&self) -> fricon::AppHandle {
        self.manager
            .lock()
            .expect("Failed to acquire lock on app state")
            .as_ref()
            .expect("App should be running")
            .0
            .handle()
            .clone()
    }

    fn shutdown(&self) {
        async_runtime::block_on(async {
            let (app_manager, _guard) = self
                .manager
                .lock()
                .expect("Failed to acquire lock on app state")
                .take()
                .expect("App should be running");
            app_manager.shutdown().await;
        });
    }

    async fn dataset(&self, id: i32) -> Result<Arc<fricon::DatasetReader>> {
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
                .dataset_manager()
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

enum WorkspaceSelection {
    Selected(PathBuf),
    Exit,
}

enum MissingWorkspaceAction {
    ChooseWorkspace,
    ShowCliHelpAndExit,
    Exit,
}

const CHOOSE_WORKSPACE_BUTTON: &str = "Choose workspace";
const HELP_BUTTON: &str = "Help";
const EXIT_BUTTON: &str = "Exit";

pub fn run_with_context(context: &LaunchContext) -> Result<()> {
    install_panic_hook(context.interaction_mode);
    match context.interaction_mode {
        InteractionMode::Terminal => run_with_context_terminal_mode(context),
        InteractionMode::Dialog => run_with_context_dialog_mode(context),
    }
}

pub fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    run_with_context(&LaunchContext {
        launch_source: LaunchSource::Standalone,
        workspace_path: Some(workspace_path),
        interaction_mode: InteractionMode::Dialog,
    })
}

fn resolve_workspace_path(context: &LaunchContext) -> Result<Option<PathBuf>> {
    match &context.workspace_path {
        Some(path) => match validate_workspace_path(path) {
            Ok(path) => Ok(Some(path)),
            Err(err) => match context.interaction_mode {
                InteractionMode::Dialog => Ok(None),
                InteractionMode::Terminal => Err(err.into()),
            },
        },
        None => match context.interaction_mode {
            InteractionMode::Dialog => Ok(None),
            InteractionMode::Terminal => Err(WorkspaceLaunchError::WorkspacePathMissing.into()),
        },
    }
}

fn run_with_context_terminal_mode(context: &LaunchContext) -> Result<()> {
    let workspace_path =
        resolve_workspace_path(context)?.ok_or(WorkspaceLaunchError::WorkspacePathMissing)?;
    run_with_canonical_workspace(workspace_path)
}

fn run_with_context_dialog_mode(context: &LaunchContext) -> Result<()> {
    let mut next_workspace = resolve_workspace_path(context)?;
    loop {
        let workspace_path = match next_workspace.take() {
            Some(path) => path,
            None => match select_workspace_path(&context.launch_source)? {
                WorkspaceSelection::Selected(path) => path,
                WorkspaceSelection::Exit => return Ok(()),
            },
        };

        if let Err(err) = run_with_canonical_workspace(workspace_path) {
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Failed to open workspace")
                .set_description(err.to_string())
                .set_buttons(MessageButtons::Ok)
                .show();
            continue;
        }

        return Ok(());
    }
}

fn run_with_canonical_workspace(workspace_path: PathBuf) -> Result<()> {
    let app_state = AppState::new(workspace_path).context("Failed to open workspace")?;

    #[expect(clippy::exit, reason = "Required by Tauri framework")]
    let tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands::invoke_handler())
        .manage(app_state)
        .setup(|app| {
            install_ctrl_c_handler(app);
            build_system_tray(app)?;
            commands::mount_typed_events(&app.handle().clone());

            // Start event listener
            let app_state = app.state::<AppState>();
            app_state.start_event_listener(app.handle().clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    tauri_app.run(|app, event| match event {
        RunEvent::Exit => {
            app.state::<AppState>().shutdown();
        }
        RunEvent::ExitRequested {
            code: None, api, ..
        } => {
            api.prevent_exit();
        }
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            api.prevent_close();
            let window = app
                .get_webview_window(&label)
                .expect("Failed to get webview window");
            let _ = window.hide();
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            show_main_window(app);
        }
        _ => (),
    });

    Ok(())
}

pub fn export_bindings(path: impl AsRef<std::path::Path>) -> Result<()> {
    commands::export_bindings(path)
}

fn select_workspace_path(launch_source: &LaunchSource) -> Result<WorkspaceSelection> {
    loop {
        match prompt_missing_workspace_action(launch_source) {
            MissingWorkspaceAction::ChooseWorkspace => {}
            MissingWorkspaceAction::ShowCliHelpAndExit => {
                if let LaunchSource::Cli {
                    command_name,
                    cli_help,
                } = launch_source
                {
                    show_cli_help(command_name, cli_help);
                }
                return Ok(WorkspaceSelection::Exit);
            }
            MissingWorkspaceAction::Exit => return Ok(WorkspaceSelection::Exit),
        }

        let Some(path) = FileDialog::new().pick_folder() else {
            return Ok(WorkspaceSelection::Exit);
        };

        match validate_workspace_path(&path) {
            Ok(path) => return Ok(WorkspaceSelection::Selected(path)),
            Err(err) => {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Invalid workspace")
                    .set_description(err.to_string())
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    }
}

fn validate_workspace_path(
    path: &std::path::Path,
) -> std::result::Result<PathBuf, WorkspaceLaunchError> {
    let canonical =
        fs::canonicalize(path).map_err(|err| WorkspaceLaunchError::WorkspacePathInvalid {
            path: Some(path.to_path_buf()),
            reason: err.to_string(),
        })?;
    fricon::WorkspaceRoot::validate(canonical.clone()).map_err(|err| {
        WorkspaceLaunchError::WorkspacePathInvalid {
            path: Some(canonical.clone()),
            reason: err.to_string(),
        }
    })?;
    Ok(canonical)
}

fn dialog_is_choose_workspace(result: &MessageDialogResult) -> bool {
    match result {
        MessageDialogResult::Ok | MessageDialogResult::Yes => true,
        MessageDialogResult::Custom(value) => value == CHOOSE_WORKSPACE_BUTTON,
        MessageDialogResult::No | MessageDialogResult::Cancel => false,
    }
}

fn prompt_missing_workspace_action(launch_source: &LaunchSource) -> MissingWorkspaceAction {
    let action = match launch_source {
        LaunchSource::Standalone => MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Workspace not found")
            .set_description(
                "No valid workspace path is available.\n\nChoose a workspace folder, or exit.",
            )
            .set_buttons(MessageButtons::OkCancelCustom(
                CHOOSE_WORKSPACE_BUTTON.to_string(),
                EXIT_BUTTON.to_string(),
            ))
            .show(),
        LaunchSource::Cli { .. } => MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Workspace not found")
            .set_description(
                "No valid workspace path is available.\n\nChoose a workspace folder, or view \
                 command line help.",
            )
            .set_buttons(MessageButtons::OkCancelCustom(
                CHOOSE_WORKSPACE_BUTTON.to_string(),
                HELP_BUTTON.to_string(),
            ))
            .show(),
    };

    if dialog_is_choose_workspace(&action) {
        return MissingWorkspaceAction::ChooseWorkspace;
    }

    match launch_source {
        LaunchSource::Standalone => MissingWorkspaceAction::Exit,
        LaunchSource::Cli { .. } => MissingWorkspaceAction::ShowCliHelpAndExit,
    }
}

fn show_cli_help(command_name: &str, cli_help: &str) {
    MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("Command line help")
        .set_description(build_cli_help_message(command_name, cli_help))
        .set_buttons(MessageButtons::Ok)
        .show();
}

fn build_cli_help_message(command_name: &str, cli_help: &str) -> String {
    format!("Command: {command_name}\n\n{cli_help}")
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn build_system_tray(app: &mut tauri::App) -> Result<()> {
    let menu = MenuBuilder::new(app).text("quit", "Quit").build()?;
    let _tray = TrayIconBuilder::new()
        .icon(
            app.default_window_icon()
                .expect("Failed to get default window icon")
                .clone(),
        )
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_main_window(app);
            }
        })
        .build(app)?;
    Ok(())
}

fn install_ctrl_c_handler(app: &mut tauri::App) {
    let app_handle = app.handle().clone();
    async_runtime::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                app_handle.exit(0);
            }
            Err(err) => {
                info!("Failed to listen for Ctrl+C: {}", err);
            }
        }
    });
}

fn setup_logging(workspace_path: PathBuf) -> Result<WorkerGuard> {
    let log_dir = fricon::get_log_dir(workspace_path)?;
    let rolling = RollingFileAppender::new(Rotation::DAILY, log_dir, "fricon.log");
    let (writer, guard) = tracing_appender::non_blocking(rolling);
    let file_layer = fmt::layer().json().with_writer(writer);
    let stdout_layer = if cfg!(debug_assertions) {
        Some(fmt::layer().with_writer(io::stdout))
    } else {
        None
    };
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    Ok(guard)
}

fn install_panic_hook(interaction_mode: InteractionMode) {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        if interaction_mode != InteractionMode::Dialog {
            return;
        }
        let message = build_panic_dialog_message(panic_info.payload(), panic_info.location());
        let _ = panic::catch_unwind(|| {
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Fricon crashed")
                .set_description(message)
                .set_buttons(MessageButtons::Ok)
                .show();
        });
    }));
}

fn build_panic_dialog_message(
    payload: &(dyn Any + Send),
    location: Option<&panic::Location<'_>>,
) -> String {
    let reason = if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    };
    let where_text = location.map_or_else(
        || "location unavailable".to_string(),
        |loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()),
    );
    format!("An unexpected internal error occurred.\n\nReason: {reason}\nLocation: {where_text}")
}

#[cfg(test)]
mod tests {
    use std::{fs, panic::Location, path::PathBuf};

    use fricon::WorkspaceRoot;
    use tempfile::tempdir;

    use super::{
        InteractionMode, LaunchContext, LaunchSource, WorkspaceLaunchError,
        build_panic_dialog_message, resolve_workspace_path, validate_workspace_path,
    };

    #[test]
    fn terminal_mode_missing_workspace_returns_workspace_missing_error() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: None,
            interaction_mode: InteractionMode::Terminal,
        });
        let error = result.expect_err("expected missing-workspace error");
        let launch_error = error
            .downcast_ref::<WorkspaceLaunchError>()
            .expect("error should be WorkspaceLaunchError");
        assert!(matches!(
            launch_error,
            WorkspaceLaunchError::WorkspacePathMissing
        ));
    }

    #[test]
    fn terminal_mode_invalid_workspace_returns_workspace_invalid_error() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: Some(PathBuf::from("/definitely/not/a/real/path")),
            interaction_mode: InteractionMode::Terminal,
        });
        let error = result.expect_err("expected invalid-workspace error");
        let launch_error = error
            .downcast_ref::<WorkspaceLaunchError>()
            .expect("error should be WorkspaceLaunchError");
        assert!(matches!(
            launch_error,
            WorkspaceLaunchError::WorkspacePathInvalid { .. }
        ));
    }

    #[test]
    fn dialog_mode_invalid_workspace_defers_to_picker_flow() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: Some(PathBuf::from("/definitely/not/a/real/path")),
            interaction_mode: InteractionMode::Dialog,
        });
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn validate_workspace_path_accepts_valid_workspace() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let workspace_path = temp_dir.path().join("workspace");
        let workspace =
            WorkspaceRoot::create_new(workspace_path.clone()).expect("workspace should be created");
        drop(workspace);

        let result = validate_workspace_path(&workspace_path);
        let expected =
            fs::canonicalize(workspace_path).expect("workspace path should canonicalize");
        assert_eq!(result.expect("workspace should validate"), expected);
    }

    #[test]
    fn validate_workspace_path_rejects_non_workspace_directory() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let non_workspace_dir = temp_dir.path().join("not-workspace");
        fs::create_dir_all(&non_workspace_dir).expect("directory should be created");

        let result = validate_workspace_path(&non_workspace_dir);
        assert!(matches!(
            result,
            Err(WorkspaceLaunchError::WorkspacePathInvalid { .. })
        ));
    }

    #[test]
    fn panic_dialog_message_includes_reason_and_location() {
        let message = build_panic_dialog_message(&"boom", Some(Location::caller()));
        assert!(message.contains("Reason: boom"));
        assert!(message.contains("Location: "));
    }
}

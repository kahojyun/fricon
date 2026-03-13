use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use fricon::ExistingUiProbeResult;
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use tauri::{
    Manager, RunEvent, WindowEvent, async_runtime,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tokio::signal;
use tracing::error;

use crate::{
    api,
    desktop_runtime::{
        app_state::AppState,
        event_forwarder::start_event_forwarder,
        logging::{
            WorkspaceLogSession, attach_workspace_file_logging, shutdown_workspace_file_logging,
        },
    },
    launch::{
        LaunchContext, WorkspaceLaunchError,
        resolve::{resolve_workspace_path, select_workspace_path},
    },
};

pub(crate) fn run_with_context_terminal_mode(context: &LaunchContext) -> Result<()> {
    let workspace_path =
        resolve_workspace_path(context)?.ok_or(WorkspaceLaunchError::WorkspacePathMissing)?;
    match prepare_workspace_runtime(&workspace_path)? {
        WorkspaceLaunchOutcome::Delegated => Ok(()),
        WorkspaceLaunchOutcome::Start {
            log_session: _log_session,
            app_state,
        } => run_with_app_state(app_state),
    }
}

pub(crate) fn run_with_context_dialog_mode(context: &LaunchContext) -> Result<()> {
    let mut next_workspace = resolve_workspace_path(context)?;
    loop {
        let workspace_path = match next_workspace.take() {
            Some(path) => path,
            None => match select_workspace_path(&context.launch_source)? {
                Some(path) => path,
                None => return Ok(()),
            },
        };

        match prepare_workspace_runtime(&workspace_path) {
            Ok(WorkspaceLaunchOutcome::Delegated) => return Ok(()),
            Ok(WorkspaceLaunchOutcome::Start {
                log_session: _log_session,
                app_state,
            }) => return run_with_app_state(app_state),
            Err(err) => {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Failed to open workspace")
                    .set_description(format!("{err:#}"))
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    }
}

#[derive(Debug)]
enum WorkspaceLaunchOutcome<LogSession = WorkspaceLogSession, State = AppState> {
    Delegated,
    Start {
        log_session: LogSession,
        app_state: State,
    },
}

fn prepare_workspace_runtime(workspace_path: &Path) -> Result<WorkspaceLaunchOutcome> {
    let probe_result =
        tauri::async_runtime::block_on(fricon::Client::probe_existing_ui(workspace_path))?;
    prepare_workspace_runtime_from_probe(probe_result, || {
        build_new_workspace_runtime(workspace_path)
    })
}

fn prepare_workspace_runtime_from_probe<LogSession, State, BuildRuntime>(
    probe_result: ExistingUiProbeResult,
    build_runtime: BuildRuntime,
) -> Result<WorkspaceLaunchOutcome<LogSession, State>>
where
    BuildRuntime: FnOnce() -> Result<(LogSession, State)>,
{
    match probe_result {
        ExistingUiProbeResult::UiShown => Ok(WorkspaceLaunchOutcome::Delegated),
        ExistingUiProbeResult::UiUnavailable => {
            bail!("workspace is already served by another process without a desktop UI attached")
        }
        ExistingUiProbeResult::NotRunning => {
            let (log_session, app_state) = build_runtime()?;
            Ok(WorkspaceLaunchOutcome::Start {
                log_session,
                app_state,
            })
        }
    }
}

fn build_new_workspace_runtime(workspace_path: &Path) -> Result<(WorkspaceLogSession, AppState)> {
    build_new_workspace_runtime_with(
        workspace_path,
        |path| {
            attach_workspace_file_logging(path).context("Failed to initialize workspace logging")
        },
        |path| AppState::new(path).context("Failed to open workspace"),
    )
}

fn build_new_workspace_runtime_with<LogSession, State, AttachLogging, BuildState>(
    workspace_path: &Path,
    attach_logging: AttachLogging,
    build_state: BuildState,
) -> Result<(LogSession, State)>
where
    AttachLogging: FnOnce(&Path) -> Result<LogSession>,
    BuildState: FnOnce(PathBuf) -> Result<State>,
{
    let log_session = attach_logging(workspace_path)?;
    let app_state = build_state(workspace_path.to_path_buf())?;
    Ok((log_session, app_state))
}

fn run_with_app_state(app_state: AppState) -> Result<()> {
    let specta_builder = api::specta_builder();
    #[expect(clippy::exit, reason = "Required by Tauri framework")]
    let tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .manage(app_state)
        .setup(move |app| {
            install_ctrl_c_handler(app);
            build_system_tray(app)?;
            specta_builder.mount_events(&app.handle().clone());

            let app_state = app.state::<AppState>();
            start_event_forwarder(app_state.session(), app.handle().clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let exit_code = tauri_app.run_return(|app, event| match event {
        RunEvent::Exit => {
            app.state::<AppState>().shutdown();
            shutdown_workspace_file_logging();
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

    if exit_code != 0 {
        bail!("tauri application exited with status code {exit_code}");
    }

    Ok(())
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
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
                error!(error = %err, "Failed to listen for Ctrl+C");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::*;

    #[test]
    fn prepare_workspace_runtime_from_probe_delegates_without_building() {
        let built = AtomicBool::new(false);

        let outcome = prepare_workspace_runtime_from_probe(ExistingUiProbeResult::UiShown, || {
            built.store(true, Ordering::SeqCst);
            Ok::<_, anyhow::Error>(("log", "state"))
        })
        .expect("delegation should succeed");

        assert!(matches!(outcome, WorkspaceLaunchOutcome::Delegated));
        assert!(!built.load(Ordering::SeqCst));
    }

    #[test]
    fn prepare_workspace_runtime_from_probe_errors_for_non_ui_server() {
        let built = AtomicBool::new(false);

        let error =
            prepare_workspace_runtime_from_probe(ExistingUiProbeResult::UiUnavailable, || {
                built.store(true, Ordering::SeqCst);
                Ok::<_, anyhow::Error>(("log", "state"))
            })
            .expect_err("non-UI server should block desktop startup");

        assert!(error.to_string().contains("without a desktop UI attached"));
        assert!(!built.load(Ordering::SeqCst));
    }

    #[test]
    fn build_new_workspace_runtime_with_attaches_logging_before_app_state() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let workspace_path = Path::new("/tmp/fricon-workspace");

        let (log_session, app_state) = build_new_workspace_runtime_with(
            workspace_path,
            {
                let order = Arc::clone(&order);
                move |path| {
                    order.lock().expect("order lock").push("attach_logging");
                    assert_eq!(path, workspace_path);
                    Ok::<_, anyhow::Error>("log")
                }
            },
            {
                let order = Arc::clone(&order);
                move |path| {
                    order.lock().expect("order lock").push("build_state");
                    assert_eq!(path, workspace_path.to_path_buf());
                    Ok::<_, anyhow::Error>("state")
                }
            },
        )
        .expect("runtime should build");

        assert_eq!(log_session, "log");
        assert_eq!(app_state, "state");
        assert_eq!(
            *order.lock().expect("order lock"),
            vec!["attach_logging", "build_state"]
        );
    }
}

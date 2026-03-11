use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use tauri::{
    Manager, RunEvent, WindowEvent, async_runtime,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tokio::signal;
use tracing::error;

use crate::{
    desktop_runtime::{
        app_state::AppState,
        logging::{
            WorkspaceLogSession, attach_workspace_file_logging, shutdown_workspace_file_logging,
        },
    },
    launch::{LaunchContext, WorkspaceLaunchError},
    tauri_api,
    workspace::{resolve_workspace_path, select_workspace_path},
};

pub(crate) fn run_with_context_terminal_mode(context: &LaunchContext) -> Result<()> {
    let workspace_path =
        resolve_workspace_path(context)?.ok_or(WorkspaceLaunchError::WorkspacePathMissing)?;
    let (_log_session, app_state) = build_workspace_runtime(workspace_path)?;
    run_with_app_state(app_state)
}

pub(crate) fn run_with_context_dialog_mode(context: &LaunchContext) -> Result<()> {
    let mut next_workspace = resolve_workspace_path(context)?;
    let (_log_session, app_state) = loop {
        let workspace_path = match next_workspace.take() {
            Some(path) => path,
            None => match select_workspace_path(&context.launch_source)? {
                Some(path) => path,
                None => return Ok(()),
            },
        };

        match build_workspace_runtime(workspace_path) {
            Ok(run_inputs) => break run_inputs,
            Err(err) => {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Failed to open workspace")
                    .set_description(format!("{err:#}"))
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    };

    run_with_app_state(app_state)
}

fn build_workspace_runtime(workspace_path: PathBuf) -> Result<(WorkspaceLogSession, AppState)> {
    let log_session = attach_workspace_file_logging(&workspace_path)
        .context("Failed to initialize workspace logging")?;
    let app_state = AppState::new(workspace_path).context("Failed to open workspace")?;
    Ok((log_session, app_state))
}

fn run_with_app_state(app_state: AppState) -> Result<()> {
    let specta_builder = tauri_api::specta_builder();
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
            app_state.start_event_listener(app.handle().clone());

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
                error!(error = %err, "Failed to listen for Ctrl+C");
            }
        }
    });
}

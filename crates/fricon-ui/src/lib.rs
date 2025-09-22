#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]
mod commands;

use std::{path::PathBuf, sync::Mutex};

use anyhow::{Context as _, Result};
use tauri::{
    Emitter, Manager, RunEvent, WindowEvent, async_runtime,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tracing::info;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, prelude::*};

struct AppState(Mutex<Option<(fricon::AppManager, WorkerGuard)>>);

impl AppState {
    async fn new(workspace_path: PathBuf) -> Result<Self> {
        let log_guard = setup_logging(workspace_path.clone())?;
        let app_manager = fricon::AppManager::serve_with_path(&workspace_path).await?;
        Ok(Self(Mutex::new(Some((app_manager, log_guard)))))
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
                        uuid,
                        name,
                        description,
                        tags,
                    } => {
                        let _ = app_handle.emit(
                            "dataset-created",
                            serde_json::json!({
                                "id": id,
                                "uuid": uuid,
                                "name": name,
                                "description": description,
                                "tags": tags
                            }),
                        );
                    }
                }
            }
        });
    }

    fn app(&self) -> fricon::AppHandle {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .expect("App should be running")
            .0
            .handle()
            .clone()
    }

    fn shutdown(&self) {
        async_runtime::block_on(async {
            let (app_manager, _guard) = self
                .0
                .lock()
                .unwrap()
                .take()
                .expect("App should be running");
            app_manager.shutdown().await;
        });
    }
}

pub fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    let app_state = async_runtime::block_on(AppState::new(workspace_path))
        .context("Failed to open workspace")?;

    let tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands::invoke_handler())
        .manage(app_state)
        .setup(|app| {
            install_ctrl_c_handler(app);
            build_system_tray(app)?;

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
            let window = app.get_webview_window(&label).unwrap();
            window.hide().ok();
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            show_main_window(app);
        }
        _ => (),
    });

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        w.unminimize().ok();
        w.show().ok();
        w.set_focus().ok();
    }
}

fn build_system_tray(app: &mut tauri::App) -> Result<()> {
    let menu = MenuBuilder::new(app).text("quit", "Quit").build()?;
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
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
        match tokio::signal::ctrl_c().await {
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
    let file_layer = tracing_subscriber::fmt::layer().json().with_writer(writer);

    let registry = tracing_subscriber::registry().with(file_layer);

    #[cfg(debug_assertions)]
    let registry = registry.with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout));

    registry.with(EnvFilter::from_default_env()).init();
    Ok(guard)
}

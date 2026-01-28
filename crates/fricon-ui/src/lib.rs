mod commands;
mod models;

use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context as _, Result};
use tauri::{
    Emitter, Manager, RunEvent, WindowEvent, async_runtime,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tokio::signal;
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::commands::DatasetInfo;

struct AppState {
    manager: Mutex<Option<(fricon::AppManager, WorkerGuard)>>,
    current_dataset: Mutex<Option<(i32, Arc<fricon::DatasetReader>)>>,
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
                        let _ = app_handle.emit(
                            "dataset-created",
                            DatasetInfo {
                                id,
                                name,
                                description,
                                favorite,
                                tags,
                                status,
                                created_at,
                            },
                        );
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
                        let _ = app_handle.emit(
                            "dataset-updated",
                            DatasetInfo {
                                id,
                                name,
                                description,
                                favorite,
                                tags,
                                status,
                                created_at,
                            },
                        );
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

pub fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    let app_state = AppState::new(workspace_path).context("Failed to open workspace")?;

    #[expect(clippy::exit, reason = "Required by Tauri framework")]
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

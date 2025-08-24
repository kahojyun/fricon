#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]
mod commands;

use std::{path::PathBuf, sync::Mutex};

use anyhow::{Context as _, Result};
use tauri::{Manager, RunEvent, async_runtime};
use tracing::info;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};

struct AppState(Mutex<Option<(fricon::AppManager, WorkerGuard)>>);

impl AppState {
    async fn new(workspace_path: PathBuf) -> Result<Self> {
        let log_guard = setup_logging(workspace_path.clone())?;
        let app_manager = fricon::AppManager::serve(&workspace_path).await?;
        Ok(Self(Mutex::new(Some((app_manager, log_guard)))))
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
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    tauri_app.run(|app, event| {
        if let RunEvent::Exit = event {
            app.state::<AppState>().shutdown();
        }
    });

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
    tracing_subscriber::fmt().json().with_writer(writer).init();
    Ok(guard)
}

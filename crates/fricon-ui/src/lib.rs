#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]
mod commands;

use std::{path::PathBuf, sync::Mutex, time::Duration};

use anyhow::{Context as _, Result};
use tauri::{Manager, RunEvent, async_runtime};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};

struct AppLifetime {
    server_handle: JoinHandle<Result<()>>,
    _log_guard: WorkerGuard,
}

impl AppLifetime {
    fn shutdown(self) {
        if let Err(e) = async_runtime::block_on(async move {
            self.server_handle.await.context("Server task panicked")?
        }) {
            error!("Server returned an error: {e}");
        }
    }
}

fn graceful_shutdown(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    // Signal server to shutdown
    state.cancellation_token.cancel();
    state
        .lifetime
        .lock()
        .unwrap()
        .take()
        .expect("AppLifetime should be consumed only here.")
        .shutdown();
}

struct AppState {
    app: fricon::App,
    lifetime: Mutex<Option<AppLifetime>>,
    cancellation_token: CancellationToken,
}

impl AppState {
    async fn new(workspace_path: PathBuf) -> Result<Self> {
        let log_guard = setup_logging(workspace_path.clone())?;
        let app = fricon::App::open(&workspace_path).await?;
        let cancellation_token = CancellationToken::new();

        // Start gRPC server in background
        let server_app = app.clone();
        let server_token = cancellation_token.clone();
        let server_handle =
            tokio::spawn(async move { fricon::run_server(server_app, server_token).await });

        let lifetime = AppLifetime {
            server_handle,
            _log_guard: log_guard,
        };

        Ok(Self {
            app,
            lifetime: Mutex::new(Some(lifetime)),
            cancellation_token,
        })
    }
}

pub fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    let app_state = async_runtime::block_on(AppState::new(workspace_path))
        .context("Failed to open workspace")?;

    let tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands::invoke_handler())
        .manage(app_state)
        .manage(LongDrop)
        .setup(|app| {
            install_ctrl_c_handler(app);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    tauri_app.run(|app, event| {
        if let RunEvent::Exit = event {
            graceful_shutdown(app);
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

struct LongDrop;

impl Drop for LongDrop {
    fn drop(&mut self) {
        std::thread::sleep(Duration::from_secs(2));
    }
}

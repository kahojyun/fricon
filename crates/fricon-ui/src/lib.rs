#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]
mod commands;

use std::{path::PathBuf, sync::Mutex};

use anyhow::{Context as _, Result};
use tauri::{Manager, RunEvent, async_runtime};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::error;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};

struct AppState {
    app: fricon::App,
    server_handle: Mutex<Option<JoinHandle<Result<()>>>>,
    cancellation_token: CancellationToken,
    log_guard: Mutex<Option<WorkerGuard>>,
}

impl AppState {
    async fn new(workspace_path: PathBuf, log_guard: WorkerGuard) -> Result<Self> {
        let app = fricon::App::open(&workspace_path).await?;
        let cancellation_token = CancellationToken::new();

        // Start gRPC server in background
        let server_app = app.clone();
        let server_token = cancellation_token.clone();
        let server_handle =
            tokio::spawn(async move { fricon::run_server(server_app, server_token).await });

        Ok(Self {
            app,
            server_handle: Mutex::new(Some(server_handle)),
            cancellation_token,
            log_guard: Mutex::new(Some(log_guard)),
        })
    }
}

pub fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    let log_dir = fricon::get_log_dir(workspace_path.clone())?;
    let rolling = RollingFileAppender::new(Rotation::DAILY, log_dir, "fricon.log");
    let (writer, guard) = tracing_appender::non_blocking(rolling);
    tracing_subscriber::fmt().with_writer(writer).init();

    let app_state = async_runtime::block_on(AppState::new(workspace_path, guard))
        .context("Failed to open workspace")?;

    let tauri_app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands::invoke_handler())
        .manage(app_state)
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    tauri_app.run(|app, event| {
        if let RunEvent::Exit = event {
            let state = app.state::<AppState>();
            // Signal server to shutdown
            state.cancellation_token.cancel();
            let handle = state.server_handle.lock().unwrap().take().unwrap();

            let result =
                async_runtime::block_on(
                    async move { handle.await.context("Server task panicked")? },
                );
            if let Err(e) = result {
                error!("Server returned an error: {e}");
            }
            let _log_guard = state.log_guard.lock().unwrap().take();
        }
    });

    Ok(())
}

pub fn run() {
    eprintln!("Error: GUI requires workspace path. Use CLI: fricon gui <workspace_path>");
    std::process::exit(1);
}

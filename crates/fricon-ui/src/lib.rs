#![allow(clippy::needless_pass_by_value, clippy::used_underscore_binding)]

use anyhow::Result;
use std::path::PathBuf;
use tauri::{Listener, Manager};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

mod commands;

struct AppState {
    app: fricon::App,
    server_handle: JoinHandle<Result<(), anyhow::Error>>,
    cancellation_token: CancellationToken,
}

impl AppState {
    async fn new(workspace_path: PathBuf) -> Result<Self> {
        let app = fricon::App::open(&workspace_path).await?;
        let cancellation_token = CancellationToken::new();

        // Start gRPC server in background
        let server_app = app.clone();
        let server_token = cancellation_token.clone();
        let server_handle = tokio::spawn(async move {
            fricon::server::run_with_app_and_cancellation(server_app, server_token).await
        });

        Ok(Self {
            app,
            server_handle,
            cancellation_token,
        })
    }

    async fn shutdown(self) -> Result<()> {
        info!("Shutting down application");
        // Signal server to shutdown
        self.cancellation_token.cancel();

        // Wait for server to finish gracefully
        self.server_handle.await?
    }
}

pub async fn run_with_workspace(workspace_path: PathBuf) -> Result<()> {
    let app_state = AppState::new(workspace_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize workspace: {}", e))?;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(commands::invoke_handler())
        .manage(app_state)
        .setup(|app| {
            // Register shutdown handler
            let app_handle = app.handle().clone();
            app.listen("tauri://close-requested", move |_| {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        // Can't clone AppState due to JoinHandle, so we just cancel the token
                        state.cancellation_token.cancel();
                    }
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

// Legacy function for backward compatibility
pub fn run() {
    eprintln!("Error: GUI requires workspace path. Use CLI: fricon gui <workspace_path>");
    std::process::exit(1);
}

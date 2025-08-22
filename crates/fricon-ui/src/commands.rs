use super::AppState;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{State, ipc::Invoke};

#[derive(Serialize)]
struct DatasetInfo {
    id: i32,
    name: String,
    description: String,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct WorkspaceInfo {
    path: String,
    is_ready: bool,
}

#[derive(Serialize)]
struct ServerStatus {
    is_running: bool,
    ipc_path: String,
}

#[tauri::command]
async fn get_workspace_info(state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let workspace_path = state.app.root().paths().root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
        is_ready: true,
    })
}

#[tauri::command]
async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    Ok(ServerStatus {
        is_running: !state.server_handle.is_finished(),
        ipc_path: state
            .app
            .root()
            .paths()
            .ipc_file()
            .to_string_lossy()
            .to_string(),
    })
}

#[tauri::command]
async fn create_dataset(
    state: State<'_, AppState>,
    name: String,
    description: String,
    tags: Vec<String>,
    index_columns: Vec<String>,
) -> Result<i32, String> {
    let writer = state
        .app
        .create_dataset(name, description, tags, index_columns)
        .await
        .map_err(|e| e.to_string())?;

    Ok(writer.id())
}

#[tauri::command]
async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<DatasetInfo>, String> {
    let datasets = state.app.list_datasets().await.map_err(|e| e.to_string())?;

    let dataset_info: Vec<DatasetInfo> = datasets
        .into_iter()
        .map(|(dataset, tags)| DatasetInfo {
            id: dataset.id,
            name: dataset.name,
            description: dataset.description,
            tags: tags.into_iter().map(|t| t.name).collect(),
            created_at: dataset.created_at.and_utc(),
        })
        .collect();

    Ok(dataset_info)
}

#[tauri::command]
async fn shutdown_server(state: State<'_, AppState>) -> Result<(), String> {
    state.cancellation_token.cancel();
    Ok(())
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![
        get_workspace_info,
        get_server_status,
        create_dataset,
        list_datasets,
        shutdown_server
    ]
}

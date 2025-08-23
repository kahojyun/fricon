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
    let app = state.app();
    let workspace_path = app.root().paths().root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
        is_ready: true,
    })
}

#[tauri::command]
fn get_server_status(state: State<'_, AppState>) -> ServerStatus {
    ServerStatus {
        is_running: true,
        ipc_path: state
            .app()
            .root()
            .paths()
            .ipc_file()
            .to_string_lossy()
            .to_string(),
    }
}

#[tauri::command]
async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<DatasetInfo>, String> {
    let app = state.app();
    let datasets = app.list_datasets().await.map_err(|e| e.to_string())?;

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

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![get_workspace_info, get_server_status, list_datasets]
}

#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{State, ipc::Invoke};

use super::AppState;

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
}

#[tauri::command]
async fn get_workspace_info(state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let app = state.app();
    let workspace_paths = app
        .paths()
        .map_err(|e| format!("Failed to get paths: {e}"))?;
    let workspace_path = workspace_paths.root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<DatasetInfo>, String> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    let datasets = dataset_manager
        .list_datasets()
        .await
        .map_err(|e| e.to_string())?;

    let dataset_info: Vec<DatasetInfo> = datasets
        .into_iter()
        .map(|record| DatasetInfo {
            id: record.id,
            name: record.metadata.name,
            description: record.metadata.description,
            tags: record.metadata.tags,
            created_at: record.metadata.created_at,
        })
        .collect();

    Ok(dataset_info)
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![get_workspace_info, list_datasets]
}

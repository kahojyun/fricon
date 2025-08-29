use super::AppState;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{Emitter, State, Window, ipc::Invoke};

use crate::chart_config::{ChartConfigurationManager, CompleteChartConfiguration};
use crate::chart_service::{ChartDataRequest, ChartSchemaResponse, EChartsDataResponse};

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

#[tauri::command]
async fn get_chart_schema(
    state: State<'_, AppState>,
    dataset_id: i32,
) -> Result<ChartSchemaResponse, String> {
    // Use the local UI chart service
    state
        .chart_service()
        .get_schema(dataset_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_chart_data(
    state: State<'_, AppState>,
    request: ChartDataRequest,
) -> Result<EChartsDataResponse, String> {
    // Use the local UI chart service with automatic data source detection
    state
        .chart_service()
        .get_data(request)
        .await
        .map_err(|e| e.to_string())
}

/// Subscribe to live chart updates for real-time visualization
#[tauri::command]
async fn subscribe_live_chart_updates(
    state: State<'_, AppState>,
    dataset_id: i32,
    window: Window,
) -> Result<(), String> {
    // Try to subscribe to updates for the dataset using local UI chart service
    let mut rx = state
        .chart_service()
        .subscribe_updates(dataset_id)
        .map_err(|e| e.to_string())?;

    // Spawn a task to forward updates to the frontend
    tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            // Emit the chart update event to the frontend
            if let Err(e) = window.emit("chart-update", &update) {
                tracing::warn!("Failed to emit chart update: {}", e);
                break;
            }
        }
    });

    Ok(())
}

/// Save chart configuration for a dataset
#[tauri::command]
async fn save_chart_config(
    state: State<'_, AppState>,
    dataset_id: i32,
    config: CompleteChartConfiguration,
) -> Result<(), String> {
    let app = state.app();
    let config_manager = ChartConfigurationManager::new(app);

    config_manager
        .save_config(dataset_id, &config)
        .await
        .map_err(|e| e.to_string())
}

/// Load chart configuration for a dataset
#[tauri::command]
async fn load_chart_config(
    state: State<'_, AppState>,
    dataset_id: i32,
) -> Result<CompleteChartConfiguration, String> {
    let app = state.app();
    let config_manager = ChartConfigurationManager::new(app);

    config_manager
        .load_config_with_defaults(dataset_id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete chart configuration for a dataset
#[tauri::command]
async fn delete_chart_config(state: State<'_, AppState>, dataset_id: i32) -> Result<(), String> {
    let app = state.app();
    let config_manager = ChartConfigurationManager::new(app);

    config_manager
        .delete_config(dataset_id)
        .await
        .map_err(|e| e.to_string())
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![
        get_workspace_info,
        get_server_status,
        list_datasets,
        get_chart_schema,
        get_chart_data,
        subscribe_live_chart_updates,
        save_chart_config,
        load_chart_config,
        delete_chart_config
    ]
}

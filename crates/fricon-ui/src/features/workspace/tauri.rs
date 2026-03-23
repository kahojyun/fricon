use anyhow::Context;
use tauri::State;

use crate::{desktop_runtime::app_state::AppState, tauri_api::ApiError};

#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceInfo {
    pub(crate) path: String,
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_workspace_info(
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, ApiError> {
    let workspace_paths = state
        .session()
        .app()
        .paths()
        .context("Failed to retrieve workspace paths.")
        .map_err(ApiError::workspace)?;
    Ok(WorkspaceInfo {
        path: workspace_paths.root().to_string_lossy().to_string(),
    })
}

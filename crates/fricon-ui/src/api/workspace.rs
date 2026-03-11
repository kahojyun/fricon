use tauri::State;

use super::TauriCommandError;
use crate::{application::workspace as app_workspace, desktop_runtime::app_state::AppState};

#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceInfo {
    pub(crate) path: String,
}

impl From<app_workspace::WorkspaceInfo> for WorkspaceInfo {
    fn from(value: app_workspace::WorkspaceInfo) -> Self {
        Self {
            path: value.path.to_string_lossy().to_string(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_workspace_info(
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, TauriCommandError> {
    Ok(app_workspace::get_workspace_info(state.session())?.into())
}

use anyhow::Context;
use tauri::State;

use super::{AppState, TauriCommandError};
use crate::filter_data::{TableData, load_filter_data};

#[derive(serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilterTableOptions {
    #[specta(optional)]
    exclude_columns: Option<Vec<String>>,
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, TauriCommandError> {
    load_filter_data(&state, id, options.exclude_columns)
        .await
        .context("Failed to load filter table data")
        .map(TableData::from)
        .map_err(TauriCommandError::from)
}

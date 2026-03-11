use tauri::State;

use super::{AppState, TauriCommandError};
use crate::{
    application,
    tauri_api::filter_table::{FilterTableOptions, TableData},
};

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, TauriCommandError> {
    application::filter_table::get_filter_table_data(state.session(), id, options.exclude_columns)
        .await
        .map(Into::into)
        .map_err(TauriCommandError::from)
}

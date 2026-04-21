use tauri::{State, ipc::Response};
use tracing::instrument;

use crate::{
    desktop_runtime::app_state::AppState,
    features::charts::{
        chart_data, filter_table,
        types::{DatasetChartDataOptions, FilterTableOptions, LiveChartDataOptions, TableData},
        wire,
    },
    tauri_api::ApiError,
};

#[tauri::command]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetChartDataOptions,
) -> Result<Response, ApiError> {
    let snapshot = chart_data::dataset_chart_data(state.session(), id, &options)
        .await
        .map_err(ApiError::charts)?;
    wire::encode_chart_snapshot(&snapshot)
        .map(Response::new)
        .map_err(ApiError::charts)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, ApiError> {
    filter_table::get_filter_table_data(state.session(), id, options.exclude_columns)
        .await
        .map_err(ApiError::charts)
}

#[tauri::command]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_live_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: LiveChartDataOptions,
) -> Result<Response, ApiError> {
    let response = chart_data::dataset_live_chart_data(state.session(), id, &options)
        .await
        .map_err(ApiError::charts)?;
    wire::encode_live_chart_data(&response)
        .map(Response::new)
        .map_err(ApiError::charts)
}

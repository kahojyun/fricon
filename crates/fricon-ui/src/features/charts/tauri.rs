use tauri::State;
use tracing::instrument;

use crate::{
    desktop_runtime::app_state::AppState,
    features::charts::{
        chart_data, filter_table,
        types::{
            ChartSnapshot, DatasetChartDataOptions, FilterTableOptions, LiveChartDataOptions,
            LiveChartDataResponse, TableData,
        },
    },
    tauri_api::ApiError,
};

#[tauri::command]
#[specta::specta]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetChartDataOptions,
) -> Result<ChartSnapshot, ApiError> {
    chart_data::dataset_chart_data(state.session(), id, &options)
        .await
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
#[specta::specta]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_live_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: LiveChartDataOptions,
) -> Result<LiveChartDataResponse, ApiError> {
    chart_data::dataset_live_chart_data(state.session(), id, &options)
        .await
        .map_err(ApiError::charts)
}

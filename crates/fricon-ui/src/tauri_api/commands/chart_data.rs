use tauri::State;
use tracing::instrument;

use super::{AppState, TauriCommandError};
use crate::{
    application,
    tauri_api::chart_data::{ChartDataResponse, DatasetChartDataOptions},
};

#[tauri::command]
#[specta::specta]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetChartDataOptions,
) -> Result<ChartDataResponse, TauriCommandError> {
    let result =
        application::chart_data::dataset_chart_data(state.session(), id, &options.into()).await?;
    Ok(result.into())
}

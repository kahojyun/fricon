use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::instrument;

use super::TauriCommandError;
use crate::{application::charts as app, chart_data, desktop_runtime::app_state::AppState};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartType {
    Line,
    Heatmap,
    Scatter,
}

impl From<chart_data::ChartType> for ChartType {
    fn from(value: chart_data::ChartType) -> Self {
        match value {
            chart_data::ChartType::Line => Self::Line,
            chart_data::ChartType::Heatmap => Self::Heatmap,
            chart_data::ChartType::Scatter => Self::Scatter,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

impl From<ComplexViewOption> for chart_data::ComplexViewOption {
    fn from(value: ComplexViewOption) -> Self {
        match value {
            ComplexViewOption::Real => Self::Real,
            ComplexViewOption::Imag => Self::Imag,
            ComplexViewOption::Mag => Self::Mag,
            ComplexViewOption::Arg => Self::Arg,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartCommonOptions {
    pub(crate) start: Option<usize>,
    pub(crate) end: Option<usize>,
    pub(crate) index_filters: Option<Vec<usize>>,
    pub(crate) exclude_columns: Option<Vec<String>>,
}

impl From<ChartCommonOptions> for chart_data::ChartCommonOptions {
    fn from(value: ChartCommonOptions) -> Self {
        Self {
            start: value.start,
            end: value.end,
            index_filters: value.index_filters,
            exclude_columns: value.exclude_columns,
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LineChartDataOptions {
    pub(crate) series: String,
    pub(crate) x_column: Option<String>,
    pub(crate) complex_views: Option<Vec<ComplexViewOption>>,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

impl From<LineChartDataOptions> for chart_data::LineChartDataOptions {
    fn from(value: LineChartDataOptions) -> Self {
        Self {
            series: value.series,
            x_column: value.x_column,
            complex_views: value
                .complex_views
                .map(|views| views.into_iter().map(Into::into).collect()),
            common: value.common.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeatmapChartDataOptions {
    pub(crate) series: String,
    pub(crate) x_column: Option<String>,
    pub(crate) y_column: String,
    pub(crate) complex_view_single: Option<ComplexViewOption>,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

impl From<HeatmapChartDataOptions> for chart_data::HeatmapChartDataOptions {
    fn from(value: HeatmapChartDataOptions) -> Self {
        Self {
            series: value.series,
            x_column: value.x_column,
            y_column: value.y_column,
            complex_view_single: value.complex_view_single.map(Into::into),
            common: value.common.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub(crate) enum ScatterModeOptions {
    Complex {
        series: String,
    },
    TraceXy {
        #[serde(rename = "traceXColumn")]
        trace_x_column: String,
        #[serde(rename = "traceYColumn")]
        trace_y_column: String,
    },
    Xy {
        #[serde(rename = "xColumn")]
        x_column: String,
        #[serde(rename = "yColumn")]
        y_column: String,
        #[serde(rename = "binColumn")]
        bin_column: Option<String>,
    },
}

impl From<ScatterModeOptions> for chart_data::ScatterModeOptions {
    fn from(value: ScatterModeOptions) -> Self {
        match value {
            ScatterModeOptions::Complex { series } => Self::Complex { series },
            ScatterModeOptions::TraceXy {
                trace_x_column,
                trace_y_column,
            } => Self::TraceXy {
                trace_x_column,
                trace_y_column,
            },
            ScatterModeOptions::Xy {
                x_column,
                y_column,
                bin_column,
            } => Self::Xy {
                x_column,
                y_column,
                bin_column,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScatterChartDataOptions {
    pub(crate) scatter: ScatterModeOptions,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

impl From<ScatterChartDataOptions> for chart_data::ScatterChartDataOptions {
    fn from(value: ScatterChartDataOptions) -> Self {
        Self {
            scatter: value.scatter.into(),
            common: value.common.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "chartType", rename_all = "snake_case")]
pub(crate) enum DatasetChartDataOptions {
    Line(LineChartDataOptions),
    Heatmap(HeatmapChartDataOptions),
    Scatter(ScatterChartDataOptions),
}

impl From<DatasetChartDataOptions> for chart_data::DatasetChartDataOptions {
    fn from(value: DatasetChartDataOptions) -> Self {
        match value {
            DatasetChartDataOptions::Line(options) => Self::Line(options.into()),
            DatasetChartDataOptions::Heatmap(options) => Self::Heatmap(options.into()),
            DatasetChartDataOptions::Scatter(options) => Self::Scatter(options.into()),
        }
    }
}

#[derive(Serialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Series {
    pub(crate) name: String,
    pub(crate) data: Vec<Vec<f64>>,
}

impl From<chart_data::Series> for Series {
    fn from(value: chart_data::Series) -> Self {
        Self {
            name: value.name,
            data: value.data,
        }
    }
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartDataResponse {
    pub(crate) r#type: ChartType,
    pub(crate) x_name: String,
    pub(crate) y_name: Option<String>,
    pub(crate) x_categories: Option<Vec<f64>>,
    pub(crate) y_categories: Option<Vec<f64>>,
    pub(crate) series: Vec<Series>,
}

impl From<chart_data::ChartDataResponse> for ChartDataResponse {
    fn from(value: chart_data::ChartDataResponse) -> Self {
        Self {
            r#type: value.r#type.into(),
            x_name: value.x_name,
            y_name: value.y_name,
            x_categories: value.x_categories,
            y_categories: value.y_categories,
            series: value.series.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Row {
    pub(crate) display_values: Vec<String>,
    pub(crate) value_indices: Vec<usize>,
    pub(crate) index: usize,
}

impl From<app::Row> for Row {
    fn from(value: app::Row) -> Self {
        Self {
            display_values: value.display_values,
            value_indices: value.value_indices,
            index: value.index,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnUniqueValue {
    pub(crate) index: usize,
    pub(crate) display_value: String,
}

impl From<app::ColumnUniqueValue> for ColumnUniqueValue {
    fn from(value: app::ColumnUniqueValue) -> Self {
        Self {
            index: value.index,
            display_value: value.display_value,
        }
    }
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TableData {
    pub(crate) fields: Vec<String>,
    pub(crate) rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

impl From<app::TableData> for TableData {
    fn from(value: app::TableData) -> Self {
        Self {
            fields: value.fields,
            rows: value.rows.into_iter().map(Into::into).collect(),
            column_unique_values: value
                .column_unique_values
                .into_iter()
                .map(|(key, values)| (key, values.into_iter().map(Into::into).collect()))
                .collect(),
        }
    }
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilterTableOptions {
    #[specta(optional)]
    pub(crate) exclude_columns: Option<Vec<String>>,
}

#[tauri::command]
#[specta::specta]
#[instrument(level = "debug", skip(state, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetChartDataOptions,
) -> Result<ChartDataResponse, TauriCommandError> {
    let result = app::dataset_chart_data(state.session(), id, &options.into()).await?;
    Ok(result.into())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, TauriCommandError> {
    app::get_filter_table_data(state.session(), id, options.exclude_columns)
        .await
        .map(Into::into)
        .map_err(TauriCommandError::from)
}

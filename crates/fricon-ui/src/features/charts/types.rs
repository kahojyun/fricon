use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum XYPlotMode {
    QuantityVsSweep,
    Xy,
    ComplexPlane,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum XYDrawStyle {
    Line,
    Points,
    LinePoints,
}

impl XYDrawStyle {
    pub(crate) const fn includes_lines(self) -> bool {
        matches!(self, Self::Line | Self::LinePoints)
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

#[derive(Debug, Clone, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct XYTraceRoleOptions {
    #[specta(optional)]
    pub(crate) trace_group_index_columns: Option<Vec<String>>,
    #[specta(optional)]
    pub(crate) sweep_index_column: Option<String>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "plotMode", rename_all = "snake_case")]
pub(crate) enum XYPlotModeOptions {
    QuantityVsSweep {
        quantity: String,
        complex_views: Option<Vec<ComplexViewOption>>,
    },
    Xy {
        #[serde(rename = "xColumn")]
        x_column: String,
        #[serde(rename = "yColumn")]
        y_column: String,
    },
    ComplexPlane {
        quantity: String,
    },
}

impl XYPlotModeOptions {
    pub(crate) const fn plot_mode(&self) -> XYPlotMode {
        match self {
            Self::QuantityVsSweep { .. } => XYPlotMode::QuantityVsSweep,
            Self::Xy { .. } => XYPlotMode::Xy,
            Self::ComplexPlane { .. } => XYPlotMode::ComplexPlane,
        }
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct XYChartDataOptions {
    pub(crate) draw_style: XYDrawStyle,
    #[serde(flatten)]
    pub(crate) plot_mode: XYPlotModeOptions,
    #[serde(flatten)]
    pub(crate) trace_roles: XYTraceRoleOptions,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeatmapChartDataOptions {
    pub(crate) quantity: String,
    pub(crate) x_column: Option<String>,
    pub(crate) y_column: String,
    pub(crate) complex_view_single: Option<ComplexViewOption>,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "view", rename_all = "snake_case")]
pub(crate) enum DatasetChartDataOptions {
    Xy(XYChartDataOptions),
    Heatmap(HeatmapChartDataOptions),
}

impl DatasetChartDataOptions {
    pub(crate) fn common(&self) -> &ChartCommonOptions {
        match self {
            Self::Xy(options) => &options.common,
            Self::Heatmap(options) => &options.common,
        }
    }

    pub(crate) const fn view_name(&self) -> &'static str {
        match self {
            Self::Xy(_) => "xy",
            Self::Heatmap(_) => "heatmap",
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FlatXYSeries {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) values: Vec<f64>,
    pub(crate) point_count: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FlatXYZSeries {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) values: Vec<f64>,
    pub(crate) point_count: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct XYChartSnapshot {
    pub(crate) plot_mode: XYPlotMode,
    pub(crate) draw_style: XYDrawStyle,
    pub(crate) x_name: String,
    pub(crate) y_name: Option<String>,
    pub(crate) series: Vec<FlatXYSeries>,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeatmapChartSnapshot {
    pub(crate) x_name: String,
    pub(crate) y_name: String,
    pub(crate) series: Vec<FlatXYZSeries>,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ChartSnapshot {
    Xy(XYChartSnapshot),
    Heatmap(HeatmapChartSnapshot),
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub(crate) enum FlatSeries {
    Xy(FlatXYSeries),
    Xyz(FlatXYZSeries),
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LiveChartAppendOperation {
    AppendPoints {
        series_id: String,
        values: Vec<f64>,
        point_count: usize,
    },
    AppendSeries {
        series: FlatSeries,
    },
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub(crate) enum LiveChartDataResponse {
    Reset {
        row_count: usize,
        snapshot: ChartSnapshot,
    },
    Append {
        row_count: usize,
        ops: Vec<LiveChartAppendOperation>,
    },
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Row {
    pub(crate) display_values: Vec<String>,
    pub(crate) value_indices: Vec<usize>,
    pub(crate) index: usize,
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnUniqueValue {
    pub(crate) index: usize,
    pub(crate) display_value: String,
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TableData {
    pub(crate) fields: Vec<String>,
    pub(crate) rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilterTableOptions {
    #[specta(optional)]
    pub(crate) exclude_columns: Option<Vec<String>>,
}

pub(crate) fn transform_complex_values(
    reals: &[f64],
    imags: &[f64],
    option: ComplexViewOption,
) -> Vec<f64> {
    match option {
        ComplexViewOption::Real => reals.to_vec(),
        ComplexViewOption::Imag => imags.to_vec(),
        ComplexViewOption::Mag => reals
            .iter()
            .zip(imags)
            .map(|(re, im)| (re * re + im * im).sqrt())
            .collect(),
        ComplexViewOption::Arg => reals
            .iter()
            .zip(imags)
            .map(|(re, im)| im.atan2(*re))
            .collect(),
    }
}

pub(crate) fn complex_view_label(option: ComplexViewOption) -> &'static str {
    match option {
        ComplexViewOption::Real => "real",
        ComplexViewOption::Imag => "imag",
        ComplexViewOption::Mag => "mag",
        ComplexViewOption::Arg => "arg",
    }
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveXYOptions {
    pub(crate) draw_style: XYDrawStyle,
    pub(crate) tail_count: usize,
    #[specta(optional)]
    pub(crate) known_row_count: Option<usize>,
    #[serde(flatten)]
    pub(crate) plot_mode: XYPlotModeOptions,
    #[serde(flatten)]
    pub(crate) trace_roles: XYTraceRoleOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveHeatmapOptions {
    pub(crate) quantity: String,
    pub(crate) complex_view_single: Option<ComplexViewOption>,
    #[specta(optional)]
    pub(crate) known_row_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "view", rename_all = "snake_case")]
pub(crate) enum LiveChartDataOptions {
    Xy(LiveXYOptions),
    Heatmap(LiveHeatmapOptions),
}

impl LiveChartDataOptions {
    pub(crate) const fn known_row_count(&self) -> Option<usize> {
        match self {
            Self::Xy(options) => options.known_row_count,
            Self::Heatmap(options) => options.known_row_count,
        }
    }
}

impl FlatXYSeries {
    pub(crate) fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        values: Vec<f64>,
        point_count: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            values,
            point_count,
        }
    }
}

impl FlatXYZSeries {
    pub(crate) fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        values: Vec<f64>,
        point_count: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            values,
            point_count,
        }
    }
}

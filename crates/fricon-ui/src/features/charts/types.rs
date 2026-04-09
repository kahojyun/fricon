use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

#[derive(Debug, Clone, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartCommonOptions {
    pub(crate) start: Option<usize>,
    pub(crate) end: Option<usize>,
    pub(crate) index_filters: Option<Vec<usize>>,
    pub(crate) exclude_columns: Option<Vec<String>>,
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
    },
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScatterChartDataOptions {
    pub(crate) scatter: ScatterModeOptions,
    #[serde(flatten)]
    pub(crate) common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "chartType", rename_all = "snake_case")]
pub(crate) enum DatasetChartDataOptions {
    Line(LineChartDataOptions),
    Heatmap(HeatmapChartDataOptions),
    Scatter(ScatterChartDataOptions),
}

impl DatasetChartDataOptions {
    pub(crate) fn common(&self) -> &ChartCommonOptions {
        match self {
            Self::Line(options) => &options.common,
            Self::Heatmap(options) => &options.common,
            Self::Scatter(options) => &options.common,
        }
    }

    pub(crate) const fn chart_type_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Heatmap(_) => "heatmap",
            Self::Scatter(_) => "scatter",
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
pub(crate) struct LineChartSnapshot {
    pub(crate) x_name: String,
    pub(crate) series: Vec<FlatXYSeries>,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScatterChartSnapshot {
    pub(crate) x_name: String,
    pub(crate) y_name: String,
    pub(crate) series: Vec<FlatXYSeries>,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeatmapChartSnapshot {
    pub(crate) x_name: String,
    pub(crate) y_name: String,
    pub(crate) x_categories: Vec<f64>,
    pub(crate) y_categories: Vec<f64>,
    pub(crate) series: Vec<FlatXYZSeries>,
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ChartSnapshot {
    Line(LineChartSnapshot),
    Heatmap(HeatmapChartSnapshot),
    Scatter(ScatterChartSnapshot),
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub(crate) enum FlatSeries {
    Xy(FlatXYSeries),
    Xyz(FlatXYZSeries),
}

#[derive(Serialize, Clone, Debug, PartialEq, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[expect(
    clippy::enum_variant_names,
    reason = "Wire format uses append-prefixed operation names"
)]
pub(crate) enum LiveChartAppendOperation {
    AppendPoints {
        series_id: String,
        values: Vec<f64>,
        point_count: usize,
    },
    AppendSeries {
        series: FlatSeries,
    },
    AppendHeatmapCategories {
        x_categories: Option<Vec<f64>>,
        y_categories: Option<Vec<f64>>,
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
pub(crate) struct LiveLineOptions {
    pub(crate) series: String,
    pub(crate) complex_views: Option<Vec<ComplexViewOption>>,
    pub(crate) tail_count: usize,
    #[specta(optional)]
    pub(crate) known_row_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveHeatmapOptions {
    pub(crate) series: String,
    pub(crate) complex_view_single: Option<ComplexViewOption>,
    #[specta(optional)]
    pub(crate) known_row_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveScatterOptions {
    pub(crate) scatter: ScatterModeOptions,
    pub(crate) tail_count: usize,
    #[specta(optional)]
    pub(crate) known_row_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "chartType", rename_all = "snake_case")]
pub(crate) enum LiveChartDataOptions {
    Line(LiveLineOptions),
    Heatmap(LiveHeatmapOptions),
    Scatter(LiveScatterOptions),
}

impl LiveChartDataOptions {
    pub(crate) const fn known_row_count(&self) -> Option<usize> {
        match self {
            Self::Line(options) => options.known_row_count,
            Self::Heatmap(options) => options.known_row_count,
            Self::Scatter(options) => options.known_row_count,
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

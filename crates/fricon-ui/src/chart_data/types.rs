use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartType {
    Line,
    Heatmap,
    Scatter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartCommonOptions {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub index_filters: Option<Vec<usize>>,
    pub exclude_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LineChartDataOptions {
    pub series: String,
    pub x_column: Option<String>,
    pub complex_views: Option<Vec<ComplexViewOption>>,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeatmapChartDataOptions {
    pub series: String,
    pub x_column: Option<String>,
    pub y_column: String,
    pub complex_view_single: Option<ComplexViewOption>,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScatterChartDataOptions {
    pub scatter: ScatterModeOptions,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Series {
    pub name: String,
    pub data: Vec<Vec<f64>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartDataResponse {
    pub r#type: ChartType,
    pub x_name: String,
    pub y_name: Option<String>,
    pub x_categories: Option<Vec<f64>>,
    pub y_categories: Option<Vec<f64>>,
    pub series: Vec<Series>,
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

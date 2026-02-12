use std::collections::HashMap;

use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Line,
    Heatmap,
    Scatter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, specta::Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

#[derive(Debug, Clone, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartCommonOptions {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub index_filters: Option<Vec<usize>>,
    pub exclude_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LineChartDataOptions {
    pub series: String,
    pub x_column: Option<String>,
    pub complex_views: Option<Vec<ComplexViewOption>>,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapChartDataOptions {
    pub series: String,
    pub x_column: Option<String>,
    pub y_column: String,
    pub complex_view_single: Option<ComplexViewOption>,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ScatterModeOptions {
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

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScatterChartDataOptions {
    pub scatter: ScatterModeOptions,
    #[serde(flatten)]
    pub common: ChartCommonOptions,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(tag = "chartType", rename_all = "snake_case")]
pub enum DatasetChartDataOptions {
    Line(LineChartDataOptions),
    Heatmap(HeatmapChartDataOptions),
    Scatter(ScatterChartDataOptions),
}

impl DatasetChartDataOptions {
    pub fn common(&self) -> &ChartCommonOptions {
        match self {
            Self::Line(options) => &options.common,
            Self::Heatmap(options) => &options.common,
            Self::Scatter(options) => &options.common,
        }
    }
}

#[derive(Serialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub name: String,
    pub data: Vec<Vec<f64>>,
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DataResponse {
    pub r#type: Type,
    pub x_name: String,
    pub y_name: Option<String>,
    pub x_categories: Option<Vec<f64>>,
    pub y_categories: Option<Vec<f64>>,
    pub series: Vec<Series>,
}

pub fn transform_complex_values(
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

pub fn complex_view_label(option: ComplexViewOption) -> &'static str {
    match option {
        ComplexViewOption::Real => "real",
        ComplexViewOption::Imag => "imag",
        ComplexViewOption::Mag => "mag",
        ComplexViewOption::Arg => "arg",
    }
}

pub fn build_line_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &LineChartDataOptions,
) -> Result<DataResponse> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;

    let (x_values, y_values_array) = if is_trace {
        let Some(values) = series_array.expand_trace(0)? else {
            return Ok(DataResponse {
                r#type: Type::Line,
                x_name,
                y_name: None,
                x_categories: None,
                y_categories: None,
                series: vec![],
            });
        };
        values
    } else {
        let x_column = options
            .x_column
            .as_ref()
            .context("Line chart requires x column")?;
        let x_array = batch
            .column_by_name(x_column)
            .cloned()
            .context("X column not found")?;
        let ds_x: DatasetArray = x_array.try_into()?;
        let x_vals = ds_x
            .as_numeric()
            .context("X must be numeric")?
            .values()
            .to_vec();
        (
            x_vals,
            batch
                .column_by_name(series_name)
                .cloned()
                .context("Column not found")?,
        )
    };

    if is_trace && x_values.is_empty() {
        return Ok(DataResponse {
            r#type: Type::Line,
            x_name,
            y_name: None,
            x_categories: None,
            y_categories: None,
            series: vec![],
        });
    }

    let series = if is_complex {
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let complex_array = ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();

        let view_options = options
            .complex_views
            .clone()
            .unwrap_or_else(|| vec![ComplexViewOption::Real, ComplexViewOption::Imag]);
        view_options
            .into_iter()
            .map(|option| {
                let y_values = transform_complex_values(reals, imags, option);
                let len = x_values.len().min(y_values.len());
                let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
                Series {
                    name: format!("{series_name} ({})", complex_view_label(option)),
                    data,
                }
            })
            .collect()
    } else {
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = x_values.len().min(y_values.len());
        vec![Series {
            name: series_name.clone(),
            data: (0..len).map(|i| vec![x_values[i], y_values[i]]).collect(),
        }]
    };

    Ok(DataResponse {
        r#type: Type::Line,
        x_name,
        y_name: None,
        x_categories: None,
        y_categories: None,
        series,
    })
}

pub fn build_heatmap_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &HeatmapChartDataOptions,
) -> Result<DataResponse> {
    let series_name = &options.series;
    let y_column = &options.y_column;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let view_option = options
        .complex_view_single
        .unwrap_or(ComplexViewOption::Mag);

    let mut series = if is_trace {
        process_trace_heatmap(
            batch,
            series_name,
            y_column,
            &series_array,
            is_complex,
            view_option,
        )?
    } else {
        let x_column = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        process_scalar_heatmap(
            batch,
            series_name,
            x_column,
            y_column,
            &series_array,
            is_complex,
            view_option,
        )?
    };

    let (x_categories, y_categories) = normalize_heatmap_series(&mut series);

    Ok(DataResponse {
        r#type: Type::Heatmap,
        x_name,
        y_name: Some(y_column.clone()),
        x_categories: Some(x_categories),
        y_categories: Some(y_categories),
        series,
    })
}

fn normalize_heatmap_series(series: &mut [Series]) -> (Vec<f64>, Vec<f64>) {
    fn f64_key(value: f64) -> u64 {
        if value == 0.0 { 0_u64 } else { value.to_bits() }
    }

    let mut x_categories: Vec<f64> = Vec::new();
    let mut y_categories: Vec<f64> = Vec::new();
    let mut x_index_by_value: HashMap<u64, usize> = HashMap::new();
    let mut y_index_by_value: HashMap<u64, usize> = HashMap::new();

    for item in series.iter_mut() {
        for point in item.data.iter_mut() {
            if point.len() < 3 {
                continue;
            }
            let x_value = point[0];
            let y_value = point[1];

            let x_index = if let Some(index) = x_index_by_value.get(&f64_key(x_value)) {
                *index
            } else {
                let index = x_categories.len();
                x_categories.push(x_value);
                x_index_by_value.insert(f64_key(x_value), index);
                index
            };

            let y_index = if let Some(index) = y_index_by_value.get(&f64_key(y_value)) {
                *index
            } else {
                let index = y_categories.len();
                y_categories.push(y_value);
                y_index_by_value.insert(f64_key(y_value), index);
                index
            };

            point[0] = x_index as f64;
            point[1] = y_index as f64;
        }
    }

    (x_categories, y_categories)
}

fn process_trace_heatmap(
    batch: &RecordBatch,
    series_name: &str,
    y_column: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    view_option: ComplexViewOption,
) -> Result<Vec<Series>> {
    let y_array = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y column not found")?;
    let ds_y: DatasetArray = y_array.try_into()?;
    let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();
    let mut data = Vec::new();
    for row in 0..batch.num_rows() {
        let Some((x_values, trace_values)) = series_array.expand_trace(row)? else {
            continue;
        };
        let y_value = *y_values.get(row).unwrap_or(&0.0);
        let ds_trace: DatasetArray = trace_values.try_into()?;
        if is_complex {
            let complex_array = ds_trace.as_complex().context("Expected complex array")?;
            let z_values = transform_complex_values(
                complex_array.real().values(),
                complex_array.imag().values(),
                view_option,
            );
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                data.push(vec![x_values[i], y_value, z_values[i]]);
            }
        } else {
            let z_values = ds_trace
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                data.push(vec![x_values[i], y_value, z_values[i]]);
            }
        }
    }
    let name = if is_complex {
        format!("{series_name} ({})", complex_view_label(view_option))
    } else {
        series_name.to_string()
    };
    Ok(vec![Series { name, data }])
}

fn process_scalar_heatmap(
    batch: &RecordBatch,
    series_name: &str,
    x_column: &str,
    y_column: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    view_option: ComplexViewOption,
) -> Result<Vec<Series>> {
    let x_array = batch
        .column_by_name(x_column)
        .cloned()
        .context("X not found")?;
    let y_array = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y not found")?;
    let ds_x: DatasetArray = x_array.try_into()?;
    let ds_y: DatasetArray = y_array.try_into()?;
    let x_values = ds_x.as_numeric().context("X must be numeric")?.values();
    let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();

    let data = if is_complex {
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let z_values = transform_complex_values(
            complex_array.real().values(),
            complex_array.imag().values(),
            view_option,
        );
        let len = x_values.len().min(y_values.len()).min(z_values.len());
        (0..len)
            .map(|i| vec![x_values[i], y_values[i], z_values[i]])
            .collect()
    } else {
        let z_values = series_array
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = x_values.len().min(y_values.len()).min(z_values.len());
        (0..len)
            .map(|i| vec![x_values[i], y_values[i], z_values[i]])
            .collect()
    };
    let name = if is_complex {
        format!("{series_name} ({})", complex_view_label(view_option))
    } else {
        series_name.to_string()
    };
    Ok(vec![Series { name, data }])
}

pub fn build_scatter_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ScatterChartDataOptions,
) -> Result<DataResponse> {
    let (x_name, y_name, series) = match &options.scatter {
        ScatterModeOptions::Complex { series } => process_complex_scatter(batch, schema, series)?,
        ScatterModeOptions::TraceXy {
            trace_x_column,
            trace_y_column,
        } => process_trace_xy_scatter(batch, trace_x_column, trace_y_column)?,
        ScatterModeOptions::Xy {
            x_column, y_column, ..
        } => process_xy_scatter(batch, x_column, y_column)?,
    };

    Ok(DataResponse {
        r#type: Type::Scatter,
        x_name,
        y_name: Some(y_name),
        x_categories: None,
        y_categories: None,
        series,
    })
}

fn process_complex_scatter(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
) -> Result<(String, String, Vec<Series>)> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let mut data = Vec::new();
    if is_trace {
        for row in 0..batch.num_rows() {
            let Some((_x_values, trace_values)) = series_array.expand_trace(row)? else {
                continue;
            };
            let ds_trace: DatasetArray = trace_values.try_into()?;
            let complex_array = ds_trace.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            let len = reals.len().min(imags.len());
            for i in 0..len {
                data.push(vec![reals[i], imags[i]]);
            }
        }
    } else {
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        let len = reals.len().min(imags.len());
        for i in 0..len {
            data.push(vec![reals[i], imags[i]]);
        }
    }
    Ok((
        format!("{series_name} (real)"),
        format!("{series_name} (imag)"),
        vec![Series {
            name: series_name.to_string(),
            data,
        }],
    ))
}

fn process_trace_xy_scatter(
    batch: &RecordBatch,
    trace_x: &str,
    trace_y: &str,
) -> Result<(String, String, Vec<Series>)> {
    let x_array: DatasetArray = batch
        .column_by_name(trace_x)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(trace_y)
        .cloned()
        .context("Y not found")?
        .try_into()?;

    let mut data = Vec::new();
    for row in 0..batch.num_rows() {
        let Some((_x_axis, x_values_array)) = x_array.expand_trace(row)? else {
            continue;
        };
        let Some((_y_axis, y_values_array)) = y_array.expand_trace(row)? else {
            continue;
        };
        let ds_x: DatasetArray = x_values_array.try_into()?;
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let x_values = ds_x.as_numeric().context("X must be numeric")?.values();
        let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();
        let len = x_values.len().min(y_values.len());
        for i in 0..len {
            data.push(vec![x_values[i], y_values[i]]);
        }
    }
    let series_name = format!("{trace_x} vs {trace_y}");
    Ok((
        trace_x.to_string(),
        trace_y.to_string(),
        vec![Series {
            name: series_name,
            data,
        }],
    ))
}

fn process_xy_scatter(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
) -> Result<(String, String, Vec<Series>)> {
    let x_array: DatasetArray = batch
        .column_by_name(x_column)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y not found")?
        .try_into()?;
    let x_values = x_array.as_numeric().context("X must be numeric")?.values();
    let y_values = y_array.as_numeric().context("Y must be numeric")?.values();
    let len = x_values.len().min(y_values.len());
    let data = (0..len)
        .map(|i| vec![x_values[i], y_values[i]])
        .collect::<Vec<_>>();
    let series_name = format!("{x_column} vs {y_column}");
    Ok((
        x_column.to_string(),
        y_column.to_string(),
        vec![Series {
            name: series_name,
            data,
        }],
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Array, ArrayRef, Float64Array, StructArray, new_empty_array};
    use arrow_schema::{DataType, Field};
    use fricon::{DatasetArray, DatasetScalar, ScalarArray, ScalarKind, TraceKind};
    use indexmap::IndexMap;
    use num::complex::Complex64;

    use super::*;

    #[test]
    fn test_build_line_series_numeric() {
        let x_vals = vec![1.0, 2.0, 3.0];
        let y_vals = vec![10.0, 20.0, 30.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].name, "y");
        assert_eq!(
            res.series[0].data,
            vec![vec![1.0, 10.0], vec![2.0, 20.0], vec![3.0, 30.0]]
        );
    }

    #[test]
    fn test_build_line_series_complex() {
        let real_vals = vec![1.0, 2.0];
        let imag_vals = vec![3.0, 4.0];
        let real_array = Arc::new(Float64Array::from(real_vals));
        let imag_array = Arc::new(Float64Array::from(imag_vals));

        let fields = vec![
            Arc::new(Field::new("real", DataType::Float64, false)),
            Arc::new(Field::new("imag", DataType::Float64, false)),
        ];
        let complex_struct =
            StructArray::try_new(fields.into(), vec![real_array, imag_array], None).unwrap();

        let x_vals = vec![0.1, 0.2];
        let x_array = Arc::new(Float64Array::from(x_vals));

        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", complex_struct.data_type().clone(), false),
        ]));
        let batch =
            RecordBatch::try_new(arrow_schema, vec![x_array, Arc::new(complex_struct)]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: Some(vec![ComplexViewOption::Mag]),
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert!(res.series[0].name.contains("mag"));
        // Mag of (1,3) is sqrt(10) approx 3.16. (2,4) is sqrt(20) approx 4.47
        assert!((res.series[0].data[0][1] - 3.1622).abs() < 1e-4);
    }

    #[test]
    fn test_build_line_series_trace_empty_batch_returns_empty_series() {
        let trace_data_type = TraceKind::Simple
            .to_data_type(Arc::new(Field::new_list_field(DataType::Float64, false)));
        let trace_array = new_empty_array(&trace_data_type);
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "trace",
            trace_data_type,
            false,
        )]));
        let batch = RecordBatch::try_new(arrow_schema, vec![trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert!(res.series.is_empty());
    }

    #[test]
    fn test_build_line_series_trace_empty_trace_returns_empty_series() {
        let trace_array: ArrayRef = DatasetArray::from(DatasetScalar::SimpleTrace(
            ScalarArray::from_iter(Vec::<f64>::new()),
        ))
        .into();
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "trace",
            trace_array.data_type().clone(),
            false,
        )]));
        let batch = RecordBatch::try_new(arrow_schema, vec![trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert!(res.series.is_empty());
    }

    #[test]
    fn test_build_heatmap_series_numeric() {
        let x_vals = vec![1.0, 2.0];
        let y_vals = vec![10.0, 10.0];
        let z_vals = vec![100.0, 200.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let array_z = Arc::new(Float64Array::from(z_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("z", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y, array_z]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "z".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = HeatmapChartDataOptions {
            series: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_heatmap_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.x_categories, Some(vec![1.0, 2.0]));
        assert_eq!(res.y_categories, Some(vec![10.0]));
        assert_eq!(
            res.series[0].data,
            vec![vec![0.0, 0.0, 100.0], vec![1.0, 0.0, 200.0]]
        );
    }

    #[test]
    fn test_build_heatmap_series_maps_1_based_indexes() {
        let x_vals = vec![1.0, 2.0, 1.0];
        let y_vals = vec![1.0, 1.0, 2.0];
        let z_vals = vec![10.0, 20.0, 30.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let array_z = Arc::new(Float64Array::from(z_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("z", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y, array_z]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "z".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = HeatmapChartDataOptions {
            series: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_heatmap_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.x_categories, Some(vec![1.0, 2.0]));
        assert_eq!(res.y_categories, Some(vec![1.0, 2.0]));
        assert_eq!(
            res.series[0].data,
            vec![
                vec![0.0, 0.0, 10.0],
                vec![1.0, 0.0, 20.0],
                vec![0.0, 1.0, 30.0]
            ]
        );
    }

    #[test]
    fn test_build_heatmap_series_maps_non_contiguous_indexes() {
        let x_vals = vec![10.0, 20.0, 40.0];
        let y_vals = vec![5.0, 5.0, 9.0];
        let z_vals = vec![1.0, 2.0, 3.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let array_z = Arc::new(Float64Array::from(z_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("z", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y, array_z]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "z".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = HeatmapChartDataOptions {
            series: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_heatmap_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.x_categories, Some(vec![10.0, 20.0, 40.0]));
        assert_eq!(res.y_categories, Some(vec![5.0, 9.0]));
        assert_eq!(
            res.series[0].data,
            vec![
                vec![0.0, 0.0, 1.0],
                vec![1.0, 0.0, 2.0],
                vec![2.0, 1.0, 3.0]
            ]
        );
    }

    #[test]
    fn test_build_heatmap_series_trace_uses_same_category_semantics() {
        let y_vals = vec![100.0];
        let trace = DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![1.0, 2.0, 3.0]));
        let trace_array: ArrayRef = DatasetArray::from(trace).into();
        let y_array = Arc::new(Float64Array::from(y_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("y", DataType::Float64, false),
            Field::new("trace", trace_array.data_type().clone(), false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![y_array, trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = HeatmapChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_heatmap_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.x_categories, Some(vec![0.0, 1.0, 2.0]));
        assert_eq!(res.y_categories, Some(vec![100.0]));
        assert_eq!(
            res.series[0].data,
            vec![
                vec![0.0, 0.0, 1.0],
                vec![1.0, 0.0, 2.0],
                vec![2.0, 0.0, 3.0]
            ]
        );
    }

    #[test]
    fn test_build_scatter_series_complex_scalar_and_trace() {
        let scalar_complex_column = Arc::new(
            StructArray::try_new(
                vec![
                    Arc::new(Field::new("real", DataType::Float64, false)),
                    Arc::new(Field::new("imag", DataType::Float64, false)),
                ]
                .into(),
                vec![
                    Arc::new(Float64Array::from(vec![1.0, 2.0])),
                    Arc::new(Float64Array::from(vec![-1.0, -2.0])),
                ],
                None,
            )
            .unwrap(),
        );
        let scalar_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "c",
            scalar_complex_column.data_type().clone(),
            false,
        )]));
        let scalar_batch =
            RecordBatch::try_new(scalar_schema, vec![scalar_complex_column]).unwrap();

        let mut scalar_columns = IndexMap::new();
        scalar_columns.insert(
            "c".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        let scalar_dataset_schema = DatasetSchema::new(scalar_columns);
        let scalar_options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "c".to_string(),
            },
            common: ChartCommonOptions::default(),
        };
        let scalar_res =
            build_scatter_series(&scalar_batch, &scalar_dataset_schema, &scalar_options).unwrap();
        assert_eq!(
            scalar_res.series[0].data,
            vec![vec![1.0, -1.0], vec![2.0, -2.0]]
        );

        let trace_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                Complex64::new(3.0, 4.0),
                Complex64::new(5.0, 6.0),
            ])))
            .into();
        let trace_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "t",
            trace_array.data_type().clone(),
            false,
        )]));
        let trace_batch = RecordBatch::try_new(trace_schema, vec![trace_array]).unwrap();

        let mut trace_columns = IndexMap::new();
        trace_columns.insert(
            "t".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Complex),
        );
        let trace_dataset_schema = DatasetSchema::new(trace_columns);
        let trace_options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "t".to_string(),
            },
            common: ChartCommonOptions::default(),
        };
        let trace_res =
            build_scatter_series(&trace_batch, &trace_dataset_schema, &trace_options).unwrap();
        assert_eq!(
            trace_res.series[0].data,
            vec![vec![3.0, 4.0], vec![5.0, 6.0]]
        );
    }

    #[test]
    fn test_build_scatter_series_trace_xy_truncates_to_shorter_trace() {
        let x_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                1.0, 2.0, 3.0,
            ])))
            .into();
        let y_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                10.0, 20.0,
            ])))
            .into();
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("tx", x_array.data_type().clone(), false),
            Field::new("ty", y_array.data_type().clone(), false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![x_array, y_array]).unwrap();

        let options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::TraceXy {
                trace_x_column: "tx".to_string(),
                trace_y_column: "ty".to_string(),
            },
            common: ChartCommonOptions::default(),
        };

        let mut columns = IndexMap::new();
        columns.insert(
            "tx".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        columns.insert(
            "ty".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let res = build_scatter_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series[0].data, vec![vec![1.0, 10.0], vec![2.0, 20.0]]);
    }

    #[test]
    fn test_build_scatter_series_xy() {
        let x_vals = vec![1.0, 2.0];
        let y_vals = vec![10.0, 20.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: None,
            },
            common: ChartCommonOptions::default(),
        };

        let res = build_scatter_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].data, vec![vec![1.0, 10.0], vec![2.0, 20.0]]);
    }

    #[test]
    fn test_deserialize_scatter_xy_missing_required_field_fails() {
        let input = serde_json::json!({
            "chartType": "scatter",
            "scatter": {
                "mode": "xy",
                "xColumn": "x"
            }
        });
        let parsed: std::result::Result<DatasetChartDataOptions, _> = serde_json::from_value(input);
        assert!(parsed.is_err());
    }
}

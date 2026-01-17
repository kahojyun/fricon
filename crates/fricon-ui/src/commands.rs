#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    ops::Bound,
    sync::{Arc, LazyLock, Mutex, MutexGuard},
    time::Duration,
};

use anyhow::Context;
use arrow_array::{Array, ArrayRef, Float64Array, ListArray, RecordBatch, StructArray};
use arrow_ipc::writer::FileWriter;
use arrow_schema::DataType;
use arrow_select::concat::concat_batches;
use chrono::{DateTime, Utc};
use fricon::{DatasetDataType, DatasetSchema, DatasetUpdate, ScalarKind, SelectOptions};
use serde::{Deserialize, Serialize, Serializer};
use tauri::{
    State,
    ipc::{Channel, Invoke, Response},
};
use tokio::time;
use tokio_util::sync::CancellationToken;

use super::AppState;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct Error(#[from] anyhow::Error);

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatasetInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceInfo {
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColumnInfo {
    name: String,
    is_complex: bool,
    is_trace: bool,
    is_index: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetDetail {
    columns: Vec<ColumnInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetDataOptions {
    start: Option<usize>,
    end: Option<usize>,
    /// Indices of chosen values for each filter field
    index_filters: Option<Vec<usize>>,
    exclude_columns: Option<Vec<String>>,
    columns: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ChartType {
    Line,
    Heatmap,
    Scatter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ScatterMode {
    Complex,
    TraceXy,
    Xy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ComplexViewOption {
    Real,
    Imag,
    Mag,
    Arg,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChartDataOptions {
    chart_type: ChartType,
    series: Option<String>,
    x_column: Option<String>,
    y_column: Option<String>,
    scatter_mode: Option<ScatterMode>,
    scatter_series: Option<String>,
    scatter_x_column: Option<String>,
    scatter_y_column: Option<String>,
    scatter_trace_x_column: Option<String>,
    scatter_trace_y_column: Option<String>,
    scatter_bin_column: Option<String>,
    complex_views: Option<Vec<ComplexViewOption>>,
    complex_view_single: Option<ComplexViewOption>,
    start: Option<usize>,
    end: Option<usize>,
    /// Indices of chosen values for each filter field
    index_filters: Option<Vec<usize>>,
    exclude_columns: Option<Vec<String>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChartSeries {
    name: String,
    data: Vec<Vec<f64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartDataResponse {
    r#type: ChartType,
    x_name: String,
    y_name: Option<String>,
    series: Vec<ChartSeries>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetWriteProgress {
    row_count: usize,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FilterTableRow {
    display_values: Vec<String>,
    value_indices: Vec<usize>,
    index: usize,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ColumnUniqueValue {
    index: usize,
    display_value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilterTableData {
    fields: Vec<String>,
    rows: Vec<FilterTableRow>,
    column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

struct ProcessedFilterRows {
    unique_rows: Vec<FilterTableRow>,
    column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

struct FilterDataInternal {
    fields: Vec<String>,
    unique_rows: Vec<FilterTableRow>,
    column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn column_data_type(schema: &DatasetSchema, name: &str) -> Result<DatasetDataType, Error> {
    let (_, _, data_type) = schema
        .columns()
        .get_full(name)
        .with_context(|| format!("Column '{name}' not found"))?;
    Ok(*data_type)
}

fn is_complex_type(data_type: DatasetDataType) -> bool {
    matches!(
        data_type,
        DatasetDataType::Scalar(ScalarKind::Complex)
            | DatasetDataType::Trace(_, ScalarKind::Complex)
    )
}

fn collect_float_values(array: &ArrayRef) -> Result<Vec<f64>, Error> {
    let float_array: Arc<Float64Array> =
        fricon::downcast_array(array.clone()).context("Expected Float64 array")?;
    Ok(float_array.values().iter().copied().collect())
}

fn collect_complex_components(array: &ArrayRef) -> Result<(Vec<f64>, Vec<f64>), Error> {
    let struct_array: Arc<StructArray> =
        fricon::downcast_array(array.clone()).context("Expected complex struct array")?;
    let real_array: Arc<Float64Array> = fricon::downcast_array(struct_array.column(0).clone())
        .context("Expected real component array")?;
    let imag_array: Arc<Float64Array> = fricon::downcast_array(struct_array.column(1).clone())
        .context("Expected imag component array")?;
    Ok((
        real_array.values().iter().copied().collect(),
        imag_array.values().iter().copied().collect(),
    ))
}

fn transform_complex_values(reals: &[f64], imags: &[f64], option: ComplexViewOption) -> Vec<f64> {
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

fn concat_record_batches(
    schema: &Arc<arrow_schema::Schema>,
    batches: &[RecordBatch],
) -> Result<RecordBatch, Error> {
    if batches.is_empty() {
        return Ok(RecordBatch::new_empty(schema.clone()));
    }
    concat_batches(schema, batches)
        .context("Failed to concat batches")
        .map_err(Error::from)
}

fn extract_trace_row(
    trace_array: &ArrayRef,
    row: usize,
) -> Result<Option<(Vec<f64>, ArrayRef)>, Error> {
    match trace_array.data_type() {
        DataType::List(_) => {
            let list_array: Arc<ListArray> =
                fricon::downcast_array(trace_array.clone()).context("Expected list array")?;
            if list_array.is_null(row) {
                return Ok(None);
            }
            let values = list_array.value(row);
            let x = (0..values.len()).map(|i| i as f64).collect();
            Ok(Some((x, values)))
        }
        DataType::Struct(fields) => {
            let struct_array: Arc<StructArray> =
                fricon::downcast_array(trace_array.clone()).context("Expected struct array")?;
            if struct_array.is_null(row) {
                return Ok(None);
            }
            let names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
            if names == ["x0", "step", "y"] {
                let x0_array: Arc<Float64Array> =
                    fricon::downcast_array(struct_array.column(0).clone())
                        .context("Expected x0 array")?;
                let step_array: Arc<Float64Array> =
                    fricon::downcast_array(struct_array.column(1).clone())
                        .context("Expected step array")?;
                let y_array: Arc<ListArray> =
                    fricon::downcast_array(struct_array.column(2).clone())
                        .context("Expected y list array")?;
                if y_array.is_null(row) {
                    return Ok(None);
                }
                let y_values = y_array.value(row);
                let x0 = x0_array.value(row);
                let step = step_array.value(row);
                let x = (0..y_values.len())
                    .map(|i| x0 + (i as f64) * step)
                    .collect();
                Ok(Some((x, y_values)))
            } else if names == ["x", "y"] {
                let x_array: Arc<ListArray> =
                    fricon::downcast_array(struct_array.column(0).clone())
                        .context("Expected x list array")?;
                let y_array: Arc<ListArray> =
                    fricon::downcast_array(struct_array.column(1).clone())
                        .context("Expected y list array")?;
                if x_array.is_null(row) || y_array.is_null(row) {
                    return Ok(None);
                }
                let x_values = x_array.value(row);
                let y_values = y_array.value(row);
                let x = collect_float_values(&x_values)?;
                Ok(Some((x, y_values)))
            } else {
                return Err(Error(anyhow::anyhow!("Unsupported trace struct layout")));
            }
        }
        _ => return Err(Error(anyhow::anyhow!("Unsupported trace data type"))),
    }
}

fn complex_view_label(option: ComplexViewOption) -> &'static str {
    match option {
        ComplexViewOption::Real => "real",
        ComplexViewOption::Imag => "imag",
        ComplexViewOption::Mag => "mag",
        ComplexViewOption::Arg => "arg",
    }
}

fn column_index(schema: &DatasetSchema, name: &str) -> Result<usize, Error> {
    let (idx, _, _) = schema
        .columns()
        .get_full(name)
        .with_context(|| format!("Column '{name}' not found"))?;
    Ok(idx)
}

fn push_column(columns: &mut Vec<usize>, index: usize) {
    if !columns.contains(&index) {
        columns.push(index);
    }
}

fn build_chart_selected_columns(
    schema: &DatasetSchema,
    options: &ChartDataOptions,
) -> Result<Vec<usize>, Error> {
    let mut selected = Vec::new();
    match options.chart_type {
        ChartType::Line => {
            let series_name = options
                .series
                .as_ref()
                .context("Line chart requires a series column")?;
            let series_index = column_index(schema, series_name)?;
            let data_type = column_data_type(schema, series_name)?;
            push_column(&mut selected, series_index);
            if !matches!(data_type, DatasetDataType::Trace(_, _)) {
                let x_name = options
                    .x_column
                    .as_ref()
                    .context("Line chart requires x column")?;
                let x_index = column_index(schema, x_name)?;
                push_column(&mut selected, x_index);
            }
        }
        ChartType::Heatmap => {
            let series_name = options
                .series
                .as_ref()
                .context("Heatmap chart requires a series column")?;
            let series_index = column_index(schema, series_name)?;
            let data_type = column_data_type(schema, series_name)?;
            push_column(&mut selected, series_index);
            let y_name = options
                .y_column
                .as_ref()
                .context("Heatmap chart requires y column")?;
            let y_index = column_index(schema, y_name)?;
            push_column(&mut selected, y_index);
            if !matches!(data_type, DatasetDataType::Trace(_, _)) {
                let x_name = options
                    .x_column
                    .as_ref()
                    .context("Heatmap chart requires x column")?;
                let x_index = column_index(schema, x_name)?;
                push_column(&mut selected, x_index);
            }
        }
        ChartType::Scatter => {
            let mode = options.scatter_mode.unwrap_or(ScatterMode::Complex);
            match mode {
                ScatterMode::Complex => {
                    let series_name = options
                        .scatter_series
                        .as_ref()
                        .context("Scatter complex mode requires series column")?;
                    let series_index = column_index(schema, series_name)?;
                    push_column(&mut selected, series_index);
                }
                ScatterMode::TraceXy => {
                    let x_name = options
                        .scatter_trace_x_column
                        .as_ref()
                        .context("Scatter trace_xy requires trace x column")?;
                    let y_name = options
                        .scatter_trace_y_column
                        .as_ref()
                        .context("Scatter trace_xy requires trace y column")?;
                    push_column(&mut selected, column_index(schema, x_name)?);
                    push_column(&mut selected, column_index(schema, y_name)?);
                }
                ScatterMode::Xy => {
                    let x_name = options
                        .scatter_x_column
                        .as_ref()
                        .context("Scatter xy requires x column")?;
                    let y_name = options
                        .scatter_y_column
                        .as_ref()
                        .context("Scatter xy requires y column")?;
                    push_column(&mut selected, column_index(schema, x_name)?);
                    push_column(&mut selected, column_index(schema, y_name)?);
                    if let Some(bin_name) = options.scatter_bin_column.as_ref() {
                        push_column(&mut selected, column_index(schema, bin_name)?);
                    }
                }
            }
        }
    }
    Ok(selected)
}

fn build_line_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ChartDataOptions,
) -> Result<ChartDataResponse, Error> {
    let series_name = options
        .series
        .as_ref()
        .context("Line chart requires a series column")?;
    let data_type = column_data_type(schema, series_name)?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = is_complex_type(data_type);
    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_array = batch
        .column_by_name(series_name)
        .cloned()
        .with_context(|| format!("Series column '{series_name}' not found"))?;

    let (x_values, y_values_array) = if is_trace {
        if batch.num_rows() != 1 {
            return Err(Error(anyhow::anyhow!(format!(
                "Trace series should fetch exactly 1 row, actual: {}",
                batch.num_rows()
            ))));
        }
        extract_trace_row(&series_array, 0)?.context("Trace series row is null")?
    } else {
        let x_column = options
            .x_column
            .as_ref()
            .context("Line chart requires x column")?;
        let x_array = batch
            .column_by_name(x_column)
            .cloned()
            .with_context(|| format!("X column '{x_column}' not found"))?;
        (collect_float_values(&x_array)?, series_array)
    };

    let series = if is_complex {
        let (reals, imags) = collect_complex_components(&y_values_array)?;
        let view_options = options
            .complex_views
            .clone()
            .unwrap_or_else(|| vec![ComplexViewOption::Real, ComplexViewOption::Imag]);
        view_options
            .into_iter()
            .map(|option| {
                let y_values = transform_complex_values(&reals, &imags, option);
                let len = x_values.len().min(y_values.len());
                let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
                ChartSeries {
                    name: format!("{series_name} ({})", complex_view_label(option)),
                    data,
                }
            })
            .collect()
    } else {
        let y_values = collect_float_values(&y_values_array)?;
        let len = x_values.len().min(y_values.len());
        vec![ChartSeries {
            name: series_name.to_string(),
            data: (0..len).map(|i| vec![x_values[i], y_values[i]]).collect(),
        }]
    };

    Ok(ChartDataResponse {
        r#type: ChartType::Line,
        x_name: x_name.to_string(),
        y_name: None,
        series,
    })
}

fn build_heatmap_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ChartDataOptions,
) -> Result<ChartDataResponse, Error> {
    let series_name = options
        .series
        .as_ref()
        .context("Heatmap chart requires a series column")?;
    let y_column = options
        .y_column
        .as_ref()
        .context("Heatmap chart requires y column")?;
    let data_type = column_data_type(schema, series_name)?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = is_complex_type(data_type);
    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_array = batch
        .column_by_name(series_name)
        .cloned()
        .with_context(|| format!("Series column '{series_name}' not found"))?;
    let view_option = options
        .complex_view_single
        .unwrap_or(ComplexViewOption::Mag);

    let series = if is_trace {
        let y_array = batch
            .column_by_name(y_column)
            .cloned()
            .with_context(|| format!("Y column '{y_column}' not found"))?;
        let y_values = collect_float_values(&y_array)?;
        let mut data = Vec::new();
        for row in 0..batch.num_rows() {
            let Some((x_values, trace_values)) = extract_trace_row(&series_array, row)? else {
                continue;
            };
            let y_value = *y_values.get(row).unwrap_or(&0.0);
            if is_complex {
                let (reals, imags) = collect_complex_components(&trace_values)?;
                let z_values = transform_complex_values(&reals, &imags, view_option);
                let len = x_values.len().min(z_values.len());
                for i in 0..len {
                    data.push(vec![x_values[i], y_value, z_values[i]]);
                }
            } else {
                let z_values = collect_float_values(&trace_values)?;
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
        vec![ChartSeries { name, data }]
    } else {
        let x_column = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        let x_array = batch
            .column_by_name(x_column)
            .cloned()
            .with_context(|| format!("X column '{x_column}' not found"))?;
        let y_array = batch
            .column_by_name(y_column)
            .cloned()
            .with_context(|| format!("Y column '{y_column}' not found"))?;
        let x_values = collect_float_values(&x_array)?;
        let y_values = collect_float_values(&y_array)?;
        let data = if is_complex {
            let (reals, imags) = collect_complex_components(&series_array)?;
            let z_values = transform_complex_values(&reals, &imags, view_option);
            let len = x_values.len().min(y_values.len()).min(z_values.len());
            (0..len)
                .map(|i| vec![x_values[i], y_values[i], z_values[i]])
                .collect()
        } else {
            let z_values = collect_float_values(&series_array)?;
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
        vec![ChartSeries { name, data }]
    };

    Ok(ChartDataResponse {
        r#type: ChartType::Heatmap,
        x_name: x_name.to_string(),
        y_name: Some(y_column.to_string()),
        series,
    })
}

fn build_scatter_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ChartDataOptions,
) -> Result<ChartDataResponse, Error> {
    let mode = options.scatter_mode.unwrap_or(ScatterMode::Complex);
    let mut series_map: HashMap<String, Vec<Vec<f64>>> = HashMap::new();
    let (x_name, y_name) = match mode {
        ScatterMode::Complex => {
            let series_name = options
                .scatter_series
                .as_ref()
                .context("Scatter complex mode requires series column")?;
            let data_type = column_data_type(schema, series_name)?;
            let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
            let series_array = batch
                .column_by_name(series_name)
                .cloned()
                .with_context(|| format!("Series column '{series_name}' not found"))?;
            let mut data = Vec::new();
            if is_trace {
                for row in 0..batch.num_rows() {
                    let Some((_x_values, trace_values)) = extract_trace_row(&series_array, row)?
                    else {
                        continue;
                    };
                    let (reals, imags) = collect_complex_components(&trace_values)?;
                    let len = reals.len().min(imags.len());
                    for i in 0..len {
                        data.push(vec![reals[i], imags[i]]);
                    }
                }
            } else {
                let (reals, imags) = collect_complex_components(&series_array)?;
                let len = reals.len().min(imags.len());
                for i in 0..len {
                    data.push(vec![reals[i], imags[i]]);
                }
            }
            series_map.insert(series_name.to_string(), data);
            (
                format!("{series_name} (real)"),
                format!("{series_name} (imag)"),
            )
        }
        ScatterMode::TraceXy => {
            let trace_x = options
                .scatter_trace_x_column
                .as_ref()
                .context("Scatter trace_xy requires trace x column")?;
            let trace_y = options
                .scatter_trace_y_column
                .as_ref()
                .context("Scatter trace_xy requires trace y column")?;
            let x_array = batch
                .column_by_name(trace_x)
                .cloned()
                .with_context(|| format!("Trace x column '{trace_x}' not found"))?;
            let y_array = batch
                .column_by_name(trace_y)
                .cloned()
                .with_context(|| format!("Trace y column '{trace_y}' not found"))?;
            let mut data = Vec::new();
            for row in 0..batch.num_rows() {
                let Some((_x_axis, x_values_array)) = extract_trace_row(&x_array, row)? else {
                    continue;
                };
                let Some((_y_axis, y_values_array)) = extract_trace_row(&y_array, row)? else {
                    continue;
                };
                let x_values = collect_float_values(&x_values_array)?;
                let y_values = collect_float_values(&y_values_array)?;
                let len = x_values.len().min(y_values.len());
                for i in 0..len {
                    data.push(vec![x_values[i], y_values[i]]);
                }
            }
            let series_name = format!("{trace_x} vs {trace_y}");
            series_map.insert(series_name.clone(), data);
            (trace_x.to_string(), trace_y.to_string())
        }
        ScatterMode::Xy => {
            let x_column = options
                .scatter_x_column
                .as_ref()
                .context("Scatter xy requires x column")?;
            let y_column = options
                .scatter_y_column
                .as_ref()
                .context("Scatter xy requires y column")?;
            let x_array = batch
                .column_by_name(x_column)
                .cloned()
                .with_context(|| format!("X column '{x_column}' not found"))?;
            let y_array = batch
                .column_by_name(y_column)
                .cloned()
                .with_context(|| format!("Y column '{y_column}' not found"))?;
            let x_values = collect_float_values(&x_array)?;
            let y_values = collect_float_values(&y_array)?;
            let len = x_values.len().min(y_values.len());
            let data = (0..len)
                .map(|i| vec![x_values[i], y_values[i]])
                .collect::<Vec<_>>();
            let series_name = format!("{x_column} vs {y_column}");
            series_map.insert(series_name.clone(), data);
            (x_column.to_string(), y_column.to_string())
        }
    };

    let series = series_map
        .into_iter()
        .map(|(name, data)| ChartSeries { name, data })
        .collect();

    Ok(ChartDataResponse {
        r#type: ChartType::Scatter,
        x_name,
        y_name: Some(y_name),
        series,
    })
}

#[tauri::command]
async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: ChartDataOptions,
) -> Result<ChartDataResponse, Error> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let start = options.start.map_or(Bound::Unbounded, Bound::Included);
    let end = options.end.map_or(Bound::Unbounded, Bound::Excluded);
    let index_filters = if let Some(indices) = options.index_filters.clone() {
        build_filter_batch(
            &state,
            id,
            options.exclude_columns.clone(),
            &indices,
            dataset.arrow_schema().clone(),
        )
        .await?
    } else {
        None
    };

    let selected_columns = build_chart_selected_columns(schema, &options)?;
    let (output_schema, batches) = dataset
        .select_data(&SelectOptions {
            start,
            end,
            index_filters,
            selected_columns: Some(selected_columns),
        })
        .context("Failed to select data.")?;
    let batch = concat_record_batches(&output_schema, &batches)?;

    match options.chart_type {
        ChartType::Line => build_line_series(&batch, schema, &options),
        ChartType::Heatmap => build_heatmap_series(&batch, schema, &options),
        ChartType::Scatter => build_scatter_series(&batch, schema, &options),
    }
}

fn process_filter_rows(
    fields: &[String],
    json_rows: Vec<serde_json::Map<String, serde_json::Value>>,
) -> ProcessedFilterRows {
    let mut unique_rows = Vec::new();
    let mut seen_keys = HashSet::new();
    let mut column_unique_values: HashMap<String, Vec<ColumnUniqueValue>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();
    let mut column_raw_values: HashMap<String, Vec<serde_json::Value>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();

    for (global_row_idx, json_row) in json_rows.into_iter().enumerate() {
        // Extract values in field order
        let values: Vec<serde_json::Value> = fields
            .iter()
            .map(|field| {
                json_row
                    .get(field)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            })
            .collect();

        // Create key for deduplication
        let key = serde_json::to_string(&values).unwrap_or_default();

        if !seen_keys.contains(&key) {
            seen_keys.insert(key);
            let display_values = values.iter().map(format_json_value).collect();
            let mut value_indices = Vec::with_capacity(values.len());

            // Collect unique values per column and track indices
            for (col_idx, value) in values.iter().enumerate() {
                if let Some(field_name) = fields.get(col_idx) {
                    let raw_values = column_raw_values
                        .get_mut(field_name)
                        .expect("Field should exist in column_raw_values");

                    let index = if let Some(pos) = raw_values.iter().position(|v| v == value) {
                        pos
                    } else {
                        let new_index = raw_values.len();
                        raw_values.push(value.clone());

                        let display_value = format_json_value(value);
                        column_unique_values
                            .get_mut(field_name)
                            .expect("Field should exist in column_unique_values")
                            .push(ColumnUniqueValue {
                                index: new_index,
                                display_value,
                            });
                        new_index
                    };
                    value_indices.push(index);
                }
            }

            unique_rows.push(FilterTableRow {
                display_values,
                value_indices,
                index: global_row_idx,
            });
        }
    }
    ProcessedFilterRows {
        unique_rows,
        column_unique_values,
        column_raw_values,
    }
}

#[tauri::command]
async fn get_workspace_info(state: State<'_, AppState>) -> Result<WorkspaceInfo, Error> {
    let app = state.app();
    let workspace_paths = app.paths().context("Failed to retrieve workspace paths.")?;
    let workspace_path = workspace_paths.root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetListOptions {
    search: Option<String>,
    tags: Option<Vec<String>>,
}

#[tauri::command]
async fn list_datasets(
    state: State<'_, AppState>,
    options: Option<DatasetListOptions>,
) -> Result<Vec<DatasetInfo>, Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    let (search, tags) = options
        .map(|options| (options.search, options.tags))
        .unwrap_or_default();
    let datasets = dataset_manager
        .list_datasets(search.as_deref(), tags.as_deref())
        .await
        .context("Failed to list datasets.")?;

    let dataset_info: Vec<DatasetInfo> = datasets
        .into_iter()
        .map(|record| DatasetInfo {
            id: record.id,
            name: record.metadata.name,
            description: record.metadata.description,
            favorite: record.metadata.favorite,
            tags: record.metadata.tags,
            created_at: record.metadata.created_at,
        })
        .collect();

    Ok(dataset_info)
}

#[tauri::command]
async fn dataset_detail(state: State<'_, AppState>, id: i32) -> Result<DatasetDetail, Error> {
    let reader = state.dataset(id).await?;
    let schema = reader.schema();
    let index = reader.index_columns();
    let columns = schema
        .columns()
        .iter()
        .enumerate()
        .map(|(i, (name, data_type))| ColumnInfo {
            name: name.to_owned(),
            is_complex: matches!(
                data_type,
                DatasetDataType::Scalar(ScalarKind::Complex)
                    | DatasetDataType::Trace(_, ScalarKind::Complex)
            ),
            is_trace: matches!(data_type, DatasetDataType::Trace(_, _)),
            is_index: index.as_ref().is_some_and(|index| index.contains(&i)),
        })
        .collect();
    Ok(DatasetDetail { columns })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetFavoriteUpdate {
    favorite: bool,
}

#[tauri::command]
async fn update_dataset_favorite(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetFavoriteUpdate,
) -> Result<(), Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    dataset_manager
        .update_dataset(
            id,
            DatasetUpdate {
                name: None,
                description: None,
                favorite: Some(update.favorite),
            },
        )
        .await
        .context("Failed to update dataset favorite status.")?;
    Ok(())
}

type SubscriptionRecords = HashMap<u32, CancellationToken>;
static DATASET_SUBSCRIPTION: LazyLock<Mutex<SubscriptionRecords>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn subscriptions_mut() -> MutexGuard<'static, SubscriptionRecords> {
    DATASET_SUBSCRIPTION
        .lock()
        .expect("Should never be poisoned")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilterTableOptions {
    exclude_columns: Option<Vec<String>>,
}

async fn get_filter_data_internal(
    state: &AppState,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<FilterDataInternal, Error> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();

    // Get index column indices
    let Some(index_col_indices) = index_columns else {
        return Ok(FilterDataInternal {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        });
    };

    // Filter out X column from index columns
    let filtered_indices: Vec<usize> = index_col_indices
        .iter()
        .filter(|&&i| {
            let col_name = schema.columns().keys().nth(i).map(String::as_str);
            if let Some(exclude) = &exclude_columns {
                col_name.is_none_or(|name| !exclude.iter().any(|e| e == name))
            } else {
                true
            }
        })
        .copied()
        .collect();

    if filtered_indices.is_empty() {
        return Ok(FilterDataInternal {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        });
    }

    // Get field names
    let fields: Vec<String> = filtered_indices
        .iter()
        .filter_map(|&i| schema.columns().keys().nth(i).cloned())
        .collect();

    // Fetch index column data
    let (_, batches) = dataset
        .select_data(&SelectOptions {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
            index_filters: None,
            selected_columns: Some(filtered_indices.clone()),
        })
        .context("Failed to select index data.")?;

    // Convert batches to JSON rows using arrow_json ArrayWriter
    let mut buf = Vec::new();
    {
        let mut writer = arrow_json::ArrayWriter::new(&mut buf);
        for batch in &batches {
            writer.write(batch).context("Failed to write batch")?;
        }
        writer.finish().context("Failed to finish JSON writer")?;
    }
    let json_rows: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_slice(&buf).context("Failed to parse JSON")?;

    // Process rows: deduplicate and compute unique values and raw values for
    // indexing
    let processed = process_filter_rows(&fields, json_rows);
    Ok(FilterDataInternal {
        fields,
        unique_rows: processed.unique_rows,
        column_unique_values: processed.column_unique_values,
        column_raw_values: processed.column_raw_values,
    })
}

async fn build_filter_batch(
    state: &AppState,
    id: i32,
    exclude_columns: Option<Vec<String>>,
    indices: &[usize],
    arrow_schema: Arc<arrow_schema::Schema>,
) -> Result<Option<arrow_array::RecordBatch>, Error> {
    let filter_data = get_filter_data_internal(state, id, exclude_columns).await?;
    let fields = filter_data.fields;
    let raw_values_map = filter_data.column_raw_values;

    let mut filter_map = serde_json::Map::new();
    for (idx, &value_idx) in indices.iter().enumerate() {
        if let Some(field_name) = fields.get(idx)
            && let Some(val) = raw_values_map
                .get(field_name)
                .and_then(|values| values.get(value_idx))
        {
            filter_map.insert(field_name.clone(), val.clone());
        }
    }

    if filter_map.is_empty() {
        return Ok(None);
    }

    // Build filter schema from the dataset schema (only include filter fields)
    let projection_indices: Vec<usize> = filter_map
        .keys()
        .map(|field_name| {
            arrow_schema
                .index_of(field_name)
                .with_context(|| format!("Field '{field_name}' not found in schema"))
        })
        .collect::<Result<_, _>>()?;
    let filter_schema = Arc::new(
        arrow_schema
            .project(&projection_indices)
            .context("Failed to project arrow schema")?,
    );

    // Convert HashMap to JSON array (single row)
    let json_row = serde_json::Value::Object(filter_map);
    // serialize as single object (NDJSON style) for arrow_json Reader
    let json_array = serde_json::to_vec(&json_row).context("Failed to serialize filter to JSON")?;

    // Use arrow_json::ReaderBuilder to decode with schema
    let mut reader = arrow_json::ReaderBuilder::new(filter_schema)
        .build(Cursor::new(json_array))
        .context("Failed to create JSON reader")?;

    let batch = reader
        .next()
        .context("No batch returned")?
        .context("Failed to decode filter batch")?;

    Ok(Some(batch))
}

#[tauri::command]
async fn dataset_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetDataOptions,
) -> Result<Response, Error> {
    let dataset = state.dataset(id).await?;
    let start = options.start.map_or(Bound::Unbounded, Bound::Included);
    let end = options.end.map_or(Bound::Unbounded, Bound::Excluded);

    let index_filters = if let Some(indices) = options.index_filters {
        build_filter_batch(
            &state,
            id,
            options.exclude_columns,
            &indices,
            dataset.arrow_schema().clone(),
        )
        .await?
    } else {
        None
    };

    let (output_schema, batches) = dataset
        .select_data(&SelectOptions {
            start,
            end,
            index_filters,
            selected_columns: options.columns,
        })
        .context("Failed to select data.")?;
    let buffer = vec![];
    let mut writer =
        FileWriter::try_new(buffer, &output_schema).context("Failed to create writer")?;
    for batch in batches {
        writer.write(&batch).context("Failed to write batch")?;
    }
    let buffer = writer.into_inner().context("Failed to finish writer")?;
    Ok(Response::new(buffer))
}

#[tauri::command]
async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<FilterTableData, Error> {
    let filter_data = get_filter_data_internal(&state, id, options.exclude_columns).await?;

    Ok(FilterTableData {
        fields: filter_data.fields,
        rows: filter_data.unique_rows,
        column_unique_values: filter_data.column_unique_values,
    })
}

#[tauri::command]
async fn subscribe_dataset_update(
    state: State<'_, AppState>,
    id: i32,
    on_update: Channel<DatasetWriteProgress>,
) -> Result<bool, Error> {
    let dataset = state.dataset(id).await?;
    if let Some(mut watcher) = dataset.subscribe() {
        let token = CancellationToken::new();
        let channel_id = on_update.id();
        subscriptions_mut().insert(channel_id, token.clone());
        tokio::spawn(async move {
            token
                .run_until_cancelled(async move {
                    while watcher.changed().await.is_ok() {
                        let msg = DatasetWriteProgress {
                            row_count: watcher.borrow_and_update().row_count,
                        };
                        if on_update.send(msg).is_err() {
                            break;
                        }
                        time::sleep(Duration::from_millis(200)).await;
                    }
                })
                .await;
            subscriptions_mut().remove(&channel_id);
        });
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn unsubscribe_dataset_update(channel_id: u32) -> Result<(), Error> {
    if let Some(t) = subscriptions_mut().remove(&channel_id) {
        t.cancel();
    }
    Ok(())
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![
        get_workspace_info,
        list_datasets,
        dataset_detail,
        dataset_data,
        dataset_chart_data,
        get_filter_table_data,
        update_dataset_favorite,
        subscribe_dataset_update,
        unsubscribe_dataset_update
    ]
}

#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::{
    collections::HashMap,
    io::Cursor,
    ops::Bound,
    sync::{Arc, LazyLock, Mutex, MutexGuard},
    time::Duration,
};

use anyhow::Context;
use arrow_array::RecordBatch;
use arrow_ipc::writer::FileWriter;
use chrono::{DateTime, Utc};
use fricon::{DatasetDataType, DatasetSchema, DatasetUpdate, SelectOptions};
use serde::{Deserialize, Serialize};
use tauri::{
    State,
    ipc::{Channel, Invoke, Response},
};
use tokio::time;
use tokio_util::sync::CancellationToken;

use super::AppState;
use crate::models::{
    chart::{
        ChartDataOptions, ChartDataResponse, ChartType, ScatterMode, build_heatmap_series,
        build_line_series, build_scatter_series,
    },
    filter::{FilterDataInternal, FilterTableData, process_filter_rows},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct Error(#[from] anyhow::Error);

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetWriteProgress {
    row_count: usize,
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
            let data_type = *schema
                .columns()
                .get(series_name)
                .context("Column not found")?;
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
            let data_type = *schema
                .columns()
                .get(series_name)
                .context("Column not found")?;
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

    let batch = if batches.is_empty() {
        RecordBatch::new_empty(output_schema)
    } else {
        arrow_select::concat::concat_batches(&output_schema, &batches)
            .context("Failed to concat batches")?
    };

    match options.chart_type {
        ChartType::Line => build_line_series(&batch, schema, &options).map_err(Error::from),
        ChartType::Heatmap => build_heatmap_series(&batch, schema, &options).map_err(Error::from),
        ChartType::Scatter => build_scatter_series(&batch, schema, &options).map_err(Error::from),
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
            is_complex: data_type.is_complex(),
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

#[expect(clippy::type_complexity)]
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

    let Some(index_col_indices) = index_columns else {
        return Ok(FilterDataInternal {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        });
    };

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

    let fields: Vec<String> = filtered_indices
        .iter()
        .filter_map(|&i| schema.columns().keys().nth(i).cloned())
        .collect();

    let (_, batches) = dataset
        .select_data(&SelectOptions {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
            index_filters: None,
            selected_columns: Some(filtered_indices.clone()),
        })
        .context("Failed to select index data.")?;

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

    let json_row = serde_json::Value::Object(filter_map);
    let json_array = serde_json::to_vec(&json_row).context("Failed to serialize filter to JSON")?;

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
            let _ = token
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

#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::{
    collections::HashMap,
    ops::Bound,
    sync::{LazyLock, Mutex, MutexGuard},
    time::Duration,
};

use anyhow::Context;
use arrow_ipc::writer::FileWriter;
use chrono::{DateTime, Utc};
use fricon::{DatasetDataType, ScalarKind, SelectOptions};
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
    /// JSON object mapping field names to filter values
    index_filters: Option<HashMap<String, serde_json::Value>>,
    columns: Option<Vec<usize>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetWriteProgress {
    row_count: usize,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FilterTableRow {
    values: Vec<serde_json::Value>,
    display_values: Vec<String>,
    index: usize,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ColumnUniqueValue {
    value: serde_json::Value,
    display_value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilterTableData {
    fields: Vec<String>,
    rows: Vec<FilterTableRow>,
    column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
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

#[tauri::command]
async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<DatasetInfo>, Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    let datasets = dataset_manager
        .list_datasets()
        .await
        .context("Failed to list datasets.")?;

    let dataset_info: Vec<DatasetInfo> = datasets
        .into_iter()
        .map(|record| DatasetInfo {
            id: record.id,
            name: record.metadata.name,
            description: record.metadata.description,
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

#[tauri::command]
async fn dataset_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetDataOptions,
) -> Result<Response, Error> {
    let dataset = state.dataset(id).await?;
    let start = options.start.map_or(Bound::Unbounded, Bound::Included);
    let end = options.end.map_or(Bound::Unbounded, Bound::Excluded);
    let index_filters = options
        .index_filters
        .map(|filter_map| -> Result<_, anyhow::Error> {
            // Build filter schema from the dataset schema (only include filter fields)
            let arrow_schema = dataset.arrow_schema();
            let indices: Vec<usize> = filter_map
                .keys()
                .map(|field_name| {
                    arrow_schema
                        .index_of(field_name)
                        .with_context(|| format!("Field '{}' not found in schema", field_name))
                })
                .collect::<Result<_, _>>()?;
            let filter_schema = std::sync::Arc::new(arrow_schema.project(&indices)?);

            // Convert HashMap to JSON array (single row)
            let json_row = serde_json::Value::Object(filter_map.into_iter().collect());
            // serialize as single object (NDJSON style) for arrow_json Reader
            let json_array = serde_json::to_vec(&json_row)?;

            // Use arrow_json::ReaderBuilder to decode with schema
            let mut reader = arrow_json::ReaderBuilder::new(filter_schema)
                .build(std::io::Cursor::new(json_array))
                .context("Failed to create JSON reader")?;
            reader
                .next()
                .context("No batch returned")?
                .context("Failed to decode filter batch")
        })
        .transpose()
        .context("Failed to decode index filters.")?;
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
    x_column_name: Option<String>,
}

#[tauri::command]
async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<FilterTableData, Error> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();

    // Get index column indices
    let index_col_indices = match index_columns {
        Some(indices) => indices,
        None => {
            return Ok(FilterTableData {
                fields: vec![],
                rows: vec![],
                column_unique_values: HashMap::new(),
            });
        }
    };

    // Filter out X column from index columns
    let x_column_name = options.x_column_name.as_deref();
    let filtered_indices: Vec<usize> = index_col_indices
        .iter()
        .filter(|&&i| {
            let col_name = schema.columns().keys().nth(i).map(String::as_str);
            col_name != x_column_name
        })
        .copied()
        .collect();

    if filtered_indices.is_empty() {
        return Ok(FilterTableData {
            fields: vec![],
            rows: vec![],
            column_unique_values: HashMap::new(),
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

    // Process rows: deduplicate and compute unique values
    let mut unique_rows: Vec<FilterTableRow> = Vec::new();
    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut column_values: HashMap<String, Vec<ColumnUniqueValue>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();

    let format_json_value = |value: &serde_json::Value| match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };

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
            unique_rows.push(FilterTableRow {
                values: values.clone(),
                display_values,
                index: global_row_idx,
            });

            // Collect unique values per column
            for (col_idx, value) in values.iter().enumerate() {
                if let Some(field_name) = fields.get(col_idx) {
                    let display_value = format_json_value(value);
                    let unique_value = ColumnUniqueValue {
                        value: value.clone(),
                        display_value,
                    };
                    if let Some(col_values) = column_values.get_mut(field_name) {
                        if !col_values.contains(&unique_value) {
                            col_values.push(unique_value);
                        }
                    }
                }
            }
        }
    }

    Ok(FilterTableData {
        fields,
        rows: unique_rows,
        column_unique_values: column_values,
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
        get_filter_table_data,
        subscribe_dataset_update,
        unsubscribe_dataset_update
    ]
}

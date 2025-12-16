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
use arrow_ipc::{reader::StreamReader, writer::FileWriter};
use base64::prelude::*;
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
    index_filters: Option<String>,
    columns: Option<Vec<usize>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetWriteProgress {
    row_count: usize,
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
        .map(|t| -> Result<_, anyhow::Error> {
            let buffer = BASE64_STANDARD
                .decode(t)
                .context("Failed to decode base64 string.")?;
            let mut reader = StreamReader::try_new(buffer.as_slice(), None)?;
            Ok(reader.next().context("No RecordBatch.")??)
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
        subscribe_dataset_update,
        unsubscribe_dataset_update
    ]
}

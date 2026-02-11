#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::{collections::HashMap, io::Cursor, ops::Bound, path::Path, sync::Arc};

use anyhow::Context;
use arrow_array::RecordBatch;
use arrow_ipc::writer::FileWriter;
use arrow_select::concat::concat_batches;
use chrono::{DateTime, Utc};
use fricon::{
    DatasetDataType, DatasetId, DatasetListQuery, DatasetSchema, DatasetSortBy, DatasetStatus,
    DatasetUpdate, SelectOptions, SortDirection,
};
use serde::{Deserialize, Serialize};
use tauri::{
    State,
    ipc::{Invoke, Response},
};
use tauri_specta::{Builder, collect_commands, collect_events};

use super::AppState;
use crate::models::{
    chart::{
        DataResponse, DatasetChartDataOptions, HeatmapChartDataOptions, LineChartDataOptions,
        ScatterChartDataOptions, ScatterModeOptions, build_heatmap_series, build_line_series,
        build_scatter_series,
    },
    filter::{DataInternal, TableData, process_filter_rows},
};

#[derive(Debug, Clone, Serialize, specta::Type, thiserror::Error)]
#[error("{message}")]
struct Error {
    message: String,
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, specta::Type)]
pub enum UiDatasetStatus {
    Writing,
    Completed,
    Aborted,
}

impl From<DatasetStatus> for UiDatasetStatus {
    fn from(value: DatasetStatus) -> Self {
        match value {
            DatasetStatus::Writing => Self::Writing,
            DatasetStatus::Completed => Self::Completed,
            DatasetStatus::Aborted => Self::Aborted,
        }
    }
}

impl From<UiDatasetStatus> for DatasetStatus {
    fn from(value: UiDatasetStatus) -> Self {
        match value {
            UiDatasetStatus::Writing => Self::Writing,
            UiDatasetStatus::Completed => Self::Completed,
            UiDatasetStatus::Aborted => Self::Aborted,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
enum UiDatasetSortBy {
    Id,
    Name,
    CreatedAt,
}

impl From<UiDatasetSortBy> for DatasetSortBy {
    fn from(value: UiDatasetSortBy) -> Self {
        match value {
            UiDatasetSortBy::Id => Self::Id,
            UiDatasetSortBy::Name => Self::Name,
            UiDatasetSortBy::CreatedAt => Self::CreatedAt,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
enum UiSortDirection {
    Asc,
    Desc,
}

impl From<UiSortDirection> for SortDirection {
    fn from(value: UiSortDirection) -> Self {
        match value {
            UiSortDirection::Asc => Self::Asc,
            UiSortDirection::Desc => Self::Desc,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DatasetInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub status: UiDatasetStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct DatasetCreated(pub DatasetInfo);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct DatasetUpdated(pub DatasetInfo);

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct WorkspaceInfo {
    path: String,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct ColumnInfo {
    name: String,
    is_complex: bool,
    is_trace: bool,
    is_index: bool,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetDetail {
    id: i32,
    name: String,
    description: String,
    favorite: bool,
    tags: Vec<String>,
    status: UiDatasetStatus,
    created_at: DateTime<Utc>,
    columns: Vec<ColumnInfo>,
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetDataOptions {
    start: Option<usize>,
    end: Option<usize>,
    /// Indices of chosen values for each filter field
    index_filters: Option<Vec<usize>>,
    exclude_columns: Option<Vec<String>>,
    columns: Option<Vec<usize>>,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetWriteStatus {
    row_count: usize,
    is_complete: bool,
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

fn resolve_series_column(
    schema: &DatasetSchema,
    series_name: &str,
) -> Result<(usize, DatasetDataType), Error> {
    let series_index = column_index(schema, series_name)?;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    Ok((series_index, data_type))
}

fn build_line_selected_columns(
    schema: &DatasetSchema,
    options: &LineChartDataOptions,
) -> Result<Vec<usize>, Error> {
    let mut selected = Vec::new();
    let (series_index, data_type) = resolve_series_column(schema, &options.series)?;
    push_column(&mut selected, series_index);
    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        let x_name = options
            .x_column
            .as_ref()
            .context("Line chart requires x column")?;
        let x_index = column_index(schema, x_name)?;
        push_column(&mut selected, x_index);
    }
    Ok(selected)
}

fn build_heatmap_selected_columns(
    schema: &DatasetSchema,
    options: &HeatmapChartDataOptions,
) -> Result<Vec<usize>, Error> {
    let mut selected = Vec::new();
    let (series_index, data_type) = resolve_series_column(schema, &options.series)?;
    push_column(&mut selected, series_index);

    let y_index = column_index(schema, &options.y_column)?;
    push_column(&mut selected, y_index);

    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        let x_name = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        let x_index = column_index(schema, x_name)?;
        push_column(&mut selected, x_index);
    }

    Ok(selected)
}

fn build_scatter_selected_columns(
    schema: &DatasetSchema,
    options: &ScatterChartDataOptions,
) -> Result<Vec<usize>, Error> {
    let mut selected = Vec::new();
    match &options.scatter {
        ScatterModeOptions::Complex { series } => {
            push_column(&mut selected, column_index(schema, series)?);
        }
        ScatterModeOptions::TraceXy {
            trace_x_column,
            trace_y_column,
        } => {
            push_column(&mut selected, column_index(schema, trace_x_column)?);
            push_column(&mut selected, column_index(schema, trace_y_column)?);
        }
        ScatterModeOptions::Xy {
            x_column,
            y_column,
            bin_column,
        } => {
            push_column(&mut selected, column_index(schema, x_column)?);
            push_column(&mut selected, column_index(schema, y_column)?);
            if let Some(bin_name) = bin_column.as_ref() {
                push_column(&mut selected, column_index(schema, bin_name)?);
            }
        }
    }
    Ok(selected)
}

fn build_chart_selected_columns(
    schema: &DatasetSchema,
    options: &DatasetChartDataOptions,
) -> Result<Vec<usize>, Error> {
    match options {
        DatasetChartDataOptions::Line(options) => build_line_selected_columns(schema, options),
        DatasetChartDataOptions::Heatmap(options) => build_heatmap_selected_columns(schema, options),
        DatasetChartDataOptions::Scatter(options) => build_scatter_selected_columns(schema, options),
    }
}

#[tauri::command]
#[specta::specta]
async fn dataset_chart_data(
    state: State<'_, AppState>,
    id: i32,
    options: DatasetChartDataOptions,
) -> Result<DataResponse, Error> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let common = options.common();
    let start = common.start.map_or(Bound::Unbounded, Bound::Included);
    let end = common.end.map_or(Bound::Unbounded, Bound::Excluded);
    let index_filters = if let Some(indices) = common.index_filters.clone() {
        build_filter_batch(
            &state,
            id,
            common.exclude_columns.clone(),
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
        concat_batches(&output_schema, &batches).context("Failed to concat batches")?
    };

    match &options {
        DatasetChartDataOptions::Line(options) => {
            build_line_series(&batch, schema, options).map_err(Error::from)
        }
        DatasetChartDataOptions::Heatmap(options) => {
            build_heatmap_series(&batch, schema, options).map_err(Error::from)
        }
        DatasetChartDataOptions::Scatter(options) => {
            build_scatter_series(&batch, schema, options).map_err(Error::from)
        }
    }
}

#[tauri::command]
#[specta::specta]
async fn get_workspace_info(state: State<'_, AppState>) -> Result<WorkspaceInfo, Error> {
    let app = state.app();
    let workspace_paths = app.paths().context("Failed to retrieve workspace paths.")?;
    let workspace_path = workspace_paths.root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
    })
}

#[derive(Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetListOptions {
    search: Option<String>,
    tags: Option<Vec<String>>,
    favorite_only: Option<bool>,
    statuses: Option<Vec<UiDatasetStatus>>,
    sort_by: Option<UiDatasetSortBy>,
    sort_dir: Option<UiSortDirection>,
    limit: Option<i64>,
    offset: Option<i64>,
}

fn validate_non_negative(value: Option<i64>, field_name: &str) -> Result<Option<i64>, Error> {
    match value {
        Some(v) if v < 0 => Err(anyhow::anyhow!("{field_name} must be non-negative").into()),
        _ => Ok(value),
    }
}

#[tauri::command]
#[specta::specta]
async fn list_datasets(
    state: State<'_, AppState>,
    options: Option<DatasetListOptions>,
) -> Result<Vec<DatasetInfo>, Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    let options = options.unwrap_or_default();
    let query = DatasetListQuery {
        search: options.search,
        tags: options.tags,
        favorite_only: options.favorite_only.unwrap_or(false),
        statuses: options
            .statuses
            .map(|statuses| statuses.into_iter().map(Into::into).collect()),
        sort_by: options.sort_by.map_or(DatasetSortBy::Id, Into::into),
        sort_direction: options.sort_dir.map_or(SortDirection::Desc, Into::into),
        limit: validate_non_negative(options.limit, "limit")?,
        offset: validate_non_negative(options.offset, "offset")?,
    };
    let datasets = dataset_manager
        .list_datasets(query)
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
            status: record.metadata.status.into(),
            created_at: record.metadata.created_at,
        })
        .collect();

    Ok(dataset_info)
}

#[tauri::command]
#[specta::specta]
async fn list_dataset_tags(state: State<'_, AppState>) -> Result<Vec<String>, Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    dataset_manager
        .list_dataset_tags()
        .await
        .context("Failed to list dataset tags.")
        .map_err(Error::from)
}

#[tauri::command]
#[specta::specta]
async fn dataset_detail(state: State<'_, AppState>, id: i32) -> Result<DatasetDetail, Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();
    let record = dataset_manager
        .get_dataset(DatasetId::Id(id))
        .await
        .context("Failed to load dataset metadata.")?;
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
    Ok(DatasetDetail {
        id: record.id,
        name: record.metadata.name,
        description: record.metadata.description,
        favorite: record.metadata.favorite,
        tags: record.metadata.tags,
        status: record.metadata.status.into(),
        created_at: record.metadata.created_at,
        columns,
    })
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetFavoriteUpdate {
    favorite: bool,
}

#[tauri::command]
#[specta::specta]
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

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct DatasetInfoUpdate {
    name: Option<String>,
    description: Option<String>,
    favorite: Option<bool>,
    tags: Option<Vec<String>>,
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut unique = std::collections::BTreeSet::new();
    for tag in tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use fricon::{DatasetDataType, DatasetSchema, ScalarKind};
    use indexmap::IndexMap;

    use super::{build_chart_selected_columns, normalize_tags};
    use crate::models::chart::{
        ChartCommonOptions, DatasetChartDataOptions, HeatmapChartDataOptions,
        LineChartDataOptions, ScatterChartDataOptions, ScatterModeOptions,
    };

    fn numeric_schema() -> DatasetSchema {
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
        DatasetSchema::new(columns)
    }

    #[test]
    fn normalize_tags_trims_dedupes_and_sorts() {
        let input = vec![
            " beta".to_string(),
            "alpha".to_string(),
            "alpha".to_string(),
            String::new(),
            "  ".to_string(),
            "gamma".to_string(),
            "beta".to_string(),
        ];

        let normalized = normalize_tags(input);

        assert_eq!(
            normalized,
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
        );
    }

    #[test]
    fn build_chart_selected_columns_line() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Line(LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: None,
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![1, 0]);
    }

    #[test]
    fn build_chart_selected_columns_heatmap() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Heatmap(HeatmapChartDataOptions {
            series: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![2, 1, 0]);
    }

    #[test]
    fn build_chart_selected_columns_scatter_xy() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Scatter(ScatterChartDataOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: Some("z".to_string()),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![0, 1, 2]);
    }
}

#[tauri::command]
#[specta::specta]
async fn update_dataset_info(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), Error> {
    let app = state.app();
    let dataset_manager = app.dataset_manager();

    let current = dataset_manager
        .get_dataset(DatasetId::Id(id))
        .await
        .context("Failed to load current dataset metadata.")?;

    dataset_manager
        .update_dataset(
            id,
            DatasetUpdate {
                name: update.name,
                description: update.description,
                favorite: update.favorite,
            },
        )
        .await
        .context("Failed to update dataset metadata.")?;

    if let Some(next_tags_raw) = update.tags {
        let next_tags = normalize_tags(next_tags_raw);
        let current_tags: std::collections::BTreeSet<_> =
            current.metadata.tags.into_iter().collect();
        let next_tags_set: std::collections::BTreeSet<_> = next_tags.into_iter().collect();

        let to_add: Vec<String> = next_tags_set.difference(&current_tags).cloned().collect();
        let to_remove: Vec<String> = current_tags.difference(&next_tags_set).cloned().collect();

        if !to_add.is_empty() {
            dataset_manager
                .add_tags(id, to_add)
                .await
                .context("Failed to add dataset tags.")?;
        }

        if !to_remove.is_empty() {
            dataset_manager
                .remove_tags(id, to_remove)
                .await
                .context("Failed to remove dataset tags.")?;
        }
    }

    Ok(())
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct FilterTableOptions {
    exclude_columns: Option<Vec<String>>,
}

async fn get_filter_data_internal(
    state: &AppState,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<DataInternal, Error> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();

    let Some(index_col_indices) = index_columns else {
        return Ok(DataInternal {
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
        return Ok(DataInternal {
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
    Ok(DataInternal {
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
#[specta::specta]
async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, Error> {
    let filter_data = get_filter_data_internal(&state, id, options.exclude_columns).await?;

    Ok(TableData {
        fields: filter_data.fields,
        rows: filter_data.unique_rows,
        column_unique_values: filter_data.column_unique_values,
    })
}

#[tauri::command]
#[specta::specta]
async fn get_dataset_write_status(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetWriteStatus, Error> {
    let dataset = state.dataset(id).await?;
    let (row_count, is_complete) = dataset.write_status();
    Ok(DatasetWriteStatus {
        row_count,
        is_complete,
    })
}

fn specta_builder<R: tauri::Runtime>() -> Builder<R> {
    Builder::<R>::new()
        .commands(collect_commands![
            get_workspace_info,
            list_datasets,
            list_dataset_tags,
            dataset_detail,
            dataset_chart_data,
            get_filter_table_data,
            update_dataset_favorite,
            update_dataset_info,
            get_dataset_write_status
        ])
        .events(collect_events![DatasetCreated, DatasetUpdated])
        .typ::<DatasetInfo>()
}

pub fn export_bindings(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let language = specta_typescript::Typescript::default()
        .header("// @ts-nocheck")
        .bigint(specta_typescript::BigIntExportBehavior::Number);
    specta_builder::<tauri::Wry>()
        .export(language, path)
        .map_err(|err| anyhow::anyhow!("Failed to export TypeScript bindings: {err}"))
}

pub fn mount_typed_events(app: &tauri::AppHandle) {
    specta_builder::<tauri::Wry>().mount_events(app);
}

pub fn invoke_handler() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![
        get_workspace_info,
        list_datasets,
        list_dataset_tags,
        dataset_detail,
        dataset_data,
        dataset_chart_data,
        get_filter_table_data,
        update_dataset_favorite,
        update_dataset_info,
        get_dataset_write_status
    ]
}

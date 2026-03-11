use anyhow::Context;
use fricon::{
    DatasetDataType, DatasetId, DatasetListQuery, DatasetSortBy, DatasetUpdate, SortDirection,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::TauriCommandError;
use crate::{
    AppState,
    dataset_browser::{DatasetInfo, UiDatasetSortBy, UiDatasetStatus, UiSortDirection},
    tauri_api::commands::tags::normalize_tags,
};

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceInfo {
    path: String,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnInfo {
    name: String,
    is_complex: bool,
    is_trace: bool,
    is_index: bool,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetDetail {
    id: i32,
    name: String,
    description: String,
    favorite: bool,
    tags: Vec<String>,
    status: UiDatasetStatus,
    created_at: chrono::DateTime<chrono::Utc>,
    columns: Vec<ColumnInfo>,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetWriteStatus {
    row_count: usize,
    is_complete: bool,
}

#[derive(Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetListOptions {
    #[specta(optional)]
    search: Option<String>,
    #[specta(optional)]
    tags: Option<Vec<String>>,
    #[specta(optional)]
    favorite_only: Option<bool>,
    #[specta(optional)]
    statuses: Option<Vec<UiDatasetStatus>>,
    #[specta(optional)]
    sort_by: Option<UiDatasetSortBy>,
    #[specta(optional)]
    sort_dir: Option<UiSortDirection>,
    #[specta(optional)]
    limit: Option<i64>,
    #[specta(optional)]
    offset: Option<i64>,
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetFavoriteUpdate {
    favorite: bool,
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetInfoUpdate {
    #[specta(optional)]
    name: Option<String>,
    #[specta(optional)]
    description: Option<String>,
    #[specta(optional)]
    favorite: Option<bool>,
    #[specta(optional)]
    tags: Option<Vec<String>>,
}

fn validate_non_negative(
    value: Option<i64>,
    field_name: &str,
) -> Result<Option<i64>, TauriCommandError> {
    match value {
        Some(v) if v < 0 => Err(anyhow::anyhow!("{field_name} must be non-negative").into()),
        _ => Ok(value),
    }
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_workspace_info(
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, TauriCommandError> {
    let app = state.app();
    let workspace_paths = app.paths().context("Failed to retrieve workspace paths.")?;
    let workspace_path = workspace_paths.root();

    Ok(WorkspaceInfo {
        path: workspace_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_datasets(
    state: State<'_, AppState>,
    options: Option<DatasetListOptions>,
) -> Result<Vec<DatasetInfo>, TauriCommandError> {
    let app = state.app();
    let dataset_catalog = app.dataset_catalog();
    let options = options.unwrap_or_default();
    let query = DatasetListQuery {
        search: options.search,
        tags: options.tags,
        favorite_only: options.favorite_only.unwrap_or(false),
        statuses: options
            .statuses
            .map(|statuses: Vec<UiDatasetStatus>| statuses.into_iter().map(Into::into).collect()),
        sort_by: options.sort_by.map_or(DatasetSortBy::Id, Into::into),
        sort_direction: options.sort_dir.map_or(SortDirection::Desc, Into::into),
        limit: validate_non_negative(options.limit, "limit")?,
        offset: validate_non_negative(options.offset, "offset")?,
    };
    let datasets = dataset_catalog
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
pub(crate) async fn list_dataset_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, TauriCommandError> {
    let app = state.app();
    let dataset_catalog = app.dataset_catalog();
    dataset_catalog
        .list_dataset_tags()
        .await
        .context("Failed to list dataset tags.")
        .map_err(TauriCommandError::from)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn dataset_detail(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetDetail, TauriCommandError> {
    let app = state.app();
    let dataset_catalog = app.dataset_catalog();
    let record = dataset_catalog
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

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_favorite(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetFavoriteUpdate,
) -> Result<(), TauriCommandError> {
    let app = state.app();
    let dataset_catalog = app.dataset_catalog();
    dataset_catalog
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

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_info(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), TauriCommandError> {
    let app = state.app();
    let dataset_catalog = app.dataset_catalog();

    let current = dataset_catalog
        .get_dataset(DatasetId::Id(id))
        .await
        .context("Failed to load current dataset metadata.")?;

    dataset_catalog
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
            dataset_catalog
                .add_tags(id, to_add)
                .await
                .context("Failed to add dataset tags.")?;
        }

        if !to_remove.is_empty() {
            dataset_catalog
                .remove_tags(id, to_remove)
                .await
                .context("Failed to remove dataset tags.")?;
        }
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_dataset_write_status(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetWriteStatus, TauriCommandError> {
    let dataset = state.dataset(id).await?;
    let (row_count, is_complete) = dataset.write_status();
    Ok(DatasetWriteStatus {
        row_count,
        is_complete,
    })
}

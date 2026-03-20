use chrono::{DateTime, Utc};
use fricon::{DatasetListQuery, DatasetRecord, DatasetSortBy, DatasetStatus, SortDirection};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::TauriCommandError;
use crate::{application::datasets as app, desktop_runtime::app_state::AppState};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, specta::Type)]
pub(crate) enum UiDatasetStatus {
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
pub(crate) enum UiDatasetSortBy {
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
pub(crate) enum UiSortDirection {
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

#[derive(Debug, Deserialize, Default, specta::Type)]
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
    trashed: Option<bool>,
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

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetInfoUpdate {
    #[specta(optional)]
    pub(crate) name: Option<String>,
    #[specta(optional)]
    pub(crate) description: Option<String>,
    #[specta(optional)]
    pub(crate) favorite: Option<bool>,
    #[specta(optional)]
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetInfo {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) favorite: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) status: UiDatasetStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) trashed_at: Option<DateTime<Utc>>,
}

impl From<&DatasetRecord> for DatasetInfo {
    fn from(record: &DatasetRecord) -> Self {
        Self {
            id: record.id,
            name: record.metadata.name.clone(),
            description: record.metadata.description.clone(),
            favorite: record.metadata.favorite,
            tags: record.metadata.tags.clone(),
            status: record.metadata.status.into(),
            created_at: record.metadata.created_at,
            trashed_at: record.metadata.trashed_at,
        }
    }
}

impl From<DatasetRecord> for DatasetInfo {
    fn from(record: DatasetRecord) -> Self {
        Self::from(&record)
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnInfo {
    pub(crate) name: String,
    pub(crate) is_complex: bool,
    pub(crate) is_trace: bool,
    pub(crate) is_index: bool,
}

impl From<app::ColumnInfo> for ColumnInfo {
    fn from(value: app::ColumnInfo) -> Self {
        Self {
            name: value.name,
            is_complex: value.is_complex,
            is_trace: value.is_trace,
            is_index: value.is_index,
        }
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetDetail {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) favorite: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) status: UiDatasetStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) trashed_at: Option<DateTime<Utc>>,
    pub(crate) columns: Vec<ColumnInfo>,
}

impl From<app::DatasetDetail> for DatasetDetail {
    fn from(value: app::DatasetDetail) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            favorite: value.favorite,
            tags: value.tags,
            status: value.status.into(),
            created_at: value.created_at,
            trashed_at: value.trashed_at,
            columns: value.columns.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmptyTrashResult {
    pub(crate) deleted_count: usize,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetWriteStatus {
    pub(crate) row_count: usize,
    pub(crate) is_complete: bool,
}

impl From<app::DatasetWriteStatus> for DatasetWriteStatus {
    fn from(value: app::DatasetWriteStatus) -> Self {
        Self {
            row_count: value.row_count,
            is_complete: value.is_complete,
        }
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetDeleteResult {
    pub(crate) id: i32,
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

impl From<app::DatasetDeleteResult> for DatasetDeleteResult {
    fn from(value: app::DatasetDeleteResult) -> Self {
        Self {
            id: value.id,
            success: value.success,
            error: value.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetCreated(pub(crate) DatasetInfo);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetUpdated(pub(crate) DatasetInfo);

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_datasets(
    state: State<'_, AppState>,
    options: Option<DatasetListOptions>,
) -> Result<Vec<DatasetInfo>, TauriCommandError> {
    let options = options.unwrap_or_default();
    let query = DatasetListQuery {
        search: options.search,
        tags: options.tags,
        favorite_only: options.favorite_only.unwrap_or(false),
        statuses: options
            .statuses
            .map(|statuses: Vec<UiDatasetStatus>| statuses.into_iter().map(Into::into).collect()),
        trashed: options.trashed.or(Some(false)),
        sort_by: options.sort_by.map_or(DatasetSortBy::Id, Into::into),
        sort_direction: options.sort_dir.map_or(SortDirection::Desc, Into::into),
        limit: app::validate_non_negative(options.limit, "limit")?,
        offset: app::validate_non_negative(options.offset, "offset")?,
    };
    let datasets = app::list_datasets(state.session(), query).await?;
    Ok(datasets.into_iter().map(DatasetInfo::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_dataset_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, TauriCommandError> {
    app::list_dataset_tags(state.session())
        .await
        .map_err(TauriCommandError::from)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn dataset_detail(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetDetail, TauriCommandError> {
    Ok(app::get_dataset_detail(state.session(), id).await?.into())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_favorite(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetFavoriteUpdate,
) -> Result<(), TauriCommandError> {
    app::update_dataset_favorite(state.session(), id, update.favorite).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_info(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), TauriCommandError> {
    app::update_dataset_info(
        state.session(),
        id,
        app::DatasetInfoUpdate {
            name: update.name,
            description: update.description,
            favorite: update.favorite,
            tags: update.tags,
        },
    )
    .await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_dataset_write_status(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetWriteStatus, TauriCommandError> {
    Ok(app::get_dataset_write_status(state.session(), id)
        .await?
        .into())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn delete_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, TauriCommandError> {
    let results = app::delete_datasets(state.session(), ids).await;
    Ok(results.into_iter().map(DatasetDeleteResult::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn trash_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, TauriCommandError> {
    let results = app::trash_datasets(state.session(), ids).await;
    Ok(results.into_iter().map(DatasetDeleteResult::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn restore_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, TauriCommandError> {
    let results = app::restore_datasets(state.session(), ids).await;
    Ok(results.into_iter().map(DatasetDeleteResult::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn empty_trash(
    state: State<'_, AppState>,
) -> Result<EmptyTrashResult, TauriCommandError> {
    let deleted_count = app::empty_trash(state.session()).await?;
    Ok(EmptyTrashResult { deleted_count })
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchTagUpdateOptions {
    pub(crate) ids: Vec<i32>,
    #[specta(optional)]
    pub(crate) add: Option<Vec<String>>,
    #[specta(optional)]
    pub(crate) remove: Option<Vec<String>>,
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn batch_update_dataset_tags(
    state: State<'_, AppState>,
    update: BatchTagUpdateOptions,
) -> Result<Vec<DatasetDeleteResult>, TauriCommandError> {
    let results = app::batch_update_dataset_tags(
        state.session(),
        app::BatchTagUpdate {
            ids: update.ids,
            add: update.add.unwrap_or_default(),
            remove: update.remove.unwrap_or_default(),
        },
    )
    .await;
    Ok(results.into_iter().map(DatasetDeleteResult::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn delete_tag(
    state: State<'_, AppState>,
    tag: String,
) -> Result<(), TauriCommandError> {
    app::delete_tag(state.session(), tag).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn rename_tag(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), TauriCommandError> {
    app::rename_tag(state.session(), old_name, new_name).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn merge_tag(
    state: State<'_, AppState>,
    source: String,
    target: String,
) -> Result<(), TauriCommandError> {
    app::merge_tag(state.session(), source, target).await?;
    Ok(())
}

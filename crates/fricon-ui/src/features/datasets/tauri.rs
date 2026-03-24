use fricon::{DatasetListQuery, DatasetSortBy, SortDirection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    desktop_runtime::app_state::AppState,
    features::datasets::{
        mutations, queries, transfer,
        types::{
            DatasetDeleteResult, DatasetDetail, DatasetInfo, DatasetInfoUpdate, DatasetWriteStatus,
            PreviewImportResult, UiDatasetStatus, UiImportPreview,
        },
    },
    tauri_api::ApiError,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetCreated(pub(crate) DatasetInfo);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetUpdated(pub(crate) DatasetInfo);

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_datasets(
    state: State<'_, AppState>,
    options: Option<DatasetListOptions>,
) -> Result<Vec<DatasetInfo>, ApiError> {
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
        limit: queries::validate_non_negative(options.limit, "limit")
            .map_err(ApiError::validation)?,
        offset: queries::validate_non_negative(options.offset, "offset")
            .map_err(ApiError::validation)?,
    };
    queries::list_datasets(state.session(), query)
        .await
        .map_err(ApiError::from_dataset_error)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_dataset_tags(state: State<'_, AppState>) -> Result<Vec<String>, ApiError> {
    queries::list_dataset_tags(state.session())
        .await
        .map_err(ApiError::from_dataset_error)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn dataset_detail(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetDetail, ApiError> {
    queries::get_dataset_detail(state.session(), id)
        .await
        .map_err(ApiError::from_dataset_error)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_favorite(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetFavoriteUpdate,
) -> Result<(), ApiError> {
    mutations::update_dataset_favorite(state.session(), id, update.favorite)
        .await
        .map_err(ApiError::from_dataset_error)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_info(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), ApiError> {
    mutations::update_dataset_info(state.session(), id, update)
        .await
        .map_err(ApiError::from_dataset_error)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_dataset_write_status(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetWriteStatus, ApiError> {
    queries::get_dataset_write_status(state.session(), id)
        .await
        .map_err(ApiError::from_dataset_error)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn delete_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, ApiError> {
    Ok(mutations::delete_datasets(state.session(), ids).await)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn trash_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, ApiError> {
    Ok(mutations::trash_datasets(state.session(), ids).await)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn restore_datasets(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Vec<DatasetDeleteResult>, ApiError> {
    Ok(mutations::restore_datasets(state.session(), ids).await)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn empty_trash(
    state: State<'_, AppState>,
) -> Result<Vec<DatasetDeleteResult>, ApiError> {
    mutations::empty_trash(state.session())
        .await
        .map_err(ApiError::from_dataset_error)
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
) -> Result<Vec<DatasetDeleteResult>, ApiError> {
    Ok(mutations::batch_update_dataset_tags(
        state.session(),
        mutations::BatchTagUpdate {
            ids: update.ids,
            add: update.add.unwrap_or_default(),
            remove: update.remove.unwrap_or_default(),
        },
    )
    .await)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn delete_tag(state: State<'_, AppState>, tag: String) -> Result<(), ApiError> {
    mutations::delete_tag(state.session(), tag)
        .await
        .map_err(ApiError::from_dataset_error)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn rename_tag(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), ApiError> {
    mutations::rename_tag(state.session(), old_name, new_name)
        .await
        .map_err(ApiError::from_dataset_error)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn merge_tag(
    state: State<'_, AppState>,
    source: String,
    target: String,
) -> Result<(), ApiError> {
    mutations::merge_tag(state.session(), source, target)
        .await
        .map_err(ApiError::from_dataset_error)?;
    Ok(())
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiPreviewImportResult {
    pub(crate) archive_path: String,
    pub(crate) preview: UiImportPreview,
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn export_datasets_dialog(
    state: State<'_, AppState>,
    ids: Vec<i32>,
) -> Result<Option<Vec<String>>, ApiError> {
    let result = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Export Datasets")
            .pick_folder()
    })
    .await
    .map_err(|e| ApiError::dialog(format!("Failed to open dialog: {e}")))?;

    if let Some(path) = result {
        let out_paths = transfer::export_datasets(state.session(), ids, path)
            .await
            .map_err(ApiError::from_dataset_error)?
            .into_iter()
            .map(|out_path| out_path.to_string_lossy().to_string())
            .collect();
        Ok(Some(out_paths))
    } else {
        Ok(None)
    }
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn preview_import_dialog(
    state: State<'_, AppState>,
) -> Result<Option<Vec<UiPreviewImportResult>>, ApiError> {
    let result = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Import Datasets")
            .add_filter("Archive", &["tar.zst"])
            .pick_files()
    })
    .await
    .map_err(|e| ApiError::dialog(format!("Failed to open dialog: {e}")))?;

    if let Some(paths) = result {
        let previews = transfer::preview_import_files(state.session(), paths)
            .await
            .map_err(ApiError::from_dataset_error)?
            .into_iter()
            .map(UiPreviewImportResult::from)
            .collect();
        Ok(Some(previews))
    } else {
        Ok(None)
    }
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn preview_import_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<UiPreviewImportResult>, ApiError> {
    transfer::preview_import_files(
        state.session(),
        paths.into_iter().map(std::path::PathBuf::from).collect(),
    )
    .await
    .map(|results| {
        results
            .into_iter()
            .map(UiPreviewImportResult::from)
            .collect()
    })
    .map_err(ApiError::from_dataset_error)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn import_dataset(
    state: State<'_, AppState>,
    archive_path: String,
    force: bool,
) -> Result<DatasetInfo, ApiError> {
    transfer::import_dataset(
        state.session(),
        std::path::PathBuf::from(archive_path),
        force,
    )
    .await
    .map_err(ApiError::from_dataset_error)
}

impl From<PreviewImportResult> for UiPreviewImportResult {
    fn from(value: PreviewImportResult) -> Self {
        Self {
            archive_path: value.archive_path.to_string_lossy().to_string(),
            preview: value.preview.into(),
        }
    }
}

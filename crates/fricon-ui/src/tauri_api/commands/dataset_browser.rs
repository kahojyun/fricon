use fricon::{DatasetListQuery, DatasetSortBy, SortDirection};
use serde::Deserialize;
use tauri::State;

use super::{AppState, TauriCommandError};
use crate::{
    application::{dataset_browser as app_dataset_browser, workspace as app_workspace},
    tauri_api::dataset::{
        DatasetDetail, DatasetInfo, DatasetInfoUpdate, DatasetWriteStatus, UiDatasetSortBy,
        UiDatasetStatus, UiSortDirection, WorkspaceInfo,
    },
};

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

#[tauri::command]
#[specta::specta]
pub(crate) async fn get_workspace_info(
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, TauriCommandError> {
    Ok(app_workspace::get_workspace_info(state.session())?.into())
}

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
        sort_by: options.sort_by.map_or(DatasetSortBy::Id, Into::into),
        sort_direction: options.sort_dir.map_or(SortDirection::Desc, Into::into),
        limit: app_dataset_browser::validate_non_negative(options.limit, "limit")?,
        offset: app_dataset_browser::validate_non_negative(options.offset, "offset")?,
    };
    let datasets = app_dataset_browser::list_datasets(state.session(), query).await?;
    Ok(datasets.into_iter().map(DatasetInfo::from).collect())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn list_dataset_tags(
    state: State<'_, AppState>,
) -> Result<Vec<String>, TauriCommandError> {
    app_dataset_browser::list_dataset_tags(state.session())
        .await
        .map_err(TauriCommandError::from)
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn dataset_detail(
    state: State<'_, AppState>,
    id: i32,
) -> Result<DatasetDetail, TauriCommandError> {
    Ok(app_dataset_browser::get_dataset_detail(state.session(), id)
        .await?
        .into())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_favorite(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetFavoriteUpdate,
) -> Result<(), TauriCommandError> {
    app_dataset_browser::update_dataset_favorite(state.session(), id, update.favorite).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub(crate) async fn update_dataset_info(
    state: State<'_, AppState>,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), TauriCommandError> {
    app_dataset_browser::update_dataset_info(
        state.session(),
        id,
        app_dataset_browser::DatasetInfoUpdate {
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
    Ok(
        app_dataset_browser::get_dataset_write_status(state.session(), id)
            .await?
            .into(),
    )
}

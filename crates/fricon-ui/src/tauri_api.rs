#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use tauri_specta::{Builder, collect_commands, collect_events};

use crate::features::{
    charts::tauri as charts,
    datasets::{
        tauri as datasets,
        tauri::{DatasetCreated, DatasetUpdated},
        types::DatasetInfo,
    },
    workspace::tauri as workspace,
};

#[derive(Debug, Clone, Copy, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ApiErrorCode {
    Workspace,
    Datasets,
    Charts,
    Dialog,
    Validation,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub(crate) struct ApiError {
    code: ApiErrorCode,
    message: String,
}

impl ApiError {
    fn new(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn workspace(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Workspace, error.to_string())
    }

    pub(crate) fn datasets(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Datasets, error.to_string())
    }

    pub(crate) fn charts(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Charts, error.to_string())
    }

    pub(crate) fn dialog(message: impl Into<String>) -> Self {
        Self::new(ApiErrorCode::Dialog, message)
    }

    pub(crate) fn validation(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Validation, error.to_string())
    }
}

pub(crate) fn specta_builder() -> Builder {
    Builder::new()
        .commands(collect_commands![
            workspace::get_workspace_info,
            datasets::list_datasets,
            datasets::list_dataset_tags,
            datasets::dataset_detail,
            charts::dataset_chart_data,
            charts::get_filter_table_data,
            datasets::update_dataset_favorite,
            datasets::update_dataset_info,
            datasets::get_dataset_write_status,
            datasets::delete_datasets,
            datasets::trash_datasets,
            datasets::restore_datasets,
            datasets::empty_trash,
            datasets::batch_update_dataset_tags,
            datasets::delete_tag,
            datasets::rename_tag,
            datasets::merge_tag,
            datasets::export_datasets_dialog,
            datasets::preview_import_dialog,
            datasets::preview_import_files,
            datasets::import_dataset
        ])
        .events(collect_events![DatasetCreated, DatasetUpdated])
        .typ::<DatasetInfo>()
}

pub fn export_bindings(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let language = specta_typescript::Typescript::default()
        .header("// @ts-nocheck")
        .bigint(specta_typescript::BigIntExportBehavior::Number);
    specta_builder()
        .export(language, path)
        .map_err(|err| anyhow::anyhow!("Failed to export TypeScript bindings: {err}"))
}

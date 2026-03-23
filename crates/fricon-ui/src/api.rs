#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use tauri_specta::{Builder, collect_commands, collect_events};

pub(crate) mod charts;
pub(crate) mod datasets;
pub(crate) mod workspace;

use crate::api::datasets::{DatasetCreated, DatasetInfo, DatasetUpdated};

#[derive(Debug, Clone, serde::Serialize, specta::Type, thiserror::Error)]
#[error("{message}")]
pub(crate) struct TauriCommandError {
    message: String,
}

impl From<anyhow::Error> for TauriCommandError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            message: value.to_string(),
        }
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
            datasets::export_dataset_dialog,
            datasets::preview_import_dialog,
            datasets::preview_import_file,
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

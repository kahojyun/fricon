#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use tauri_specta::{Builder, collect_commands, collect_events};

pub(crate) mod chart_data;
pub(crate) mod commands;
pub(crate) mod dataset;
pub(crate) mod filter_table;

use crate::tauri_api::dataset::{DatasetCreated, DatasetInfo, DatasetUpdated};

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
            commands::dataset_browser::get_workspace_info,
            commands::dataset_browser::list_datasets,
            commands::dataset_browser::list_dataset_tags,
            commands::dataset_browser::dataset_detail,
            commands::chart_data::dataset_chart_data,
            commands::filter_data::get_filter_table_data,
            commands::dataset_browser::update_dataset_favorite,
            commands::dataset_browser::update_dataset_info,
            commands::dataset_browser::get_dataset_write_status
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

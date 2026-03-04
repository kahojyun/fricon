#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use tauri_specta::{Builder, collect_commands, collect_events};

mod chart;
mod chart_selection;
mod dataset;
mod filter_data;
mod tags;
mod types;

pub(super) use types::TauriCommandError;
pub(crate) use types::{DatasetCreated, DatasetInfo, DatasetUpdated};

pub(super) use crate::AppState;

pub(crate) fn specta_builder() -> Builder {
    Builder::new()
        .commands(collect_commands![
            dataset::get_workspace_info,
            dataset::list_datasets,
            dataset::list_dataset_tags,
            dataset::dataset_detail,
            chart::dataset_chart_data,
            filter_data::get_filter_table_data,
            dataset::update_dataset_favorite,
            dataset::update_dataset_info,
            dataset::get_dataset_write_status
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

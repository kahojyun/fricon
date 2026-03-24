#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use anyhow::Error as AnyhowError;
use fricon::dataset::{catalog::CatalogError, read::ReadError};
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
    DatasetNotFound,
    DatasetDeleted,
    DatasetNotTrashed,
    InvalidTag,
    SameTagName,
    SameSourceTarget,
    Workspace,
    Charts,
    Dialog,
    Validation,
    Internal,
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

    pub(crate) fn charts(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Charts, error.to_string())
    }

    pub(crate) fn dialog(message: impl Into<String>) -> Self {
        Self::new(ApiErrorCode::Dialog, message)
    }

    pub(crate) fn validation(error: impl std::fmt::Display) -> Self {
        Self::new(ApiErrorCode::Validation, error.to_string())
    }

    pub(crate) fn from_catalog_error(error: &CatalogError) -> Self {
        let code = match error {
            CatalogError::NotFound { .. } => ApiErrorCode::DatasetNotFound,
            CatalogError::Deleted { .. } => ApiErrorCode::DatasetDeleted,
            CatalogError::NotTrashed => ApiErrorCode::DatasetNotTrashed,
            CatalogError::EmptyTag => ApiErrorCode::InvalidTag,
            CatalogError::SameTagName => ApiErrorCode::SameTagName,
            CatalogError::SameSourceTarget => ApiErrorCode::SameSourceTarget,
            CatalogError::StateDropped
            | CatalogError::TaskPanic { .. }
            | CatalogError::TaskCancelled { .. }
            | CatalogError::DatasetFs(_)
            | CatalogError::Database(_)
            | CatalogError::Portability(_) => ApiErrorCode::Internal,
        };
        Self::new(code, error.to_string())
    }

    pub(crate) fn from_read_error(error: &ReadError) -> Self {
        let code = match error {
            ReadError::NotFound { .. } => ApiErrorCode::DatasetNotFound,
            ReadError::Deleted { .. } => ApiErrorCode::DatasetDeleted,
            ReadError::EmptyDataset
            | ReadError::StateDropped
            | ReadError::TaskPanic { .. }
            | ReadError::TaskCancelled { .. }
            | ReadError::Dataset(_)
            | ReadError::DatasetFs(_)
            | ReadError::Database(_) => ApiErrorCode::Internal,
        };
        Self::new(code, error.to_string())
    }

    pub(crate) fn from_dataset_error(error: AnyhowError) -> Self {
        if let Some(error) = error.downcast_ref::<CatalogError>() {
            return Self::from_catalog_error(error);
        }
        if let Some(error) = error.downcast_ref::<ReadError>() {
            return Self::from_read_error(error);
        }
        Self::new(ApiErrorCode::Internal, error.to_string())
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

#[cfg(test)]
mod tests {
    use fricon::dataset::{PortabilityError, catalog::CatalogError, read::ReadError};

    use super::{ApiError, ApiErrorCode};

    #[test]
    fn catalog_not_found_maps_to_dataset_not_found() {
        let error = ApiError::from_catalog_error(&CatalogError::NotFound {
            id: "42".to_string(),
        });
        assert!(matches!(error.code, ApiErrorCode::DatasetNotFound));
    }

    #[test]
    fn catalog_deleted_maps_to_dataset_deleted() {
        let error = ApiError::from_catalog_error(&CatalogError::Deleted {
            id: "42".to_string(),
        });
        assert!(matches!(error.code, ApiErrorCode::DatasetDeleted));
    }

    #[test]
    fn not_trashed_maps_to_dataset_not_trashed() {
        let error = ApiError::from_catalog_error(&CatalogError::NotTrashed);
        assert!(matches!(error.code, ApiErrorCode::DatasetNotTrashed));
    }

    #[test]
    fn empty_tag_maps_to_invalid_tag() {
        let error = ApiError::from_catalog_error(&CatalogError::EmptyTag);
        assert!(matches!(error.code, ApiErrorCode::InvalidTag));
    }

    #[test]
    fn same_tag_name_maps_to_same_tag_name() {
        let error = ApiError::from_catalog_error(&CatalogError::SameTagName);
        assert!(matches!(error.code, ApiErrorCode::SameTagName));
    }

    #[test]
    fn same_source_target_maps_to_same_source_target() {
        let error = ApiError::from_catalog_error(&CatalogError::SameSourceTarget);
        assert!(matches!(error.code, ApiErrorCode::SameSourceTarget));
    }

    #[test]
    fn read_deleted_maps_to_dataset_deleted() {
        let error = ApiError::from_read_error(&ReadError::Deleted {
            id: "42".to_string(),
        });
        assert!(matches!(error.code, ApiErrorCode::DatasetDeleted));
    }

    #[test]
    fn read_not_found_maps_to_dataset_not_found() {
        let error = ApiError::from_read_error(&ReadError::NotFound {
            id: "42".to_string(),
        });
        assert!(matches!(error.code, ApiErrorCode::DatasetNotFound));
    }

    #[test]
    fn internal_dataset_failures_map_to_internal() {
        let error = ApiError::from_catalog_error(&CatalogError::Portability(
            PortabilityError::MissingMetadata,
        ));
        assert!(matches!(error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn dataset_error_downcasts_through_anyhow_context() {
        let error =
            anyhow::Error::new(CatalogError::Portability(PortabilityError::MissingMetadata))
                .context("Failed to list datasets.");
        let api_error = ApiError::from_dataset_error(error);
        assert!(matches!(api_error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn non_dataset_anyhow_errors_fall_back_to_internal() {
        let error = anyhow::Error::new(std::io::Error::other("backend exploded"));
        let api_error = ApiError::from_dataset_error(error);
        assert!(matches!(api_error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn validation_helper_uses_validation_code() {
        let error = ApiError::validation("limit must be non-negative");
        assert!(matches!(error.code, ApiErrorCode::Validation));
    }
}

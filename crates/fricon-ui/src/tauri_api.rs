#![allow(
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    reason = "Tauri command handlers require specific parameter signatures"
)]

use std::path::Path;

use fricon::{
    CatalogAppError, ReadAppError,
    dataset::{catalog::CatalogError, read::ReadError},
};
use tauri_specta::{Builder, collect_commands, collect_events};

use crate::features::{
    charts::tauri as charts,
    datasets::{
        error::UiDatasetError,
        tauri as datasets,
        tauri::DatasetChanged,
        types::{DatasetInfo, DatasetOperationError},
    },
    workspace::tauri as workspace,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ApiErrorCode {
    DatasetNotFound,
    DatasetDeleted,
    DatasetNotTrashed,
    ArchiveVersionUnsupported,
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

    pub(crate) fn into_dataset_operation_error(self) -> DatasetOperationError {
        DatasetOperationError {
            code: self.code,
            message: self.message,
        }
    }
}

impl From<CatalogError> for ApiError {
    fn from(error: CatalogError) -> Self {
        let code = match error {
            CatalogError::NotFound { .. } => ApiErrorCode::DatasetNotFound,
            CatalogError::Deleted { .. } => ApiErrorCode::DatasetDeleted,
            CatalogError::NotTrashed => ApiErrorCode::DatasetNotTrashed,
            CatalogError::Portability(
                fricon::dataset::PortabilityError::UnsupportedArchiveVersion { .. },
            ) => ApiErrorCode::ArchiveVersionUnsupported,
            CatalogError::EmptyTag => ApiErrorCode::InvalidTag,
            CatalogError::SameTagName => ApiErrorCode::SameTagName,
            CatalogError::SameSourceTarget => ApiErrorCode::SameSourceTarget,
            CatalogError::DatasetFs(_)
            | CatalogError::Database(_)
            | CatalogError::Portability(_) => ApiErrorCode::Internal,
        };
        ApiError::new(code, error.to_string())
    }
}

impl From<CatalogAppError> for ApiError {
    fn from(error: CatalogAppError) -> Self {
        match error {
            CatalogAppError::Domain(error) => Self::from(error),
            CatalogAppError::StateDropped
            | CatalogAppError::TaskPanic { .. }
            | CatalogAppError::TaskCancelled { .. } => {
                Self::new(ApiErrorCode::Internal, error.to_string())
            }
        }
    }
}

impl From<ReadError> for ApiError {
    fn from(error: ReadError) -> Self {
        let code = match error {
            ReadError::NotFound { .. } => ApiErrorCode::DatasetNotFound,
            ReadError::Deleted { .. } => ApiErrorCode::DatasetDeleted,
            ReadError::EmptyDataset
            | ReadError::Dataset(_)
            | ReadError::DatasetFs(_)
            | ReadError::Database(_) => ApiErrorCode::Internal,
        };
        ApiError::new(code, error.to_string())
    }
}

impl From<ReadAppError> for ApiError {
    fn from(error: ReadAppError) -> Self {
        match error {
            ReadAppError::Domain(error) => Self::from(error),
            ReadAppError::StateDropped
            | ReadAppError::TaskPanic { .. }
            | ReadAppError::TaskCancelled { .. } => {
                Self::new(ApiErrorCode::Internal, error.to_string())
            }
        }
    }
}

impl From<UiDatasetError> for ApiError {
    fn from(error: UiDatasetError) -> Self {
        match error {
            UiDatasetError::Catalog(error) => Self::from(error),
            UiDatasetError::Read(error) => Self::from(error),
            UiDatasetError::Validation { message } => Self::new(ApiErrorCode::Validation, message),
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
            datasets::export_datasets_dialog,
            datasets::preview_import_dialog,
            datasets::preview_import_files,
            datasets::import_dataset
        ])
        .events(collect_events![DatasetChanged])
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
    use fricon::{
        CatalogAppError, ReadAppError,
        dataset::{PortabilityError, catalog::CatalogError},
    };

    use super::{ApiError, ApiErrorCode};
    use crate::features::datasets::error::UiDatasetError;

    #[test]
    fn catalog_not_found_maps_to_dataset_not_found() {
        let error = ApiError::from(CatalogError::NotFound {
            id: "42".to_string(),
        });
        assert!(matches!(error.code, ApiErrorCode::DatasetNotFound));
    }

    #[test]
    fn internal_dataset_failures_map_to_internal() {
        let error = ApiError::from(CatalogError::Portability(PortabilityError::MissingMetadata));
        assert!(matches!(error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn unsupported_archive_version_maps_to_dedicated_code() {
        let error = ApiError::from(CatalogError::Portability(
            PortabilityError::UnsupportedArchiveVersion {
                found: 2,
                supported: 1,
            },
        ));
        assert!(matches!(
            error.code,
            ApiErrorCode::ArchiveVersionUnsupported
        ));
    }

    #[test]
    fn dataset_error_maps_runtime_failures_to_internal() {
        let error = UiDatasetError::Catalog(CatalogAppError::TaskCancelled {
            operation: "joining catalog task",
        });
        let api_error = ApiError::from(error);
        assert!(matches!(api_error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn catalog_app_error_maps_domain_variants() {
        let error = ApiError::from(CatalogAppError::Domain(CatalogError::Deleted {
            id: "42".to_string(),
        }));
        assert!(matches!(error.code, ApiErrorCode::DatasetDeleted));
    }

    #[test]
    fn read_app_error_maps_runtime_variants_to_internal() {
        let error = ApiError::from(ReadAppError::StateDropped);
        assert!(matches!(error.code, ApiErrorCode::Internal));
    }

    #[test]
    fn validation_dataset_errors_map_to_validation() {
        let error = UiDatasetError::validation("limit must be non-negative");
        let api_error = ApiError::from(error);
        assert!(matches!(api_error.code, ApiErrorCode::Validation));
    }

    #[test]
    fn validation_helper_uses_validation_code() {
        let error = ApiError::validation("limit must be non-negative");
        assert!(matches!(error.code, ApiErrorCode::Validation));
    }
}

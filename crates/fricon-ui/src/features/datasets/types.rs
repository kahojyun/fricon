use std::path::PathBuf;

use chrono::{DateTime, Utc};
use fricon::{DatasetRecord, DatasetStatus};
use serde::{Deserialize, Serialize};

use crate::tauri_api::ApiErrorCode;

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
    pub(crate) deleted_at: Option<DateTime<Utc>>,
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
            deleted_at: record.metadata.deleted_at,
        }
    }
}

impl From<DatasetRecord> for DatasetInfo {
    fn from(record: DatasetRecord) -> Self {
        Self::from(&record)
    }
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnInfo {
    pub(crate) name: String,
    pub(crate) is_complex: bool,
    pub(crate) is_trace: bool,
    pub(crate) is_index: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
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
    pub(crate) deleted_at: Option<DateTime<Utc>>,
    pub(crate) payload_available: bool,
    pub(crate) columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetWriteStatus {
    pub(crate) row_count: usize,
    pub(crate) is_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetOperationError {
    pub(crate) code: ApiErrorCode,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetDeleteResult {
    pub(crate) id: i32,
    pub(crate) success: bool,
    pub(crate) error: Option<DatasetOperationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetTagBatchResult {
    pub(crate) id: i32,
    pub(crate) success: bool,
    pub(crate) add_error: Option<DatasetOperationError>,
    pub(crate) remove_error: Option<DatasetOperationError>,
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiExportedMetadata {
    pub(crate) uid: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) favorite: bool,
    pub(crate) status: UiDatasetStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) tags: Vec<String>,
}

impl From<fricon::dataset::ExportedMetadata> for UiExportedMetadata {
    fn from(value: fricon::dataset::ExportedMetadata) -> Self {
        Self {
            uid: value.uid.to_string(),
            name: value.name,
            description: value.description,
            favorite: value.favorite,
            status: value.status.into(),
            created_at: value.created_at,
            tags: value.tags,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiFieldDiff {
    pub(crate) field: String,
    pub(crate) existing_value: String,
    pub(crate) incoming_value: String,
}

impl From<fricon::dataset::FieldDiff> for UiFieldDiff {
    fn from(value: fricon::dataset::FieldDiff) -> Self {
        Self {
            field: value.field,
            existing_value: value.existing_value,
            incoming_value: value.incoming_value,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiImportConflict {
    pub(crate) existing: UiExportedMetadata,
    pub(crate) diffs: Vec<UiFieldDiff>,
}

impl From<fricon::dataset::ImportConflict> for UiImportConflict {
    fn from(value: fricon::dataset::ImportConflict) -> Self {
        Self {
            existing: value.existing.into(),
            diffs: value.diffs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiImportPreview {
    pub(crate) metadata: UiExportedMetadata,
    pub(crate) conflict: Option<UiImportConflict>,
}

impl From<fricon::dataset::ImportPreview> for UiImportPreview {
    fn from(value: fricon::dataset::ImportPreview) -> Self {
        Self {
            metadata: value.metadata.into(),
            conflict: value.conflict.map(Into::into),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreviewImportResult {
    pub(crate) archive_path: PathBuf,
    pub(crate) preview: fricon::dataset::ImportPreview,
}

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use fricon::{
    DatasetRecord, DatasetStatus, ResolvedSemanticCapabilities, ResolvedSemanticDescriptor,
    SemanticRole, SemanticShapeKind, SemanticValueKind,
};
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
    pub(crate) label: Option<String>,
    pub(crate) unit: Option<String>,
    pub(crate) semantic: ChartSemanticDescriptor,
    pub(crate) capabilities: ChartSemanticCapabilities,
    pub(crate) is_inferred_axis: bool,
    pub(crate) hidden_by_default: bool,
    pub(crate) is_chart_axis_candidate: bool,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartDuplicatePolicy {
    LatestByRecordId,
    RowOrderPlaceholder,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartIndexRealization {
    None,
    Implicit,
    Sidecar,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartSemanticAxisKind {
    LogicalIndex,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartSemanticValueKind {
    Numeric,
    Categorical,
    Boolean,
    Timestamp,
    Complex,
    Display,
}

impl From<SemanticValueKind> for ChartSemanticValueKind {
    fn from(value: SemanticValueKind) -> Self {
        match value {
            SemanticValueKind::Numeric => Self::Numeric,
            SemanticValueKind::Categorical => Self::Categorical,
            SemanticValueKind::Boolean => Self::Boolean,
            SemanticValueKind::Timestamp => Self::Timestamp,
            SemanticValueKind::Complex => Self::Complex,
            SemanticValueKind::Display => Self::Display,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartSemanticShapeKind {
    Scalar,
    Trace,
}

impl From<SemanticShapeKind> for ChartSemanticShapeKind {
    fn from(value: SemanticShapeKind) -> Self {
        match value {
            SemanticShapeKind::Scalar => Self::Scalar,
            SemanticShapeKind::Trace => Self::Trace,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChartSemanticRole {
    Value,
    LogicalIndex,
    System,
    Display,
}

impl From<SemanticRole> for ChartSemanticRole {
    fn from(value: SemanticRole) -> Self {
        match value {
            SemanticRole::Value => Self::Value,
            SemanticRole::LogicalIndex => Self::LogicalIndex,
            SemanticRole::System => Self::System,
            SemanticRole::Display => Self::Display,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartSemanticDescriptor {
    pub(crate) value_kind: ChartSemanticValueKind,
    pub(crate) shape_kind: ChartSemanticShapeKind,
    pub(crate) role: ChartSemanticRole,
}

impl From<ResolvedSemanticDescriptor> for ChartSemanticDescriptor {
    fn from(value: ResolvedSemanticDescriptor) -> Self {
        Self {
            value_kind: value.value_kind.into(),
            shape_kind: value.shape_kind.into(),
            role: value.role.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "DTO-facing capability flags intentionally stay explicit and independently consumable"
)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartSemanticCapabilities {
    pub(crate) numeric_coordinate: bool,
    pub(crate) filterable: bool,
    pub(crate) groupable: bool,
    pub(crate) trace_source: bool,
    pub(crate) complex_projectable: bool,
    pub(crate) plottable_value: bool,
}

impl From<ResolvedSemanticCapabilities> for ChartSemanticCapabilities {
    fn from(value: ResolvedSemanticCapabilities) -> Self {
        Self {
            numeric_coordinate: value.numeric_coordinate,
            filterable: value.filterable,
            groupable: value.groupable,
            trace_source: value.trace_source,
            complex_projectable: value.complex_projectable,
            plottable_value: value.plottable_value,
        }
    }
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartSemanticColumn {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) label: Option<String>,
    pub(crate) semantic: ChartSemanticDescriptor,
    pub(crate) capabilities: ChartSemanticCapabilities,
    pub(crate) hidden_by_default: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartSemanticAxis {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) label: Option<String>,
    pub(crate) kind: ChartSemanticAxisKind,
    pub(crate) semantic: ChartSemanticDescriptor,
    pub(crate) capabilities: ChartSemanticCapabilities,
    pub(crate) is_inferred_axis: bool,
    pub(crate) physical_column: Option<String>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartSemantics {
    pub(crate) duplicate_policy: ChartDuplicatePolicy,
    pub(crate) index_realization: ChartIndexRealization,
    pub(crate) axes: Vec<ChartSemanticAxis>,
    pub(crate) value_columns: Vec<ChartSemanticColumn>,
    pub(crate) chart_axis_candidates: Vec<ChartSemanticAxis>,
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
    pub(crate) chart_semantics: Option<ChartSemantics>,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetWriteStatus {
    pub(crate) row_count: usize,
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

use chrono::{DateTime, Utc};
use fricon::{DatasetRecord, DatasetSortBy, DatasetStatus, SortDirection};
use serde::{Deserialize, Serialize};

use crate::application::{dataset_browser as app, workspace};

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
}

impl DatasetInfo {
    pub(crate) fn new(
        id: i32,
        name: String,
        description: String,
        favorite: bool,
        tags: Vec<String>,
        status: UiDatasetStatus,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            favorite,
            tags,
            status,
            created_at,
        }
    }
}

impl From<DatasetRecord> for DatasetInfo {
    fn from(record: DatasetRecord) -> Self {
        Self::new(
            record.id,
            record.metadata.name,
            record.metadata.description,
            record.metadata.favorite,
            record.metadata.tags,
            record.metadata.status.into(),
            record.metadata.created_at,
        )
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceInfo {
    pub(crate) path: String,
}

impl From<workspace::WorkspaceInfo> for WorkspaceInfo {
    fn from(value: workspace::WorkspaceInfo) -> Self {
        Self {
            path: value.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnInfo {
    pub(crate) name: String,
    pub(crate) is_complex: bool,
    pub(crate) is_trace: bool,
    pub(crate) is_index: bool,
}

impl From<app::ColumnInfo> for ColumnInfo {
    fn from(value: app::ColumnInfo) -> Self {
        Self {
            name: value.name,
            is_complex: value.is_complex,
            is_trace: value.is_trace,
            is_index: value.is_index,
        }
    }
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetDetail {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) favorite: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) status: UiDatasetStatus,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) columns: Vec<ColumnInfo>,
}

impl From<app::DatasetDetail> for DatasetDetail {
    fn from(value: app::DatasetDetail) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            favorite: value.favorite,
            tags: value.tags,
            status: value.status.into(),
            created_at: value.created_at,
            columns: value.columns.into_iter().map(Into::into).collect(),
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

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetWriteStatus {
    pub(crate) row_count: usize,
    pub(crate) is_complete: bool,
}

impl From<app::DatasetWriteStatus> for DatasetWriteStatus {
    fn from(value: app::DatasetWriteStatus) -> Self {
        Self {
            row_count: value.row_count,
            is_complete: value.is_complete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetCreated(pub(crate) DatasetInfo);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetUpdated(pub(crate) DatasetInfo);

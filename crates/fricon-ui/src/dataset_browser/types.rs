use chrono::{DateTime, Utc};
use fricon::{DatasetSortBy, DatasetStatus, SortDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, specta::Type, thiserror::Error)]
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, specta::Type)]
pub enum UiDatasetStatus {
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
    pub id: i32,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub status: UiDatasetStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetCreated(pub DatasetInfo);

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub(crate) struct DatasetUpdated(pub DatasetInfo);

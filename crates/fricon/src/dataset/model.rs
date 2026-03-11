use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: DatasetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatasetStatus {
    Writing,
    Completed,
    Aborted,
}

#[derive(Debug, Clone, Default)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DatasetSortBy {
    Id,
    Name,
    CreatedAt,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct DatasetListQuery {
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub favorite_only: bool,
    pub statuses: Option<Vec<DatasetStatus>>,
    pub sort_by: DatasetSortBy,
    pub sort_direction: SortDirection,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for DatasetListQuery {
    fn default() -> Self {
        Self {
            search: None,
            tags: None,
            favorite_only: false,
            statuses: None,
            sort_by: DatasetSortBy::Id,
            sort_direction: SortDirection::Desc,
            limit: None,
            offset: None,
        }
    }
}

#[derive(Debug, Clone, Copy, From)]
pub enum DatasetId {
    Id(i32),
    Uid(Uuid),
}

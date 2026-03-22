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
    pub trashed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
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
    pub trashed: Option<bool>,
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
            trashed: Some(false),
            sort_by: DatasetSortBy::Id,
            sort_direction: SortDirection::Desc,
            limit: None,
            offset: None,
        }
    }
}

impl DatasetListQuery {
    pub(crate) fn include_trashed(mut self) -> Self {
        self.trashed = None;
        self
    }

    pub(crate) fn unbounded(mut self) -> Self {
        self.limit = Some(i64::MAX);
        self.offset = Some(0);
        self
    }
}

#[derive(Debug, Clone, Copy, From)]
pub enum DatasetId {
    Id(i32),
    Uid(Uuid),
}

#[cfg(test)]
mod tests {
    use super::DatasetListQuery;

    #[test]
    fn unbounded_include_trashed_query_overrides_default_visibility_and_paging() {
        let query = DatasetListQuery::default().include_trashed().unbounded();

        assert_eq!(query.trashed, None);
        assert_eq!(query.limit, Some(i64::MAX));
        assert_eq!(query.offset, Some(0));
    }
}

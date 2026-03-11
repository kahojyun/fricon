use chrono::{DateTime, Utc};
use derive_more::From;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;
use uuid::Uuid;

use crate::{
    database::{self, DatabaseError, DatasetStatus},
    dataset_schema::DatasetError,
    runtime::app::AppError,
    storage::DatasetFsError,
};

#[derive(Debug, thiserror::Error)]
pub enum DatasetCatalogError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("No dataset file found.")]
    EmptyDataset,
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DieselError> for DatasetCatalogError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound {
                id: "unknown".to_string(),
            },
            other => Self::Database(other.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: DatasetMetadata,
}

impl DatasetRecord {
    #[must_use]
    pub fn from_database_models(dataset: database::Dataset, tags: Vec<database::Tag>) -> Self {
        let metadata = DatasetMetadata {
            uid: dataset.uid.0,
            name: dataset.name,
            description: dataset.description,
            favorite: dataset.favorite,
            status: dataset.status,
            created_at: dataset.created_at.and_utc(),
            tags: tags.into_iter().map(|tag| tag.name).collect(),
        };

        Self {
            id: dataset.id,
            metadata,
        }
    }
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

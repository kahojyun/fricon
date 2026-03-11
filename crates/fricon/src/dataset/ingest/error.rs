use diesel::result::Error as DieselError;
use tokio::task::JoinError;

use crate::dataset::{schema::DatasetError, sqlite::DatabaseError, storage::error::DatasetFsError};

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
}

impl From<DieselError> for IngestError {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound {
                id: "unknown".to_string(),
            },
            other => Self::Database(other.into()),
        }
    }
}

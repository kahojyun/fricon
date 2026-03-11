use tokio::task::JoinError;

use crate::dataset::{schema::DatasetError, storage::error::DatasetFsError};

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

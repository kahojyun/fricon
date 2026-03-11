use tokio::task::JoinError;

use crate::dataset::{schema::DatasetError, storage::error::DatasetFsError};

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("No dataset file found.")]
    EmptyDataset,
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

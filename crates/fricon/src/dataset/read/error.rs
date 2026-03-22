use crate::dataset::{schema::DatasetError, storage::error::DatasetFsError};

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Dataset payload has been permanently deleted: {id}")]
    Deleted { id: String },
    #[error("No dataset file found.")]
    EmptyDataset,
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

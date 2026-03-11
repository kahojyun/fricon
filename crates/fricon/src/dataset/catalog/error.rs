use tokio::task::JoinError;

use crate::dataset::storage::error::DatasetFsError;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

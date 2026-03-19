use crate::dataset::storage::error::DatasetFsError;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

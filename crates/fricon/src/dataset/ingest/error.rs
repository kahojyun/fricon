use crate::{
    database::core::DatabaseError,
    dataset::{schema::DatasetError, semantics::ManifestError, storage::error::DatasetFsError},
};

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

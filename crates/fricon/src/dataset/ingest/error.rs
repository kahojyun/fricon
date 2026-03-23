use crate::{
    database::core::DatabaseError,
    dataset::{schema::DatasetError, storage::error::DatasetFsError},
};

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("App state has been dropped")]
    StateDropped,
    #[error("Background task panicked")]
    TaskPanic,
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
}

use crate::{
    database::core::DatabaseError,
    dataset::{schema::DatasetError, storage::error::DatasetFsError},
};

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Dataset payload has been permanently deleted: {id}")]
    Deleted { id: String },
    #[error("No dataset file found.")]
    EmptyDataset,
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

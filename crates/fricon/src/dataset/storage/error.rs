use std::{io, path::PathBuf};

use arrow_schema::ArrowError;

use crate::dataset::schema::DatasetError;

#[derive(Debug, thiserror::Error)]
pub enum DatasetFsError {
    #[error("Dataset directory already exists: {0}")]
    AlreadyExist(PathBuf),
    #[error("Dataset chunk not found.")]
    ChunkNotFound,
    #[error("Invalid arrow IPC file.")]
    InvalidIpcFile,
    #[error("Schema mismatch.")]
    SchemaMismatch,
    #[error(transparent)]
    Dataset(#[from] DatasetError),
    #[error(transparent)]
    Arrow(#[from] ArrowError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

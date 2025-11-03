mod reader;
mod writer;

use std::{
    io,
    path::{Path, PathBuf},
};

use arrow_schema::ArrowError;

pub use self::{reader::ChunkReader, writer::ChunkWriter};
use crate::dataset;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dataset chunk not found.")]
    ChunkNotFound,
    #[error("Invalid arrow IPC file.")]
    InvalidIpcFile,
    #[error("Schema mismatch.")]
    SchemaMismatch,
    #[error(transparent)]
    Dataset(#[from] dataset::Error),
    #[error(transparent)]
    Arrow(#[from] ArrowError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Generate a chunk filename for the given chunk index
pub fn chunk_filename(chunk_index: usize) -> String {
    format!("data_chunk_{chunk_index}.arrow")
}

/// Get the chunk path by joining the base path with the chunk filename
pub fn chunk_path(dir_path: &Path, chunk_index: usize) -> PathBuf {
    dir_path.join(chunk_filename(chunk_index))
}

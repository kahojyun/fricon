mod reader;
mod writer;

use std::{
    fs, io,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use arrow_schema::ArrowError;
use tracing::warn;

pub use self::{reader::ChunkReader, writer::ChunkWriter};
use crate::dataset;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dataset directory already exists: {0}")]
    AlreadyExist(PathBuf),
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

pub fn delete_dataset(dir_path: &Path) -> Result<(), Error> {
    match fs::remove_dir_all(dir_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn create_dataset(dataset_path: &Path) -> Result<(), Error> {
    if dataset_path.exists() {
        warn!("Dataset path already exists: {}", dataset_path.display());
        return Err(Error::AlreadyExist(dataset_path.to_owned()));
    }
    fs::create_dir_all(dataset_path)?;
    Ok(())
}

pub(crate) mod error;
pub(crate) mod layout;
mod reader;
mod writer;

use std::{fs, io::ErrorKind, path::Path};

use tracing::warn;

pub(crate) use self::{error::DatasetFsError, reader::ChunkReader, writer::ChunkWriter};

pub(crate) fn delete_dataset(dir_path: &Path) -> Result<(), DatasetFsError> {
    match fs::remove_dir_all(dir_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(DatasetFsError::Io(e)),
    }
}

pub(crate) fn create_dataset(dataset_path: &Path) -> Result<(), DatasetFsError> {
    if dataset_path.exists() {
        warn!("Dataset path already exists: {}", dataset_path.display());
        return Err(DatasetFsError::AlreadyExist(dataset_path.to_owned()));
    }
    fs::create_dir_all(dataset_path)?;
    Ok(())
}

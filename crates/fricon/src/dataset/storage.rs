//! Low-level filesystem operations for dataset directories.
//!
//! # Ownership
//!
//! This module owns the three primitive filesystem operations used by the
//! catalog service: create, move, and delete. Higher-level workflows
//! (graveyard staging, import promotion) are composed in the service layer.
//!
//! # Invariants
//!
//! - [`delete_dataset`] is idempotent: deleting an already-absent directory
//!   succeeds silently.
//! - [`move_dataset`] fails if the destination already exists (no silent
//!   overwrite).
//! - [`create_dataset`] fails if the directory already exists.

pub(crate) mod error;
pub(crate) mod layout;
mod reader;
mod writer;

use std::{fs, io::ErrorKind, path::Path};

use tracing::warn;

pub(crate) use self::{error::DatasetFsError, reader::ChunkReader, writer::ChunkWriter};

/// Remove a dataset directory tree. Idempotent: returns `Ok(())` when the
/// directory does not exist.
pub(crate) fn delete_dataset(dir_path: &Path) -> Result<(), DatasetFsError> {
    match fs::remove_dir_all(dir_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(DatasetFsError::Io(e)),
    }
}

/// Rename a dataset directory from `from_path` to `to_path`.
///
/// Creates parent directories of `to_path` if needed. Fails if `to_path`
/// already exists to prevent silent data loss.
pub(crate) fn move_dataset(from_path: &Path, to_path: &Path) -> Result<(), DatasetFsError> {
    if to_path.exists() {
        return Err(DatasetFsError::AlreadyExist(to_path.to_owned()));
    }
    if let Some(parent) = to_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(from_path, to_path)?;
    Ok(())
}

/// Create a new dataset directory. Fails if the path already exists.
pub(crate) fn create_dataset(dataset_path: &Path) -> Result<(), DatasetFsError> {
    if dataset_path.exists() {
        warn!("Dataset path already exists: {}", dataset_path.display());
        return Err(DatasetFsError::AlreadyExist(dataset_path.to_owned()));
    }
    fs::create_dir_all(dataset_path)?;
    Ok(())
}

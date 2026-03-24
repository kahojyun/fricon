//! Catalog error types.
//!
//! [`CatalogError`] covers business failures (not found, already deleted)
//! and maps adapter-level errors (filesystem, database, portability) at this
//! boundary.

use crate::{
    database::core::DatabaseError,
    dataset::{portability::PortabilityError, storage::error::DatasetFsError},
};

/// Error type for dataset catalog operations.
///
/// Business variants carry a human-readable `id` string (numeric id or uuid).
/// Adapter failures are wrapped in `DatasetFs`, `Database`, or `Portability`.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Dataset has been permanently deleted: {id}")]
    Deleted { id: String },
    #[error("Tag name must not be empty")]
    EmptyTag,
    #[error("Dataset must be moved to trash before permanent deletion")]
    NotTrashed,
    #[error("Old tag name and new tag name must differ")]
    SameTagName,
    #[error("Source tag and target tag must differ")]
    SameSourceTarget,
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Portability(#[from] PortabilityError),
}

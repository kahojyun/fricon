//! Catalog error types.
//!
//! [`CatalogError`] covers business failures (not found, already deleted)
//! and maps adapter-level errors (filesystem, unexpected) at this boundary.

use crate::dataset::storage::error::DatasetFsError;

/// Error type for dataset catalog operations.
///
/// Business variants (`NotFound`, `Deleted`) carry a human-readable `id`
/// string (numeric id or uuid). Adapter failures are wrapped in
/// `DatasetFs` or `Unexpected`.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("Dataset has been permanently deleted: {id}")]
    Deleted { id: String },
    #[error(transparent)]
    DatasetFs(#[from] DatasetFsError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

//! Error types for workspace operations.

use std::io;

use tempfile::PersistError;

/// Errors that occur when acquiring the workspace file lock.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Failed to open lock file")]
    Open(#[source] io::Error),
    #[error("Failed to acquire file lock")]
    Acquire(#[source] io::Error),
}

/// Errors that occur during workspace creation, opening, or validation.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Workspace already exists")]
    AlreadyExists,
    #[error("Not a Fricon workspace: missing metadata file")]
    NotWorkspace,
    #[error(
        "Workspace version {version} requires migration before use. Open the workspace with a \
         newer fricon build that supports migrating it."
    )]
    MigrationRequired { version: u32 },
    #[error(
        "Workspace version {version} is too old to migrate automatically. Supported migrations \
         start at version {supported_from}."
    )]
    UnsupportedMigrationVersion { version: u32, supported_from: u32 },
    #[error(
        "Workspace version {version} is newer than the supported version. Please update fricon."
    )]
    VersionTooNew { version: u32 },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    FilePersist(#[from] PersistError),
    #[error(transparent)]
    Lock(#[from] LockError),
}

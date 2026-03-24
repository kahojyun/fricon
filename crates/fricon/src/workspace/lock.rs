//! Workspace lock file lifecycle.
//!
//! This module owns the file-backed exclusivity guard used when opening a
//! workspace. Holding [`FileLock`] keeps the lock file open; dropping it best
//! effort removes the lock file path.

use std::{
    fs::{self, File},
    path::PathBuf,
};

use tracing::warn;

use super::error::LockError;

/// Exclusive file-backed lock for a workspace root.
///
/// Creating the lock opens/truncates the lock file and acquires an exclusive
/// OS-level file lock. Dropping the guard releases the lock by closing the
/// file and then attempts to remove the lock file path.
#[derive(Debug)]
pub(crate) struct FileLock {
    _file: File,
    path: PathBuf,
}

impl FileLock {
    /// Acquire an exclusive lock for the given file path.
    ///
    /// The returned guard must stay alive for as long as exclusive workspace
    /// access is required.
    pub(crate) fn new(path: impl Into<PathBuf>) -> Result<Self, LockError> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(LockError::Open)?;
        file.try_lock().map_err(|e| LockError::Acquire(e.into()))?;
        Ok(Self { _file: file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            warn!("Failed to remove locked file: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_and_removes_lock_file() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("test.lock");
        {
            let _lock = FileLock::new(&lock_path)
                .expect("Lock file should be creatable in test environment");
            assert!(lock_path.exists());
        }
        assert!(!lock_path.exists());
    }

    #[test]
    fn cannot_acquire_lock_twice() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("double.lock");
        let _first_lock = FileLock::new(&lock_path)
            .expect("First lock should be acquired successfully in test environment");
        let second_lock = FileLock::new(&lock_path);
        assert!(
            second_lock.is_err(),
            "Should not acquire lock twice on same file"
        );
    }
}

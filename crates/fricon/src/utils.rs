use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};

use tracing::warn;

/// Generate a chunk filename for the given chunk index
pub fn chunk_filename(chunk_index: usize) -> String {
    format!("data_chunk_{chunk_index}.arrow")
}

/// Get the chunk path by joining the base path with the chunk filename
pub fn chunk_path(base_path: &Path, chunk_index: usize) -> PathBuf {
    base_path.join(chunk_filename(chunk_index))
}

#[derive(Debug)]
pub struct FileLock {
    _file: File,
    path: PathBuf,
}

impl FileLock {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open file for locking.")?;
        file.try_lock().context("Failed to acquire file lock.")?;
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
            let _lock = FileLock::new(&lock_path).expect("Should create lock");
            assert!(lock_path.exists());
        }
        // File should be removed after drop
        assert!(!lock_path.exists());
    }

    #[test]
    fn cannot_acquire_lock_twice() {
        let dir = tempdir().unwrap();
        let lock_path = dir.path().join("double.lock");
        let _first_lock = FileLock::new(&lock_path).expect("Should acquire first lock");
        // Attempting to acquire the same lock again should fail
        let second_lock = FileLock::new(&lock_path);
        assert!(
            second_lock.is_err(),
            "Should not acquire lock twice on same file"
        );
    }
}

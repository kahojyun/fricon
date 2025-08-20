use std::path::{self, Path, PathBuf};

use anyhow::Result;
use uuid::Uuid;

/// Path to the workspace root directory.
///
/// Provide methods to construct paths to various components of the workspace.
#[derive(Debug, Clone)]
pub struct WorkspacePath(PathBuf);

impl WorkspacePath {
    pub fn new(path: &Path) -> Result<Self> {
        let path = path::absolute(path)?;
        Ok(Self(path))
    }

    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        self.0.join("data")
    }

    #[must_use]
    pub fn log_dir(&self) -> PathBuf {
        self.0.join("log")
    }

    #[must_use]
    pub fn backup_dir(&self) -> PathBuf {
        self.0.join("backup")
    }

    #[must_use]
    pub fn ipc_file(&self) -> PathBuf {
        self.0.join("fricon.socket")
    }

    #[must_use]
    pub fn database_file(&self) -> PathBuf {
        self.0.join("fricon.sqlite3")
    }

    #[must_use]
    pub fn metadata_file(&self) -> PathBuf {
        self.0.join(".fricon_workspace.json")
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.0.join(".fricon.lock")
    }
}

#[must_use]
pub fn dataset_path_from_uuid(uuid: Uuid) -> String {
    let uuid = uuid.to_string();
    let prefix = &uuid[..2];
    format!("{prefix}/{uuid}")
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_format_dataset_path() {
        let uuid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = dataset_path_from_uuid(uuid);
        assert_eq!(path, "6e/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
    }
}

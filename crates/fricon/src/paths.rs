use std::path::{self, Path, PathBuf};

use anyhow::Result;
use chrono::NaiveDate;
use uuid::Uuid;

/// Path to the workspace root directory.
///
/// Provide methods to construct paths to various components of the workspace.
#[derive(Debug, Clone)]
pub struct WorkspacePath(PathBuf);

impl WorkspacePath {
    /// # Errors
    ///
    /// If the path cannot be converted to an absolute path.
    pub fn new(path: &Path) -> Result<Self> {
        let path = path::absolute(path)?;
        Ok(Self(path))
    }

    #[must_use]
    pub fn data_dir(&self) -> DataDirectory {
        DataDirectory(self.0.join("data"))
    }

    #[must_use]
    pub fn log_dir(&self) -> LogDirectory {
        LogDirectory(self.0.join("log"))
    }

    #[must_use]
    pub fn backup_dir(&self) -> BackupDirectory {
        BackupDirectory(self.0.join("backup"))
    }

    #[must_use]
    pub fn ipc_file(&self) -> IpcFile {
        IpcFile(self.0.join("fricon.socket"))
    }

    #[must_use]
    pub fn database_file(&self) -> DatabaseFile {
        DatabaseFile(self.0.join("fricon.sqlite3"))
    }

    #[must_use]
    pub fn version_file(&self) -> VersionFile {
        VersionFile(self.0.join(".fricon_version"))
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.0.join(".fricon.lock")
    }
}

#[derive(Debug, Clone)]
pub struct DataDirectory(pub PathBuf);

#[derive(Debug, Clone)]
pub struct LogDirectory(pub PathBuf);

#[derive(Debug, Clone)]
pub struct BackupDirectory(pub PathBuf);

#[derive(Debug, Clone)]
pub struct IpcFile(pub PathBuf);

#[derive(Debug, Clone)]
pub struct DatabaseFile(pub PathBuf);

#[derive(Debug, Clone)]
pub struct VersionFile(pub PathBuf);

impl DataDirectory {
    #[must_use]
    pub fn join(&self, path: &DatasetPath) -> PathBuf {
        self.0.join(&path.0)
    }
}

/// Path to dataset relative to data storage root in the workspace.
///
/// If the workspace root is `/workspace`, the data storage root is `/workspace/data`,
/// then the absolute path to the dataset is `/workspace/data/<DatasetPath>`.
#[derive(Debug, Clone)]
pub struct DatasetPath(pub String);

impl DatasetPath {
    /// Create a new dataset path based on the date and UUID.
    #[must_use]
    pub fn new(date: NaiveDate, uid: Uuid) -> Self {
        Self(format!("{date}/{uid}"))
    }
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_format_dataset_path() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let uid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = DatasetPath::new(date, uid);
        assert_eq!(path.0, "2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
    }
}

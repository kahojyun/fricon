//! Manage storage of fricon data.
use std::path::{self, Path, PathBuf};

use anyhow::Result;
use chrono::NaiveDate;
use uuid::Uuid;

/// Path to the workspace root directory.
///
/// Provide methods to construct paths to various components of the workspace.
#[derive(Debug, Clone)]
pub struct WorkDirectory(PathBuf);

impl WorkDirectory {
    pub fn new(path: &Path) -> Result<Self> {
        let path = path::absolute(path)?;
        Ok(Self(path))
    }
    pub fn data_dir(&self) -> DataDirectory {
        DataDirectory(self.0.join("data"))
    }

    pub fn log_dir(&self) -> LogDirectory {
        LogDirectory(self.0.join("log"))
    }

    pub fn backup_dir(&self) -> BackupDirectory {
        BackupDirectory(self.0.join("backup"))
    }

    pub fn ipc_file(&self) -> IpcFile {
        IpcFile(self.0.join("fricon.socket"))
    }

    pub fn config_file(&self) -> ConfigFile {
        ConfigFile(self.0.join("config.toml"))
    }

    pub fn database_file(&self) -> DatabaseFile {
        DatabaseFile(self.0.join("fricon.sqlite3"))
    }

    pub fn version_file(&self) -> VersionFile {
        VersionFile(self.0.join(".fricon_version"))
    }
}

pub struct DataDirectory(pub PathBuf);
pub struct LogDirectory(pub PathBuf);
pub struct BackupDirectory(pub PathBuf);
#[derive(Debug, Clone)]
pub struct IpcFile(pub PathBuf);
pub struct ConfigFile(pub PathBuf);
pub struct DatabaseFile(pub PathBuf);
pub struct VersionFile(pub PathBuf);

impl DataDirectory {
    pub fn join(&self, path: &DatasetPath) -> PathBuf {
        self.0.join(&path.0)
    }
}

/// Path to dataset relative to data storage root in the workspace.
///
/// If the workspace root is `/workspace`, the data storage root is `/workspace/data`,
/// then the absolute path to the dataset is `/workspace/data/<DatasetPath>`.
pub struct DatasetPath(String);

impl DatasetPath {
    /// Create a new dataset path based on the date and UUID.
    pub fn new(date: NaiveDate, uid: Uuid) -> Self {
        Self(format!("{date}/{uid}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
        assert_eq!(
            path.as_str(),
            "2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0"
        );
    }
}

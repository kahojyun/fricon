use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDateTime;
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::FileLock;

const WORKSPACE_VERSION: Version = Version::new(0, 1, 0);

#[derive(Debug, PartialEq)]
pub enum VersionCheckResult {
    Current,
    NeedsMigration,
}

pub fn get_log_dir(workspace_path: impl Into<PathBuf>) -> Result<PathBuf> {
    Ok(WorkspaceRoot::validate(workspace_path)?.log_dir())
}

fn check_version(version: &Version) -> Result<VersionCheckResult> {
    use std::cmp::Ordering;

    match version.cmp(&WORKSPACE_VERSION) {
        Ordering::Equal => Ok(VersionCheckResult::Current),
        Ordering::Less => Ok(VersionCheckResult::NeedsMigration),
        Ordering::Greater => {
            bail!(
                "Workspace version {} is newer than supported version {}. Please update fricon.",
                version,
                WORKSPACE_VERSION
            );
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub version: Version,
}

impl WorkspaceMetadata {
    pub fn write_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path)
            .with_context(|| format!("Failed to write workspace metadata to {}", path.display()))?;
        serde_json::to_writer_pretty(file, self)
            .with_context(|| format!("Failed to write workspace metadata to {}", path.display()))?;
        Ok(())
    }

    pub fn read_json(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| {
            format!("Failed to read workspace metadata from {}", path.display())
        })?;
        let metadata = serde_json::from_reader(file).with_context(|| {
            format!("Failed to read workspace metadata from {}", path.display())
        })?;
        Ok(metadata)
    }
}

/// Manages the paths within a Fricon workspace.
///
/// This struct encapsulates the logic for constructing various sub-paths
/// relative to the workspace root
#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    root: PathBuf,
}

impl WorkspacePaths {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    #[must_use]
    pub fn log_dir(&self) -> PathBuf {
        self.root.join("log")
    }

    #[must_use]
    pub fn backup_dir(&self) -> PathBuf {
        self.root.join("backup")
    }

    #[must_use]
    pub fn ipc_file(&self) -> PathBuf {
        self.root.join("fricon.socket")
    }

    #[must_use]
    pub fn database_file(&self) -> PathBuf {
        self.root.join("fricon.sqlite3")
    }

    #[must_use]
    pub fn database_backup_file(&self, time: NaiveDateTime) -> PathBuf {
        let mut out = self.backup_dir();
        out.push(format!(
            "fricon_backup-{}.sqlite3",
            time.format("%Y%m%d_%H%M%S")
        ));
        out
    }

    #[must_use]
    pub fn metadata_file(&self) -> PathBuf {
        self.root.join(".fricon_workspace.json")
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.root.join(".fricon.lock")
    }

    #[must_use]
    pub fn dataset_path_from_uuid(&self, uuid: Uuid) -> PathBuf {
        let mut data_dir = self.data_dir();
        data_dir.push(dataset_path_from_uuid(uuid));
        data_dir
    }
}

fn init_workspace_dirs(paths: &WorkspacePaths) -> Result<()> {
    fs::create_dir(paths.data_dir())?;
    fs::create_dir(paths.log_dir())?;
    fs::create_dir(paths.backup_dir())?;
    Ok(())
}

#[must_use]
fn dataset_path_from_uuid(uuid: Uuid) -> String {
    let uuid = uuid.to_string();
    let prefix = &uuid[..2];
    format!("{prefix}/{uuid}")
}

/// An opened and validated workspace root directory with exclusive access lock.
///
/// This type ensures that the workspace is properly initialized and validated,
/// and holds an exclusive file lock to prevent concurrent access.
#[derive(Debug)]
pub struct WorkspaceRoot {
    paths: WorkspacePaths,
    _lock: FileLock,
}

impl WorkspaceRoot {
    /// Initialize a new workspace at the given path.
    ///
    /// The directory must be empty or non-existent.
    pub fn init(path: impl Into<PathBuf>) -> Result<Self> {
        let paths = WorkspacePaths::new(path);
        let root = paths.root();

        fs::create_dir_all(root).context("Failed to create directory.")?;
        let lock_file_path = paths.lock_file();
        let lock = FileLock::new(&lock_file_path)?;

        let dir_contents = root
            .read_dir()
            .context("Failed to read directory contents.")?;
        for entry_result in dir_contents {
            let entry = entry_result.context("Failed to get directory entry.")?;
            if entry.path() != lock_file_path {
                bail!("Directory is not empty.");
            }
        }

        init_workspace_dirs(&paths).context("Failed to initialize workspace directories.")?;

        let metadata = WorkspaceMetadata {
            version: WORKSPACE_VERSION,
        };
        metadata.write_json(paths.metadata_file())?;

        Ok(Self { paths, _lock: lock })
    }

    /// Open an existing workspace at the given path.
    ///
    /// Validates the workspace metadata and acquires an exclusive lock.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let paths = WorkspacePaths::new(path);
        let lock = FileLock::new(paths.lock_file())?;
        let metadata = WorkspaceMetadata::read_json(paths.metadata_file())?;
        let mut root = Self { paths, _lock: lock };

        match check_version(&metadata.version)? {
            VersionCheckResult::Current => {}
            VersionCheckResult::NeedsMigration => {
                tracing::info!("Workspace requires migration");
                root.migrate_to_current(&metadata.version)?;
            }
        }

        Ok(root)
    }

    /// Validate that a directory is a valid workspace without opening it.
    ///
    /// This checks for the presence of required files and validates metadata
    /// without acquiring a lock.
    pub fn validate(path: impl Into<PathBuf>) -> Result<WorkspacePaths> {
        let paths = WorkspacePaths::new(path);

        if !paths.metadata_file().exists() {
            bail!("Not a Fricon workspace: missing metadata file");
        }

        let metadata = WorkspaceMetadata::read_json(paths.metadata_file())?;
        match check_version(&metadata.version)? {
            VersionCheckResult::Current | VersionCheckResult::NeedsMigration => {}
        }

        Ok(paths)
    }

    #[must_use]
    pub fn paths(&self) -> &WorkspacePaths {
        &self.paths
    }

    fn migrate_to_current(&mut self, version: &Version) -> Result<()> {
        if version < &WORKSPACE_VERSION {
            tracing::info!(
                "Migrating workspace from version {} to {}",
                version,
                WORKSPACE_VERSION
            );
            let mut metadata = WorkspaceMetadata::read_json(self.paths.metadata_file())?;
            metadata.version = WORKSPACE_VERSION;
            metadata.write_json(self.paths.metadata_file())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tempfile::tempdir;
    use uuid::uuid;

    use super::*;

    #[test]
    fn database_backup_file_path() {
        let paths = WorkspacePaths::new("./");
        let time = NaiveDate::from_ymd_opt(2016, 7, 8)
            .unwrap()
            .and_hms_opt(9, 10, 11)
            .unwrap();
        let expected_path = paths
            .backup_dir()
            .join("fricon_backup-20160708_091011.sqlite3");
        let actual_path = paths.database_backup_file(time);

        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn format_dataset_path() {
        let uuid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = dataset_path_from_uuid(uuid);
        assert_eq!(path, "6e/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
    }

    #[test]
    fn workspace_root_init() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        let root = WorkspaceRoot::init(workspace_path).unwrap();
        let paths = root.paths();

        assert!(paths.data_dir().exists());
        assert!(paths.log_dir().exists());
        assert!(paths.backup_dir().exists());
        assert!(paths.metadata_file().exists());
        assert!(paths.lock_file().exists());
    }

    #[test]
    fn workspace_root_open() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        let root1 = WorkspaceRoot::init(workspace_path).unwrap();
        drop(root1);

        let root2 = WorkspaceRoot::open(workspace_path).unwrap();
        assert_eq!(root2.paths().root(), workspace_path);
    }

    #[test]
    fn workspace_root_validate() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        assert!(WorkspaceRoot::validate(workspace_path).is_err());

        let root = WorkspaceRoot::init(workspace_path).unwrap();
        drop(root);

        assert!(WorkspaceRoot::validate(workspace_path).is_ok());
    }

    #[test]
    fn workspace_root_exclusive_lock() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        let _root1 = WorkspaceRoot::init(workspace_path).unwrap();

        assert!(WorkspaceRoot::open(workspace_path).is_err());
    }
}

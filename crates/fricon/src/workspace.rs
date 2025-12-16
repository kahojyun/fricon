use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDateTime;
use semver::Version;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
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
                "Workspace version {version} is newer than supported version {WORKSPACE_VERSION}. \
                 Please update fricon."
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
        let mut file = NamedTempFile::new_in(path.parent().expect("Should be workspace root."))?;
        serde_json::to_writer_pretty(&mut file, self)
            .with_context(|| format!("Failed to write workspace metadata to {}", path.display()))?;
        file.persist(path)?;
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
    pub fn dataset_path_from_uid(&self, uid: Uuid) -> PathBuf {
        let mut data_dir = self.data_dir();
        data_dir.push(dataset_path_from_uid(uid));
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
fn dataset_path_from_uid(uid: Uuid) -> String {
    let uid = uid.to_string();
    let prefix = &uid[..2];
    format!("{prefix}/{uid}")
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
    /// Create or open a workspace at the given path.
    ///
    /// If the workspace doesn't exist, it will be created.
    /// If it already exists, it will be opened.
    pub fn create(path: impl Into<PathBuf>) -> Result<Self> {
        let paths = WorkspacePaths::new(path);

        // Check if workspace already exists by checking metadata file
        if paths.metadata_file().exists() {
            // Try to open existing workspace
            match Self::open_internal(paths.clone()) {
                Ok(root) => Ok(root),
                Err(_) => {
                    // If open fails, metadata might be corrupted, try to create new
                    Self::create_new_internal(paths)
                }
            }
        } else {
            // Create new workspace
            Self::create_new_internal(paths)
        }
    }

    /// Create a new workspace at the given path, failing if it already exists.
    ///
    /// This method will return an error if the directory already exists,
    /// unlike `create()` which will open an existing workspace.
    pub fn create_new(path: impl Into<PathBuf>) -> Result<Self> {
        let paths = WorkspacePaths::new(path);
        Self::create_new_internal(paths)
    }

    /// Open an existing workspace at the given path.
    ///
    /// Validates the workspace metadata and acquires an exclusive lock.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let paths = WorkspacePaths::new(path);
        Self::open_internal(paths)
    }

    fn create_new_internal(paths: WorkspacePaths) -> Result<Self> {
        let root = paths.root();

        // Check if directory exists and is not empty (excluding lock file)
        if root.exists() {
            let mut has_non_lock_files = false;
            if let Ok(entries) = root.read_dir() {
                for entry in entries.flatten() {
                    let path = entry.path();
                    // Skip lock file as it gets deleted when WorkspaceRoot is dropped
                    if path.file_name().and_then(|n| n.to_str()) != Some(".fricon.lock") {
                        has_non_lock_files = true;
                        break;
                    }
                }
            }

            if has_non_lock_files {
                bail!("Workspace already exists");
            }
        }

        fs::create_dir_all(root).context("Failed to create directory.")?;
        let lock_file_path = paths.lock_file();
        let lock = FileLock::new(&lock_file_path)?;

        init_workspace_dirs(&paths).context("Failed to initialize workspace directories.")?;

        let metadata = WorkspaceMetadata {
            version: WORKSPACE_VERSION,
        };
        metadata.write_json(paths.metadata_file())?;

        Ok(Self { paths, _lock: lock })
    }

    fn open_internal(paths: WorkspacePaths) -> Result<Self> {
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
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_workspace_create() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create workspace with create (should work - creates new)
        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        // Create again with create (should work - opens existing)
        let root2 = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root2);
    }

    #[test]
    fn test_workspace_create_new_strict() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create workspace with create_new (should work)
        let root = WorkspaceRoot::create_new(workspace_path.clone()).unwrap();
        drop(root);

        // Try to create again with create_new (should fail)
        let result = WorkspaceRoot::create_new(workspace_path.clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_open() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create workspace first
        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root); // Release lock

        // Open existing workspace
        let _root2 = WorkspaceRoot::open(workspace_path.clone()).unwrap();
        assert!(workspace_path.exists());
    }

    #[test]
    fn test_workspace_validate() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Validate non-existent workspace
        let result = WorkspaceRoot::validate(&workspace_path);
        assert!(result.is_err());

        // Create workspace
        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        // Validate existing workspace
        let paths = WorkspaceRoot::validate(&workspace_path).unwrap();
        assert_eq!(paths.root(), workspace_path);
    }

    #[test]
    fn test_workspace_locking() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create workspace and acquire lock
        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();

        // Try to open same workspace (should fail due to lock)
        let result = WorkspaceRoot::open(workspace_path.clone());
        assert!(result.is_err());

        drop(root); // Release lock

        // Now should work
        let _root1 = WorkspaceRoot::open(workspace_path.clone()).unwrap();
    }

    #[test]
    fn test_workspace_structure() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        // Create a new workspace
        let _root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        assert!(workspace_path.exists());

        // Verify workspace structure
        assert!(workspace_path.join(".fricon_workspace.json").exists());
        assert!(workspace_path.join("data").exists());
        assert!(workspace_path.join("log").exists());
    }
}

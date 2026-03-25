mod error;
mod lock;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tracing::{debug, info, instrument};
use uuid::Uuid;

pub use self::error::WorkspaceError;
use self::lock::FileLock;

const WORKSPACE_VERSION: u32 = 1;
const MIN_MIGRATABLE_WORKSPACE_VERSION: u32 = 0;

#[derive(Debug, PartialEq)]
pub(crate) enum VersionCheckResult {
    Current,
    NeedsMigration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceValidation {
    Current(WorkspacePaths),
    NeedsMigration { paths: WorkspacePaths, version: u32 },
}

impl WorkspaceValidation {
    #[must_use]
    pub fn paths(&self) -> &WorkspacePaths {
        match self {
            Self::Current(paths) | Self::NeedsMigration { paths, .. } => paths,
        }
    }

    #[must_use]
    pub fn into_paths(self) -> WorkspacePaths {
        match self {
            Self::Current(paths) | Self::NeedsMigration { paths, .. } => paths,
        }
    }

    pub fn require_current(self) -> Result<WorkspacePaths, WorkspaceError> {
        match self {
            Self::Current(paths) => Ok(paths),
            Self::NeedsMigration { version, .. } => {
                Err(WorkspaceError::MigrationRequired { version })
            }
        }
    }
}

pub fn get_log_dir(workspace_path: impl Into<PathBuf>) -> Result<PathBuf, WorkspaceError> {
    Ok(WorkspaceRoot::validate(workspace_path)?
        .into_paths()
        .log_dir())
}

fn check_version(version: u32) -> Result<VersionCheckResult, WorkspaceError> {
    match version.cmp(&WORKSPACE_VERSION) {
        std::cmp::Ordering::Equal => Ok(VersionCheckResult::Current),
        std::cmp::Ordering::Less => Ok(VersionCheckResult::NeedsMigration),
        std::cmp::Ordering::Greater => Err(WorkspaceError::VersionTooNew { version }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WorkspaceMetadata {
    pub version: u32,
}

impl WorkspaceMetadata {
    pub(crate) fn write_json(&self, path: impl AsRef<Path>) -> Result<(), WorkspaceError> {
        let path = path.as_ref();
        let mut file = NamedTempFile::new_in(path.parent().expect("Should be workspace root."))?;
        serde_json::to_writer_pretty(&mut file, self)?;
        file.persist(path)?;
        Ok(())
    }

    pub(crate) fn read_json(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let metadata = serde_json::from_reader(file)?;
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

impl PartialEq for WorkspacePaths {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
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
    pub fn graveyard_dir(&self) -> PathBuf {
        self.data_dir().join(".graveyard")
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

    #[must_use]
    pub fn graveyard_dataset_path_from_uid(&self, uid: Uuid) -> PathBuf {
        self.graveyard_dir().join(uid.to_string())
    }
}

fn init_workspace_dirs(paths: &WorkspacePaths) -> Result<(), WorkspaceError> {
    fs::create_dir(paths.data_dir())?;
    fs::create_dir(paths.graveyard_dir())?;
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
    pub fn create(path: impl Into<PathBuf>) -> Result<Self, WorkspaceError> {
        let paths = WorkspacePaths::new(path);

        if paths.metadata_file().exists() {
            Self::open_internal(paths)
        } else {
            Self::create_new_internal(paths)
        }
    }

    /// Create a new workspace at the given path, failing if it already exists.
    ///
    /// This method will return an error if the directory already exists,
    /// unlike `create()` which will open an existing workspace.
    pub fn create_new(path: impl Into<PathBuf>) -> Result<Self, WorkspaceError> {
        let paths = WorkspacePaths::new(path);
        Self::create_new_internal(paths)
    }

    /// Open an existing workspace at the given path.
    ///
    /// Validates the workspace metadata and acquires an exclusive lock.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, WorkspaceError> {
        let paths = WorkspacePaths::new(path);
        Self::open_internal(paths)
    }

    #[instrument(skip_all, fields(path = ?paths.root()))]
    fn create_new_internal(paths: WorkspacePaths) -> Result<Self, WorkspaceError> {
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
                return Err(WorkspaceError::AlreadyExists);
            }
        }

        fs::create_dir_all(root)?;
        let lock_file_path = paths.lock_file();
        let lock = FileLock::new(&lock_file_path)?;

        init_workspace_dirs(&paths)?;

        let metadata = WorkspaceMetadata {
            version: WORKSPACE_VERSION,
        };
        metadata.write_json(paths.metadata_file())?;

        info!(path = ?paths.root(), "Workspace created");
        Ok(Self { paths, _lock: lock })
    }

    #[instrument(skip_all, fields(path = ?paths.root()))]
    fn open_internal(paths: WorkspacePaths) -> Result<Self, WorkspaceError> {
        let lock = FileLock::new(paths.lock_file())?;
        let metadata = WorkspaceMetadata::read_json(paths.metadata_file())?;
        let mut root = Self { paths, _lock: lock };

        match check_version(metadata.version)? {
            VersionCheckResult::Current => {
                debug!(path = ?root.paths.root(), version = %metadata.version, "Workspace opened");
            }
            VersionCheckResult::NeedsMigration => {
                info!(path = ?root.paths.root(), from_version = %metadata.version, "Workspace requires migration");
                root.migrate_to_current(metadata.version)?;
            }
        }

        Ok(root)
    }

    /// Validate that a directory is a valid workspace without opening it.
    ///
    /// This checks for the presence of required files and validates metadata
    /// without acquiring a lock. The returned status distinguishes between a
    /// current workspace and one that still needs migration.
    pub fn validate(path: impl Into<PathBuf>) -> Result<WorkspaceValidation, WorkspaceError> {
        let paths = WorkspacePaths::new(path);

        if !paths.metadata_file().exists() {
            return Err(WorkspaceError::NotWorkspace);
        }

        let metadata = WorkspaceMetadata::read_json(paths.metadata_file())?;
        match check_version(metadata.version)? {
            VersionCheckResult::Current => Ok(WorkspaceValidation::Current(paths)),
            VersionCheckResult::NeedsMigration => Ok(WorkspaceValidation::NeedsMigration {
                paths,
                version: metadata.version,
            }),
        }
    }

    /// Validate that a directory is ready for current-version operations.
    pub fn validate_current(path: impl Into<PathBuf>) -> Result<WorkspacePaths, WorkspaceError> {
        Self::validate(path)?.require_current()
    }

    #[must_use]
    pub fn paths(&self) -> &WorkspacePaths {
        &self.paths
    }

    fn migrate_to_current(&mut self, mut version: u32) -> Result<(), WorkspaceError> {
        while version < WORKSPACE_VERSION {
            version = self.migrate_one_step(version)?;
        }

        Ok(())
    }

    fn migrate_one_step(&mut self, version: u32) -> Result<u32, WorkspaceError> {
        match version {
            0 => self.migrate_v0_to_v1(),
            _ => Err(WorkspaceError::UnsupportedMigrationVersion {
                version,
                supported_from: MIN_MIGRATABLE_WORKSPACE_VERSION,
            }),
        }
    }

    fn migrate_v0_to_v1(&mut self) -> Result<u32, WorkspaceError> {
        let from = 0;
        let to = 1;

        tracing::info!("Migrating workspace from version {} to {}", from, to);
        let mut metadata = WorkspaceMetadata::read_json(self.paths.metadata_file())?;
        metadata.version = to;
        metadata.write_json(self.paths.metadata_file())?;

        Ok(to)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn write_workspace_version(path: &Path, version: u32) {
        WorkspaceMetadata { version }
            .write_json(path.join(".fricon_workspace.json"))
            .unwrap();
    }

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
        let result = WorkspaceRoot::validate_current(&workspace_path);
        assert!(result.is_err());

        // Create workspace
        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        // Validate existing workspace
        let paths = WorkspaceRoot::validate_current(&workspace_path).unwrap();
        assert_eq!(paths.root(), workspace_path);
    }

    #[test]
    fn test_workspace_validate_reports_migration_required() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        write_workspace_version(&workspace_path, 0);

        let result = WorkspaceRoot::validate_current(&workspace_path)
            .expect_err("old workspace should require migration");
        assert!(matches!(
            result,
            WorkspaceError::MigrationRequired { version } if version == 0
        ));
    }

    #[test]
    fn test_workspace_validate_reports_old_workspace_status() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        write_workspace_version(&workspace_path, 0);

        let result = WorkspaceRoot::validate(&workspace_path).unwrap();
        assert!(matches!(
            result,
            WorkspaceValidation::NeedsMigration { version, .. } if version == 0
        ));
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

    #[test]
    fn test_workspace_open_migrates_supported_older_version() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        write_workspace_version(&workspace_path, 0);

        let _root = WorkspaceRoot::open(workspace_path.clone()).unwrap();
        let metadata =
            WorkspaceMetadata::read_json(workspace_path.join(".fricon_workspace.json")).unwrap();
        assert_eq!(metadata.version, WORKSPACE_VERSION);
    }

    #[test]
    fn test_workspace_open_rejects_newer_workspace_version() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        let root = WorkspaceRoot::create(workspace_path.clone()).unwrap();
        drop(root);

        write_workspace_version(&workspace_path, WORKSPACE_VERSION + 1);

        let result = WorkspaceRoot::open(workspace_path.clone())
            .expect_err("newer workspace version should be rejected");
        assert!(matches!(
            result,
            WorkspaceError::VersionTooNew { version } if version == WORKSPACE_VERSION + 1
        ));
    }
}

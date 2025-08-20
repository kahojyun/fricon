use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use semver::Version;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

const WORKSPACE_VERSION: Version = Version::new(0, 1, 0);

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    version: Version,
}

impl Metadata {
    fn write_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path)
            .with_context(|| format!("Failed to write workspace metadata to {}", path.display()))?;
        serde_json::to_writer_pretty(file, self)
            .with_context(|| format!("Failed to write workspace metadata to {}", path.display()))?;
        Ok(())
    }

    fn read_json(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| {
            format!("Failed to read workspace metadata from {}", path.display())
        })?;
        let metadata = serde_json::from_reader(file).with_context(|| {
            format!("Failed to read workspace metadata from {}", path.display())
        })?;
        Ok(metadata)
    }

    fn check_version(&self) -> Result<()> {
        // TODO: Implement version checking logic and handle version mismatch
        if self.version != WORKSPACE_VERSION {
            bail!("Workspace version mismatch.");
        }
        Ok(())
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
    /// Creates a new `WorkspacePaths` instance from a given root path.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the root path of the workspace.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the data directory path.
    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    /// Get the log directory path.
    #[must_use]
    pub fn log_dir(&self) -> PathBuf {
        self.root.join("log")
    }

    /// Get the backup directory path.
    #[must_use]
    pub fn backup_dir(&self) -> PathBuf {
        self.root.join("backup")
    }

    /// Get the IPC socket file path.
    #[must_use]
    pub fn ipc_file(&self) -> PathBuf {
        self.root.join("fricon.socket")
    }

    /// Get the database file path.
    #[must_use]
    pub fn database_file(&self) -> PathBuf {
        self.root.join("fricon.sqlite3")
    }

    /// Get the metadata file path.
    #[must_use]
    pub fn metadata_file(&self) -> PathBuf {
        self.root.join(".fricon_workspace.json")
    }

    /// Get the lock file path.
    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.root.join(".fricon.lock")
    }

    /// Get the dataset path from a UUID.
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

        // Success if path already exists.
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

        let metadata = Metadata {
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
        let metadata = Metadata::read_json(paths.metadata_file())?;
        metadata.check_version()?;

        Ok(Self { paths, _lock: lock })
    }

    /// Validate that a directory is a valid workspace without opening it.
    ///
    /// This checks for the presence of required files and validates metadata
    /// without acquiring a lock.
    pub fn validate(path: impl Into<PathBuf>) -> Result<()> {
        let paths = WorkspacePaths::new(path);

        if !paths.metadata_file().exists() {
            bail!("Not a Fricon workspace: missing metadata file");
        }

        let metadata = Metadata::read_json(paths.metadata_file())?;
        metadata.check_version()?;

        Ok(())
    }

    /// Get the paths for the current workspace.
    #[must_use]
    pub fn paths(&self) -> &WorkspacePaths {
        &self.paths
    }
}

#[derive(Debug)]
struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .context("Failed to open file for locking.")?;
        file.try_lock().context("Failed to acquire file lock.")?;
        Ok(Self { file, path })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            warn!("Failed to release file lock: {e}");
        }
        if let Err(e) = fs::remove_file(&self.path) {
            warn!("Failed to remove locked file: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use uuid::uuid;

    use super::*;

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

        // Check that all expected directories exist
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

        // Initialize workspace first
        let root1 = WorkspaceRoot::init(workspace_path).unwrap();
        drop(root1);

        // Should be able to open existing workspace
        let root2 = WorkspaceRoot::open(workspace_path).unwrap();
        assert_eq!(root2.paths().root(), workspace_path);
    }

    #[test]
    fn workspace_root_validate() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        // Should fail on non-workspace directory
        assert!(WorkspaceRoot::validate(workspace_path).is_err());

        // Initialize workspace
        let root = WorkspaceRoot::init(workspace_path).unwrap();
        drop(root);

        // Should succeed on valid workspace
        assert!(WorkspaceRoot::validate(workspace_path).is_ok());
    }

    #[test]
    fn workspace_root_exclusive_lock() {
        let temp_dir = tempdir().unwrap();
        let workspace_path = temp_dir.path();

        let _root1 = WorkspaceRoot::init(workspace_path).unwrap();

        // Should fail to open the same workspace again due to exclusive lock
        assert!(WorkspaceRoot::open(workspace_path).is_err());
    }
}

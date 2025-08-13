use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, ensure};
use chrono::Utc;
use semver::{Version, VersionReq};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    VERSION,
    database::Database,
    dataset::{self, Dataset},
    paths::{DatasetPath, VersionFile, WorkspacePath},
};

#[derive(Debug, Clone)]
pub struct Workspace(Arc<Shared>);

impl Workspace {
    pub async fn open(path: &Path) -> Result<Self> {
        let shared = Shared::open(path).await?;
        Ok(Self(Arc::new(shared)))
    }

    #[must_use]
    pub fn root(&self) -> &WorkspacePath {
        self.0.root()
    }

    #[must_use]
    pub fn database(&self) -> &Database {
        self.0.database()
    }

    pub async fn init(path: &Path) -> Result<Self> {
        let shared = Shared::init(path).await?;
        Ok(Self(Arc::new(shared)))
    }

    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<dataset::Writer> {
        self.0
            .clone()
            .create_dataset(name, description, tags, index_columns)
            .await
    }
}

#[derive(Debug)]
struct Shared {
    root: WorkspacePath,
    database: Database,
    _lock: FileLock,
}

impl Shared {
    pub async fn open(path: &Path) -> Result<Self> {
        let root = WorkspacePath::new(path)?;
        let lock = FileLock::new(root.lock_file())?;
        check_version_file(&root.version_file())?;
        let database = Database::connect(root.database_file().0).await?;
        Ok(Self {
            root,
            database,
            _lock: lock,
        })
    }

    pub async fn init(path: &Path) -> Result<Self> {
        info!("Initialize workspace: {:?}", path);
        create_empty_dir(path)?;
        let root = WorkspacePath::new(path)?;
        let database = Database::init(root.database_file().0).await?;
        init_dir(&root)?;
        write_version_file(&root.version_file())?;
        let lock = FileLock::new(root.lock_file())?;
        Ok(Self {
            root,
            database,
            _lock: lock,
        })
    }

    #[must_use]
    pub const fn root(&self) -> &WorkspacePath {
        &self.root
    }

    #[must_use]
    pub fn database(&self) -> &Database {
        &self.database
    }

    pub async fn create_dataset(
        self: Arc<Self>,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<dataset::Writer> {
        let created_at = Utc::now();
        let date = created_at.naive_local().date();
        let uid = Uuid::new_v4();
        let path = DatasetPath::new(date, uid);
        let full_path = self.root.data_dir().join(&path);
        let metadata = dataset::Metadata {
            uid,
            name,
            description,
            favorite: false,
            index_columns,
            created_at,
            tags,
        };
        let id = self
            .database()
            .create(&metadata, &path)
            .await
            .context("Failed to add dataset entry to index database.")?;
        let writer = Dataset::create(full_path, metadata, Workspace(self), id)
            .context("Failed to create dataset.")?;
        Ok(writer)
    }
}

fn create_empty_dir(path: &Path) -> Result<()> {
    // Success if path already exists.
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    let mut dir_contents = path
        .read_dir()
        .with_context(|| format!("Failed to read directory contents: {}", path.display()))?;
    ensure!(
        dir_contents.next().is_none(),
        "Directory is not empty: {:?}",
        path
    );
    Ok(())
}

fn init_dir(root: &WorkspacePath) -> Result<()> {
    fs::create_dir(root.data_dir().0).context("Failed to create data directory.")?;
    fs::create_dir(root.log_dir().0).context("Failed to create log directory.")?;
    fs::create_dir(root.backup_dir().0).context("Failed to create backup directory.")?;
    Ok(())
}

fn write_version_file(path: &VersionFile) -> Result<()> {
    let path = &path.0;
    fs::write(path, format!("{VERSION}\n")).context("Failed to write version file.")?;
    Ok(())
}

fn check_version_file(path: &VersionFile) -> Result<()> {
    let path = &path.0;
    let version_str = fs::read_to_string(path).context("Failed to read workspace version file.")?;
    let workspace_version =
        Version::parse(version_str.trim()).context("Failed to parse version.")?;
    let req =
        VersionReq::parse(&format!("={VERSION}")).expect("Failed to parse version requirement.");
    ensure!(
        req.matches(&workspace_version),
        "Version mismatch: {} != {}",
        VERSION,
        workspace_version
    );
    Ok(())
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

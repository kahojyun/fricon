use std::{fs, path::Path};

use anyhow::{ensure, Context, Result};
use arrow::datatypes::Schema;
use semver::{Version, VersionReq};
use sqlx::SqlitePool;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::Config,
    dataset::{self, Dataset},
    db,
    paths::{ConfigFile, DatasetPath, VersionFile, WorkDirectory},
    VERSION,
};

#[derive(Debug, Clone)]
pub struct Workspace {
    root: WorkDirectory,
    config: Config,
    database: SqlitePool,
}

impl Workspace {
    pub async fn open(path: &Path) -> Result<Self> {
        let root = WorkDirectory::new(path)?;
        check_version_file(&root.version_file())?;
        let config = load_config(&root.config_file())?;
        let database = db::connect(&root.database_file()).await?;
        Ok(Self {
            root,
            config,
            database,
        })
    }

    pub const fn root(&self) -> &WorkDirectory {
        &self.root
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub const fn database(&self) -> &SqlitePool {
        &self.database
    }

    pub async fn init(path: &Path) -> Result<Self> {
        info!("Initalize workspace: {:?}", path);
        create_empty_dir(path)?;
        let root = WorkDirectory::new(path)?;
        let config = Config::default();
        init_config(&root.config_file(), &config)?;
        let database = db::init(&root.database_file()).await?;
        init_dir(&root)?;
        write_version_file(&root.version_file())?;
        Ok(Self {
            root,
            config,
            database,
        })
    }

    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index: Vec<String>,
        schema: &Schema,
    ) -> Result<dataset::Writer> {
        let db = db::DatasetIndex {
            pool: self.database.clone(),
        };
        let date = chrono::Local::now().date_naive();
        let uid = Uuid::new_v4();
        let path = DatasetPath::new(date, uid);
        let id = db
            .create(uid, &name, &description, &path, &tags)
            .await
            .context("Failed to add dataset entry to index database.")?;
        let metadata = dataset::Metadata {
            uid,
            info: dataset::Info {
                name,
                description,
                tags,
                index,
            },
        };
        let writer = Dataset::create(self.root.data_dir().join(&path), id, metadata, schema)
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

fn init_config(path: &ConfigFile, config: &Config) -> Result<()> {
    let path = &path.0;
    let config_str = config.to_toml();
    info!("Initialize configuration at {}", path.display());
    fs::write(path, config_str).context("Failed to write configuration.")?;
    Ok(())
}

fn init_dir(root: &WorkDirectory) -> Result<()> {
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

fn load_config(path: &ConfigFile) -> Result<Config> {
    let path = &path.0;
    let config_str = fs::read_to_string(path).context("Failed to read configuration.")?;
    Config::from_toml(&config_str).context("Failed to parse configuration.")
}

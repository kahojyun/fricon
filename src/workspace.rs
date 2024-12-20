use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context, Result};
use semver::{Version, VersionReq};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions,
};
use tracing::info;

use crate::{
    config::Config,
    db::MIGRATOR,
    fs::{ConfigFile, DatabaseFile, VersionFile, WorkDirectory},
    VERSION,
};

#[derive(Debug)]
pub struct Workspace {
    root: WorkDirectory,
    config: Config,
}

impl Workspace {
    pub fn open(path: PathBuf) -> Result<Self> {
        let root = WorkDirectory(path);
        check_version_file(&root.version_file())?;
        let config = load_config(&root.config_file())?;
        Ok(Self { root, config })
    }

    pub const fn root(&self) -> &WorkDirectory {
        &self.root
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub async fn init(path: PathBuf) -> Result<Self> {
        info!("Initalize workspace: {:?}", path);
        create_empty_dir(&path)?;
        let root = WorkDirectory(path);
        let config = Config::default();
        init_config(&root.config_file(), &config)?;
        init_database(&root.database_file()).await?;
        init_dir(&root)?;
        write_version_file(&root.version_file())?;
        Ok(Self { root, config })
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

async fn init_database(path: &DatabaseFile) -> Result<()> {
    let path = &path.0;
    info!("Initialize database at {}", path.display());
    let mut conn = SqliteConnectOptions::new()
        .filename(path)
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .connect()
        .await
        .context("Failed to create database.")?;
    MIGRATOR
        .run(&mut conn)
        .await
        .context("Failed to initialize database schema.")?;
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

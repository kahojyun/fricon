//! Configuration

use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use directories::ProjectDirs;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    port: u16,
    data_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 22777,
            data_dir: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        if let Some(proj_dir) = ProjectDirs::from("", "", "fricon") {
            let path = proj_dir.config_dir().join("config.toml");
            println!("{:?}", path);
            let data = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&data)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn data_dir(&self) -> Option<&Path> {
        self.data_dir.as_deref()
    }
}

/// Get the project directories, return `None` if home directory is not found.
pub fn project_dirs() -> Option<&'static ProjectDirs> {
    static PROJECT_DIRS: OnceLock<Option<ProjectDirs>> = OnceLock::new();
    PROJECT_DIRS
        .get_or_init(|| ProjectDirs::from("", "", "fricon"))
        .as_ref()
}

/// Get the default configuration path, return `None` if home directory is not found.
pub fn default_config_path() -> Option<PathBuf> {
    project_dirs().map(|proj_dir| proj_dir.config_local_dir().join("config.toml"))
}

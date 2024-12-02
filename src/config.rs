//! Configuration

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self { port: 22777 }
    }
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(Error::Toml)
    }

    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("Cannot serialize configuration")
    }

    pub const fn port(&self) -> u16 {
        self.port
    }
}

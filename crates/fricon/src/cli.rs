use std::{path, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::WorkspaceRoot;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize working directory
    Init(Init),
}

#[derive(Debug, Parser)]
#[command(about = "Initialize working directory", long_about = None)]
pub struct Init {
    /// Path to working directory
    path: PathBuf,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Init(init) => init.run(),
        }
    }
}

impl Init {
    pub fn run(self) -> Result<()> {
        let _ = tracing_subscriber::fmt::try_init();
        let path = path::absolute(self.path)?;
        WorkspaceRoot::create_new(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli() {
        Init::command().debug_assert();
        Cli::command().debug_assert();
    }
}

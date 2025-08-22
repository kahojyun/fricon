//! Command line interface

use std::{
    fs,
    path::{self, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize working directory
    Init {
        /// Path to working directory
        path: PathBuf,
    },
    /// Start GUI with workspace
    Gui {
        /// Workspace path to open
        path: PathBuf,
    },
}

pub async fn main(cli: Cli) -> Result<()> {
    tracing_subscriber::fmt::init();
    match cli.command {
        Commands::Init { path } => {
            let path = path::absolute(path)?;
            fricon::init_workspace(path).await?;
        }
        Commands::Gui { path } => {
            let path = fs::canonicalize(path)?;
            fricon_ui::run_with_workspace(path).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli() {
        Cli::command().debug_assert();
    }
}

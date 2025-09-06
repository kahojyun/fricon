//! Command line interface

use std::{
    fs,
    path::{self, PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

pub use clap;

pub trait Main {
    fn main(self) -> Result<()>;
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize working directory
    Init {
        /// Path to working directory
        path: PathBuf,
    },
    /// Start GUI with workspace
    Gui(Gui),
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Gui {
    /// Workspace path to open
    path: PathBuf,
}

impl Main for Cli {
    fn main(self) -> Result<()> {
        match self.command {
            Commands::Init { path } => {
                tracing_subscriber::fmt::init();
                let path = path::absolute(path)?;
                fricon::WorkspaceRoot::create_new(path)?;
            }
            Commands::Gui(gui) => {
                gui.main()?;
            }
        }
        Ok(())
    }
}

impl Main for Gui {
    fn main(self) -> Result<()> {
        let path = fs::canonicalize(self.path)?;
        fricon_ui::run_with_workspace(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli() {
        Gui::command().debug_assert();
        Cli::command().debug_assert();
    }
}

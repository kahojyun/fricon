//! Command line interface

use std::path::PathBuf;

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
    /// Start server
    Serve {
        /// Path to working directory
        path: PathBuf,
    },
    /// Start GUI
    Gui,
}

/// Main entry point for the application
///
/// # Errors
///
/// Returns a boxed error if server initialization or operation fails
pub async fn main(cli: Cli) -> Result<()> {
    tracing_subscriber::fmt::init();
    match cli.command {
        Commands::Init { path } => {
            fricon::workspace::Workspace::init(&path).await?;
        }
        Commands::Serve { path } => {
            fricon::server::run(&path).await?;
        }
        Commands::Gui => {
            fricon_ui::run();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn test_cli() {
        Cli::command().debug_assert();
    }
}

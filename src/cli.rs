//! Command line interface

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the configuration file. If not provided, load from default location.
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    /// The port to listen on. Overrides configuration file.
    #[arg(short, long)]
    pub port: Option<u16>,
    /// The data directory. Overrides configuration file.
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show default file location.
    Config,
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

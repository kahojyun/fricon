//! Command line interface

use std::path::{self, PathBuf};

use anyhow::Result;
pub use clap;
use clap::{Parser, Subcommand};
use tracing_subscriber::fmt;

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
                fmt::init();
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
        self.main_with_command_name("fricon")
    }
}

impl Gui {
    pub fn main_with_command_name(self, command_name: impl Into<String>) -> Result<()> {
        let command_name = command_name.into();
        let cli_help = render_help_for_command::<Cli>(&command_name)?;
        self.main_with_help(command_name, cli_help)
    }

    pub fn main_with_help(self, command_name: String, cli_help: String) -> Result<()> {
        launch_gui_with_context(command_name, cli_help, Some(self.path))
    }
}

pub fn launch_gui_with_context(
    command_name: String,
    cli_help: String,
    workspace_path: Option<PathBuf>,
) -> Result<()> {
    fricon_ui::run_with_context(fricon_ui::LaunchContext {
        launch_source: fricon_ui::LaunchSource::Cli {
            command_name,
            cli_help,
        },
        workspace_path,
    })
}

pub fn render_help_for_command<T: clap::CommandFactory>(bin_name: &str) -> Result<String> {
    let mut command = T::command();
    command = command.bin_name(bin_name);
    let mut help = Vec::new();
    command.write_long_help(&mut help)?;
    Ok(String::from_utf8_lossy(&help).into_owned())
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

use std::{
    io::{IsTerminal, stderr, stdout},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Gui {
    /// Workspace path to open
    path: Option<PathBuf>,
    /// Force dialog mode even when running in a terminal
    #[arg(long)]
    force_dialog: bool,
}

impl Gui {
    pub fn run(self) -> Result<()> {
        self.run_with_command_name("fricon-ui")
    }

    pub fn run_standalone(self, default_workspace_path: Option<PathBuf>) -> Result<()> {
        crate::run_with_context(&crate::LaunchContext {
            launch_source: crate::LaunchSource::Standalone,
            workspace_path: self.path.or(default_workspace_path),
            interaction_mode: crate::InteractionMode::Dialog,
        })
    }

    pub fn run_with_command_name(self, command_name: impl Into<String>) -> Result<()> {
        let command_name = command_name.into();
        let cli_help = render_help_for_command::<Gui>(&command_name)?;
        self.run_with_help(command_name, cli_help)
    }

    pub fn run_with_help(self, command_name: String, cli_help: String) -> Result<()> {
        launch_gui_with_context(command_name, cli_help, self.path, self.force_dialog)
    }
}

pub fn launch_gui_with_context(
    command_name: String,
    cli_help: String,
    workspace_path: Option<PathBuf>,
    force_dialog: bool,
) -> Result<()> {
    let interaction_mode = detect_interaction_mode(force_dialog, has_tty());
    crate::run_with_context(&crate::LaunchContext {
        launch_source: crate::LaunchSource::Cli {
            command_name,
            cli_help,
        },
        workspace_path,
        interaction_mode,
    })
}

fn has_tty() -> bool {
    stdout().is_terminal() || stderr().is_terminal()
}

fn detect_interaction_mode(force_dialog: bool, has_tty: bool) -> crate::InteractionMode {
    if force_dialog {
        crate::InteractionMode::Dialog
    } else if has_tty {
        crate::InteractionMode::Terminal
    } else {
        crate::InteractionMode::Dialog
    }
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
    fn gui() {
        Gui::command().debug_assert();
    }

    #[test]
    fn force_dialog_overrides_terminal_detection() {
        assert_eq!(
            detect_interaction_mode(true, true),
            crate::InteractionMode::Dialog
        );
    }

    #[test]
    fn terminal_mode_when_tty_without_force_dialog() {
        assert_eq!(
            detect_interaction_mode(false, true),
            crate::InteractionMode::Terminal
        );
    }

    #[test]
    fn dialog_mode_when_no_tty_without_force_dialog() {
        assert_eq!(
            detect_interaction_mode(false, false),
            crate::InteractionMode::Dialog
        );
    }

    #[test]
    fn gui_cli_parses_without_path_argument() {
        let parsed = Gui::try_parse_from(["fricon-ui"]);
        assert!(parsed.is_ok());
    }

    #[test]
    fn gui_cli_parses_force_dialog_without_path_argument() {
        let parsed = Gui::try_parse_from(["fricon-ui", "--force-dialog"]);
        assert!(parsed.is_ok());
    }
}

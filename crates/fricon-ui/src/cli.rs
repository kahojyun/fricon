use std::{
    env,
    io::{IsTerminal, stderr, stdout},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct GuiArgs {
    /// Workspace path to open
    path: Option<PathBuf>,
    /// Force dialog mode even when running in a terminal
    #[arg(long)]
    force_dialog: bool,
}

impl GuiArgs {
    fn launch_standalone(self, default_workspace_path: Option<PathBuf>) -> Result<()> {
        crate::run_with_context(&crate::LaunchContext {
            launch_source: crate::LaunchSource::Standalone,
            workspace_path: self.path.or(default_workspace_path),
            interaction_mode: crate::InteractionMode::Dialog,
        })
    }

    pub fn launch_with_cli_context(self, command_name: String, cli_help: String) -> Result<()> {
        launch_gui_with_context(command_name, cli_help, self.path, self.force_dialog)
    }
}

pub fn run_standalone_from_env() -> Result<()> {
    let _ = dotenv();
    let default_workspace_path = env::var("FRICON_WORKSPACE").ok().map(PathBuf::from);
    let gui_args = parse_gui_args_or_fallback(env::args_os(), has_tty()).unwrap_or_else(|error| {
        error.exit();
    });
    gui_args.launch_standalone(default_workspace_path)
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

pub fn parse_gui_args_or_fallback<I, T>(
    argv: I,
    has_console_output: bool,
) -> std::result::Result<GuiArgs, clap::error::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    match GuiArgs::try_parse_from(argv) {
        Ok(gui_args) => Ok(gui_args),
        Err(parse_error) if has_console_output => Err(parse_error),
        Err(_) => Ok(GuiArgs {
            path: None,
            force_dialog: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn gui_args() {
        GuiArgs::command().debug_assert();
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
        let parsed = GuiArgs::try_parse_from(["fricon-ui"]);
        assert!(parsed.is_ok());
    }

    #[test]
    fn gui_cli_parses_force_dialog_without_path_argument() {
        let parsed = GuiArgs::try_parse_from(["fricon-ui", "--force-dialog"]);
        assert!(parsed.is_ok());
    }

    #[test]
    fn standalone_entrypoint_ignores_unknown_args_without_console_output() {
        let parsed = parse_gui_args_or_fallback(["fricon-ui", "--launcher-token"], false);
        let gui_args = parsed.expect("non-console launches should ignore unknown args");
        assert!(gui_args.path.is_none());
        assert!(!gui_args.force_dialog);
    }

    #[test]
    fn standalone_entrypoint_reports_unknown_args_with_console_output() {
        let parsed = parse_gui_args_or_fallback(["fricon-ui", "--launcher-token"], true);
        assert!(parsed.is_err());
    }
}

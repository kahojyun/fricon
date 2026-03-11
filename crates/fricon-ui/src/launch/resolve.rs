use std::{fs, path::PathBuf};

use anyhow::Result;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

use crate::launch::{InteractionMode, LaunchContext, LaunchSource, WorkspaceLaunchError};

enum MissingWorkspaceAction {
    ChooseWorkspace,
    ShowCliHelpAndExit,
    Exit,
}

const CHOOSE_WORKSPACE_BUTTON: &str = "Choose workspace";
const HELP_BUTTON: &str = "Help";
const EXIT_BUTTON: &str = "Exit";

pub(crate) fn resolve_workspace_path(context: &LaunchContext) -> Result<Option<PathBuf>> {
    match &context.workspace_path {
        Some(path) => match validate_workspace_path(path) {
            Ok(path) => Ok(Some(path)),
            Err(err) => match context.interaction_mode {
                InteractionMode::Dialog => Ok(None),
                InteractionMode::Terminal => Err(err.into()),
            },
        },
        None => match context.interaction_mode {
            InteractionMode::Dialog => Ok(None),
            InteractionMode::Terminal => Err(WorkspaceLaunchError::WorkspacePathMissing.into()),
        },
    }
}

pub(crate) fn validate_workspace_path(
    path: &std::path::Path,
) -> std::result::Result<PathBuf, WorkspaceLaunchError> {
    let canonical =
        fs::canonicalize(path).map_err(|err| WorkspaceLaunchError::WorkspacePathInvalid {
            path: Some(path.to_path_buf()),
            reason: err.to_string(),
        })?;
    fricon::WorkspaceRoot::validate(canonical.clone()).map_err(|err| {
        WorkspaceLaunchError::WorkspacePathInvalid {
            path: Some(canonical.clone()),
            reason: err.to_string(),
        }
    })?;
    Ok(canonical)
}

pub(crate) fn select_workspace_path(launch_source: &LaunchSource) -> Result<Option<PathBuf>> {
    loop {
        match prompt_missing_workspace_action(launch_source) {
            MissingWorkspaceAction::ChooseWorkspace => {}
            MissingWorkspaceAction::ShowCliHelpAndExit => {
                if let LaunchSource::Cli {
                    command_name,
                    cli_help,
                } = launch_source
                {
                    show_cli_help(command_name, cli_help);
                }
                return Ok(None);
            }
            MissingWorkspaceAction::Exit => return Ok(None),
        }

        let Some(path) = FileDialog::new().pick_folder() else {
            return Ok(None);
        };

        match validate_workspace_path(&path) {
            Ok(path) => return Ok(Some(path)),
            Err(err) => {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Invalid workspace")
                    .set_description(err.to_string())
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    }
}

fn dialog_is_choose_workspace(result: &MessageDialogResult) -> bool {
    match result {
        MessageDialogResult::Ok | MessageDialogResult::Yes => true,
        MessageDialogResult::Custom(value) => value == CHOOSE_WORKSPACE_BUTTON,
        MessageDialogResult::No | MessageDialogResult::Cancel => false,
    }
}

fn prompt_missing_workspace_action(launch_source: &LaunchSource) -> MissingWorkspaceAction {
    let action = match launch_source {
        LaunchSource::Standalone => MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Workspace not found")
            .set_description(
                "No valid workspace path is available.\n\nChoose a workspace folder, or exit.",
            )
            .set_buttons(MessageButtons::OkCancelCustom(
                CHOOSE_WORKSPACE_BUTTON.to_string(),
                EXIT_BUTTON.to_string(),
            ))
            .show(),
        LaunchSource::Cli { .. } => MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Workspace not found")
            .set_description(
                "No valid workspace path is available.\n\nChoose a workspace folder, or view \
                 command line help.",
            )
            .set_buttons(MessageButtons::OkCancelCustom(
                CHOOSE_WORKSPACE_BUTTON.to_string(),
                HELP_BUTTON.to_string(),
            ))
            .show(),
    };

    if dialog_is_choose_workspace(&action) {
        return MissingWorkspaceAction::ChooseWorkspace;
    }

    match launch_source {
        LaunchSource::Standalone => MissingWorkspaceAction::Exit,
        LaunchSource::Cli { .. } => MissingWorkspaceAction::ShowCliHelpAndExit,
    }
}

fn show_cli_help(command_name: &str, cli_help: &str) {
    MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("Command line help")
        .set_description(build_cli_help_message(command_name, cli_help))
        .set_buttons(MessageButtons::Ok)
        .show();
}

fn build_cli_help_message(command_name: &str, cli_help: &str) -> String {
    format!("Command: {command_name}\n\n{cli_help}")
}

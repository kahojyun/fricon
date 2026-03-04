use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchSource {
    Standalone,
    Cli {
        command_name: String,
        cli_help: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Dialog,
    Terminal,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceLaunchError {
    #[error("workspace path is required")]
    WorkspacePathMissing,
    #[error(
        "invalid workspace path '{}': {reason}",
        .path
            .as_ref()
            .map_or_else(|| "<none>".to_string(), |p| p.display().to_string())
    )]
    WorkspacePathInvalid {
        path: Option<PathBuf>,
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct LaunchContext {
    pub launch_source: LaunchSource,
    pub workspace_path: Option<PathBuf>,
    pub interaction_mode: InteractionMode,
}

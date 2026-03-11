mod application;
mod chart_data;
mod desktop_runtime;
mod launch;
mod tauri_api;

use anyhow::Result;

use crate::desktop_runtime::{
    logging::init_tracing_subscriber,
    panic_hook::install_panic_hook,
    runtime::{run_with_context_dialog_mode, run_with_context_terminal_mode},
};
pub use crate::{
    launch::{InteractionMode, LaunchContext, LaunchSource, WorkspaceLaunchError},
    tauri_api::export_bindings,
};

/// Run the application with the given launch context.
///
/// This function initializes the tracing subscriber, installs the panic hook,
/// and delegates to the appropriate mode handler based on the interaction mode.
///
/// Called at most once during application startup.
pub fn run_with_context(context: &LaunchContext) -> Result<()> {
    init_tracing_subscriber()?;
    install_panic_hook(context.interaction_mode);
    match context.interaction_mode {
        InteractionMode::Terminal => run_with_context_terminal_mode(context),
        InteractionMode::Dialog => run_with_context_dialog_mode(context),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, panic::Location, path::PathBuf};

    use fricon::WorkspaceRoot;
    use tempfile::tempdir;

    use super::{
        InteractionMode, LaunchContext, LaunchSource, WorkspaceLaunchError,
        desktop_runtime::panic_hook::build_panic_dialog_message,
        launch::resolve::{resolve_workspace_path, validate_workspace_path},
    };

    #[test]
    fn terminal_mode_missing_workspace_returns_workspace_missing_error() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: None,
            interaction_mode: InteractionMode::Terminal,
        });
        let error = result.expect_err("expected missing-workspace error");
        let launch_error = error
            .downcast_ref::<WorkspaceLaunchError>()
            .expect("error should be WorkspaceLaunchError");
        assert!(matches!(
            launch_error,
            WorkspaceLaunchError::WorkspacePathMissing
        ));
    }

    #[test]
    fn terminal_mode_invalid_workspace_returns_workspace_invalid_error() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: Some(PathBuf::from("/definitely/not/a/real/path")),
            interaction_mode: InteractionMode::Terminal,
        });
        let error = result.expect_err("expected invalid-workspace error");
        let launch_error = error
            .downcast_ref::<WorkspaceLaunchError>()
            .expect("error should be WorkspaceLaunchError");
        assert!(matches!(
            launch_error,
            WorkspaceLaunchError::WorkspacePathInvalid { .. }
        ));
    }

    #[test]
    fn dialog_mode_invalid_workspace_defers_to_picker_flow() {
        let result = resolve_workspace_path(&LaunchContext {
            launch_source: LaunchSource::Standalone,
            workspace_path: Some(PathBuf::from("/definitely/not/a/real/path")),
            interaction_mode: InteractionMode::Dialog,
        });
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn validate_workspace_path_accepts_valid_workspace() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let workspace_path = temp_dir.path().join("workspace");
        let workspace =
            WorkspaceRoot::create_new(workspace_path.clone()).expect("workspace should be created");
        drop(workspace);

        let result = validate_workspace_path(&workspace_path);
        let expected =
            fs::canonicalize(workspace_path).expect("workspace path should canonicalize");
        assert_eq!(result.expect("workspace should validate"), expected);
    }

    #[test]
    fn validate_workspace_path_rejects_non_workspace_directory() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let non_workspace_dir = temp_dir.path().join("not-workspace");
        fs::create_dir_all(&non_workspace_dir).expect("directory should be created");

        let result = validate_workspace_path(&non_workspace_dir);
        assert!(matches!(
            result,
            Err(WorkspaceLaunchError::WorkspacePathInvalid { .. })
        ));
    }

    #[test]
    fn panic_dialog_message_includes_reason_and_location() {
        let message = build_panic_dialog_message(&"boom", Some(Location::caller()));
        assert!(message.contains("Reason: boom"));
        assert!(message.contains("Location: "));
    }
}

use std::path::PathBuf;

use anyhow::Context;

use crate::application::session::WorkspaceSession;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceInfo {
    pub(crate) path: PathBuf,
}

pub(crate) fn get_workspace_info(session: &WorkspaceSession) -> anyhow::Result<WorkspaceInfo> {
    let workspace_paths = session
        .app()
        .paths()
        .context("Failed to retrieve workspace paths.")?;
    Ok(WorkspaceInfo {
        path: workspace_paths.root().to_path_buf(),
    })
}

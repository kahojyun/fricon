use anyhow::Result;
use fricon::AppManager;
use tauri::async_runtime;

use crate::desktop_runtime::{runtime_owner::RuntimeOwner, session::WorkspaceSession};

pub(crate) struct AppState {
    runtime: RuntimeOwner,
    session: WorkspaceSession,
}

impl AppState {
    pub(crate) fn new(workspace_path: std::path::PathBuf) -> Result<Self> {
        let runtime = async_runtime::handle();
        let app_manager = AppManager::new_with_path(workspace_path)?.start(runtime.inner())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());
        Ok(Self {
            runtime: RuntimeOwner::new(app_manager),
            session,
        })
    }

    pub(crate) fn session(&self) -> &WorkspaceSession {
        &self.session
    }

    pub(crate) fn shutdown(&self) {
        self.runtime.shutdown();
    }
}

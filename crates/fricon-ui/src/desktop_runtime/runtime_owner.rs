use std::sync::Mutex;

use fricon::AppManager;
use tauri::async_runtime;

pub(crate) struct RuntimeOwner {
    manager: Mutex<Option<AppManager>>,
}

impl RuntimeOwner {
    pub(crate) fn new(manager: AppManager) -> Self {
        Self {
            manager: Mutex::new(Some(manager)),
        }
    }

    pub(crate) fn shutdown(&self) {
        async_runtime::block_on(async {
            let app_manager = self
                .manager
                .lock()
                .expect("Failed to acquire lock on app state")
                .take()
                .expect("App should be running");
            app_manager.shutdown().await;
        });
    }
}

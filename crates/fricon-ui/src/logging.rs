use std::{
    io::{self, IsTerminal},
    path::Path,
    sync::{LazyLock, Mutex},
};

use anyhow::{Context as _, Result};
use tracing::{level_filters::LevelFilter, warn};
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    EnvFilter, fmt,
    fmt::format::{Format, Json, JsonFields},
    layer::SubscriberExt as _,
    prelude::*,
    registry::Registry,
    reload,
};

type FileLayer = fmt::Layer<Registry, JsonFields, Format<Json>, NonBlocking>;

#[derive(Default)]
struct LogManager {
    handle: Option<reload::Handle<Option<FileLayer>, Registry>>,
    guard: Option<WorkerGuard>,
    active_generation: u64,
}

impl LogManager {
    fn init(&mut self, handle: reload::Handle<Option<FileLayer>, Registry>) {
        self.handle = Some(handle);
    }

    fn attach(
        &mut self,
        layer: FileLayer,
        guard: WorkerGuard,
    ) -> Result<(u64, Option<WorkerGuard>)> {
        let handle = self
            .handle
            .as_ref()
            .context("tracing subscriber is not initialized")?;
        handle
            .modify(|l| *l = Some(layer))
            .context("failed to reload workspace file logging layer")?;

        self.active_generation = self.active_generation.wrapping_add(1);
        let old_guard = self.guard.replace(guard);

        Ok((self.active_generation, old_guard))
    }

    fn detach(&mut self, generation: u64) -> Option<WorkerGuard> {
        if self.active_generation == generation {
            if let Some(handle) = &self.handle
                && let Err(err) = handle.modify(|l| *l = None)
            {
                warn!(error = %err, "failed to disable workspace file logging layer");
            }
            self.guard.take()
        } else {
            None
        }
    }

    fn shutdown(&mut self) -> Option<WorkerGuard> {
        if let Some(handle) = &self.handle
            && let Err(err) = handle.modify(|l| *l = None)
        {
            warn!(error = %err, "failed to shutdown workspace file logging layer");
        }
        self.active_generation = self.active_generation.wrapping_add(1);
        self.guard.take()
    }
}

fn get_manager() -> std::sync::MutexGuard<'static, LogManager> {
    static LOG_MANAGER: LazyLock<Mutex<LogManager>> = LazyLock::new(Default::default);

    LOG_MANAGER.lock().expect("log manager lock poisoned")
}

pub(crate) struct WorkspaceLogSession {
    generation: u64,
}

impl Drop for WorkspaceLogSession {
    fn drop(&mut self) {
        let old_guard = get_manager().detach(self.generation);
        drop(old_guard); // dropped outside lock to avoid blocking on flush
    }
}

pub(crate) fn shutdown_workspace_file_logging() {
    let old_guard = get_manager().shutdown();
    drop(old_guard);
}

pub(crate) fn init_tracing_subscriber() -> Result<()> {
    let (file_layer, file_layer_handle) = reload::Layer::new(None);
    let stdout_layer = if io::stdout().is_terminal() {
        Some(fmt::layer().with_writer(io::stdout))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init()
        .context("Failed to initialize logging")?;

    get_manager().init(file_layer_handle);
    Ok(())
}

pub(crate) fn attach_workspace_file_logging(workspace_path: &Path) -> Result<WorkspaceLogSession> {
    let log_dir = fricon::get_log_dir(workspace_path.to_path_buf())?;
    let rolling = RollingFileAppender::new(Rotation::DAILY, log_dir, "fricon.log");
    let (writer, guard) = tracing_appender::non_blocking(rolling);
    let file_layer = fmt::layer().json().with_writer(writer);

    let (generation, old_guard) = get_manager().attach(file_layer, guard)?;
    drop(old_guard);

    Ok(WorkspaceLogSession { generation })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pure unit tests for LogManager logic without touching global state
    #[test]
    fn test_log_manager_attach_detach() {
        let (_layer, reload_handle) = reload::Layer::new(None);
        let mut manager = LogManager::default();
        manager.init(reload_handle.clone());

        // Attach 1
        let (writer1, guard1) = tracing_appender::non_blocking(std::io::sink());
        let file_layer1 = fmt::layer().json().with_writer(writer1);
        let (gen_val, old1) = manager.attach(file_layer1, guard1).unwrap();

        assert!(old1.is_none());
        assert_eq!(gen_val, 1);
        assert!(manager.guard.is_some());

        // Ensure layer is actually loaded
        reload_handle
            .with_current(|l| assert!(l.is_some()))
            .unwrap();

        // Detach with wrong generation should ignore
        let old_wrong = manager.detach(999);
        assert!(old_wrong.is_none());
        assert!(manager.guard.is_some());
        reload_handle
            .with_current(|l| assert!(l.is_some()))
            .unwrap();

        // Detach with correct generation should clear
        let old_correct = manager.detach(gen_val);
        assert!(old_correct.is_some());
        assert!(manager.guard.is_none());
        reload_handle
            .with_current(|l| assert!(l.is_none()))
            .unwrap();
    }

    #[test]
    fn test_log_manager_attach_overlap() {
        let (_layer, reload_handle) = reload::Layer::new(None);
        let mut manager = LogManager::default();
        manager.init(reload_handle.clone());

        let (writer1, guard1) = tracing_appender::non_blocking(std::io::sink());
        let layer1 = fmt::layer().json().with_writer(writer1);
        let (gen_val1, _old1) = manager.attach(layer1, guard1).unwrap();

        let (writer2, guard2) = tracing_appender::non_blocking(std::io::sink());
        let layer2 = fmt::layer().json().with_writer(writer2);
        let (gen_val2, old2) = manager.attach(layer2, guard2).unwrap();

        // Second attach should replace first guard
        assert!(old2.is_some());
        assert_eq!(gen_val2, 2);

        // Detaching first generation should do nothing because gen2 is active
        let old_detach1 = manager.detach(gen_val1);
        assert!(old_detach1.is_none());
        assert!(manager.guard.is_some());

        // Detaching second generation should clear
        let old_detach2 = manager.detach(gen_val2);
        assert!(old_detach2.is_some());
        assert!(manager.guard.is_none());
        reload_handle
            .with_current(|l| assert!(l.is_none()))
            .unwrap();
    }

    #[test]
    fn test_log_manager_shutdown() {
        let (_layer, reload_handle) = reload::Layer::new(None);
        let mut manager = LogManager::default();
        manager.init(reload_handle.clone());

        let (writer, guard) = tracing_appender::non_blocking(std::io::sink());
        let layer = fmt::layer().json().with_writer(writer);
        let (gen_val, _old) = manager.attach(layer, guard).unwrap();

        let shutdown_guard = manager.shutdown();
        assert!(shutdown_guard.is_some());
        assert!(manager.guard.is_none());
        reload_handle
            .with_current(|l| assert!(l.is_none()))
            .unwrap();

        // Further detach with old gen should do nothing
        assert!(manager.detach(gen_val).is_none());
    }
}

use std::{
    io::{self, IsTerminal},
    path::Path,
    sync::{Mutex, OnceLock},
};

use anyhow::{Context as _, Result, bail};
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
struct FileLoggingState {
    generation: u64,
    handle: Option<reload::Handle<Option<FileLayer>, Registry>>,
    guard: Option<WorkerGuard>,
}

#[derive(Default)]
struct LoggingRuntime {
    file_state: Mutex<FileLoggingState>,
}

static LOGGING_RUNTIME: OnceLock<LoggingRuntime> = OnceLock::new();
static SUBSCRIBER_INIT_LOCK: OnceLock<Mutex<bool>> = OnceLock::new();

fn logging_runtime() -> &'static LoggingRuntime {
    LOGGING_RUNTIME.get_or_init(LoggingRuntime::default)
}

fn disable_file_layer(invalidate_sessions: bool) {
    let runtime = logging_runtime();
    let (handle, old_guard) = {
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        if invalidate_sessions {
            // Invalidate all active sessions so their subsequent drop calls are no-ops.
            state.generation = state.generation.wrapping_add(1);
        }

        let handle = state.handle.clone();
        let old_guard = state.guard.take();
        (handle, old_guard)
    };

    if let Some(handle) = handle
        && let Err(err) = handle.modify(|layer| {
            *layer = None;
        })
    {
        warn!(error = %err, "failed to disable workspace file logging layer");
    }

    drop(old_guard);
}

pub(crate) struct WorkspaceLogSession {
    generation: u64,
}

impl Drop for WorkspaceLogSession {
    fn drop(&mut self) {
        let runtime = logging_runtime();
        let should_disable = {
            let state = runtime
                .file_state
                .lock()
                .expect("logging state should not be poisoned");
            state.generation == self.generation
        };

        if should_disable {
            disable_file_layer(false);
        }
    }
}

pub(crate) fn shutdown_workspace_file_logging() {
    disable_file_layer(true);
}

pub(crate) fn init_tracing_subscriber() -> Result<()> {
    let init_lock = SUBSCRIBER_INIT_LOCK.get_or_init(|| Mutex::new(false));
    let mut initialized = init_lock
        .lock()
        .expect("logging init state should not be poisoned");
    if *initialized {
        return Ok(());
    }

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

    {
        let runtime = logging_runtime();
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        state.handle = Some(file_layer_handle);
    }

    *initialized = true;
    Ok(())
}

pub(crate) fn attach_workspace_file_logging(workspace_path: &Path) -> Result<WorkspaceLogSession> {
    let log_dir = fricon::get_log_dir(workspace_path.to_path_buf())?;
    let rolling = RollingFileAppender::new(Rotation::DAILY, log_dir, "fricon.log");
    let (writer, guard) = tracing_appender::non_blocking(rolling);
    let file_layer = fmt::layer().json().with_writer(writer);

    let runtime = logging_runtime();
    let (generation, old_guard, handle) = {
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        let Some(handle) = state.handle.clone() else {
            bail!("tracing subscriber is not initialized");
        };

        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        let old_guard = state.guard.take();
        state.guard = Some(guard);
        (generation, old_guard, handle)
    };

    handle
        .modify(|layer| {
            *layer = Some(file_layer);
        })
        .context("Failed to reload workspace file logging layer")?;

    drop(old_guard);

    Ok(WorkspaceLogSession { generation })
}

#[cfg(test)]
fn has_active_file_logging() -> bool {
    let runtime = logging_runtime();
    let state = runtime
        .file_state
        .lock()
        .expect("logging state should not be poisoned");
    state.guard.is_some()
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use fricon::WorkspaceRoot;
    use tempfile::tempdir;

    use super::{
        attach_workspace_file_logging, has_active_file_logging, init_tracing_subscriber,
        shutdown_workspace_file_logging,
    };

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn init_tracing_subscriber_is_idempotent() {
        let _guard = test_lock()
            .lock()
            .expect("test lock should not be poisoned");
        init_tracing_subscriber().expect("first init should succeed");
        init_tracing_subscriber().expect("second init should be a no-op");
    }

    #[test]
    fn attach_drop_and_attach_again_is_supported() {
        let _guard = test_lock()
            .lock()
            .expect("test lock should not be poisoned");
        init_tracing_subscriber().expect("subscriber init should succeed");

        let temp_dir = tempdir().expect("tempdir should be created");
        let workspace_path = temp_dir.path().join("workspace");
        let workspace =
            WorkspaceRoot::create_new(workspace_path.clone()).expect("workspace should be created");
        drop(workspace);

        {
            let session = attach_workspace_file_logging(&workspace_path)
                .expect("attach logging should succeed");
            assert!(has_active_file_logging());
            drop(session);
        }
        assert!(!has_active_file_logging());

        let _session = attach_workspace_file_logging(&workspace_path)
            .expect("reattach logging should succeed");
        assert!(has_active_file_logging());
    }

    #[test]
    fn attach_rejects_invalid_workspace() {
        let _guard = test_lock()
            .lock()
            .expect("test lock should not be poisoned");
        init_tracing_subscriber().expect("subscriber init should succeed");

        let temp_dir = tempdir().expect("tempdir should be created");
        let invalid_workspace = temp_dir.path().join("invalid-workspace");
        let result = attach_workspace_file_logging(&invalid_workspace);

        assert!(result.is_err());
    }

    #[test]
    fn explicit_shutdown_invalidates_old_sessions() {
        let _guard = test_lock()
            .lock()
            .expect("test lock should not be poisoned");
        init_tracing_subscriber().expect("subscriber init should succeed");

        let temp_dir = tempdir().expect("tempdir should be created");
        let workspace_path = temp_dir.path().join("workspace");
        let workspace =
            WorkspaceRoot::create_new(workspace_path.clone()).expect("workspace should be created");
        drop(workspace);

        let session_1 = attach_workspace_file_logging(&workspace_path)
            .expect("first attach logging should succeed");
        assert!(has_active_file_logging());

        shutdown_workspace_file_logging();
        assert!(!has_active_file_logging());

        let session_2 = attach_workspace_file_logging(&workspace_path)
            .expect("second attach logging should succeed");
        assert!(has_active_file_logging());

        drop(session_1);
        assert!(has_active_file_logging());

        drop(session_2);
        assert!(!has_active_file_logging());
    }
}

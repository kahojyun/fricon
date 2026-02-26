use std::{
    io::{self, IsTerminal, Write},
    path::Path,
    sync::{Mutex, OnceLock},
};

use anyhow::{Context as _, Result};
use tracing::level_filters::LevelFilter;
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Default)]
struct FileLoggingState {
    generation: u64,
    writer: Option<NonBlocking>,
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

struct DynamicFileWriter;

impl Write for DynamicFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let runtime = logging_runtime();
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        if let Some(writer) = state.writer.as_mut() {
            writer.write(buf)
        } else {
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let runtime = logging_runtime();
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        if let Some(writer) = state.writer.as_mut() {
            writer.flush()
        } else {
            Ok(())
        }
    }
}

pub(crate) struct WorkspaceLogSession {
    generation: u64,
}

impl Drop for WorkspaceLogSession {
    fn drop(&mut self) {
        let runtime = logging_runtime();
        let guard = {
            let mut state = runtime
                .file_state
                .lock()
                .expect("logging state should not be poisoned");
            if state.generation != self.generation {
                return;
            }
            state.writer = None;
            state.guard.take()
        };
        drop(guard);
    }
}

pub(crate) fn shutdown_workspace_file_logging() {
    let runtime = logging_runtime();
    let guard = {
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        // Invalidate all active sessions so their subsequent drop calls are no-ops.
        state.generation = state.generation.wrapping_add(1);
        state.writer = None;
        state.guard.take()
    };
    drop(guard);
}

pub(crate) fn init_tracing_subscriber() -> Result<()> {
    let init_lock = SUBSCRIBER_INIT_LOCK.get_or_init(|| Mutex::new(false));
    let mut initialized = init_lock
        .lock()
        .expect("logging init state should not be poisoned");
    if *initialized {
        return Ok(());
    }

    let file_layer = fmt::layer().json().with_writer(|| DynamicFileWriter);
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

    *initialized = true;
    Ok(())
}

pub(crate) fn attach_workspace_file_logging(workspace_path: &Path) -> Result<WorkspaceLogSession> {
    let log_dir = fricon::get_log_dir(workspace_path.to_path_buf())?;
    let rolling = RollingFileAppender::new(Rotation::DAILY, log_dir, "fricon.log");
    let (writer, guard) = tracing_appender::non_blocking(rolling);

    let runtime = logging_runtime();
    let (generation, old_guard) = {
        let mut state = runtime
            .file_state
            .lock()
            .expect("logging state should not be poisoned");
        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        state.writer = Some(writer);
        let old_guard = state.guard.take();
        state.guard = Some(guard);
        (generation, old_guard)
    };
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
    state.writer.is_some() && state.guard.is_some()
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

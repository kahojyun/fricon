use std::path::PathBuf;

use anyhow::Result;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::get_runtime;

pub fn main_impl<T: fricon_cli::clap::Parser + fricon_cli::Main>(py: Python<'_>) -> i32 {
    fn ignore_python_sigint(py: Python<'_>) -> PyResult<()> {
        let signal = py.import("signal")?;
        let sigint = signal.getattr("SIGINT")?;
        let default_handler = signal.getattr("SIG_DFL")?;
        _ = signal.call_method1("signal", (sigint, default_handler))?;
        Ok(())
    }

    if ignore_python_sigint(py).is_err() {
        eprintln!("Failed to reset python SIGINT handler.");
        return 1;
    }

    // Skip python executable
    let argv = std::env::args_os().skip(1);
    let cli = T::parse_from(argv);
    match cli.main() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e:?}");
            1
        }
    }
}

/// Main CLI entry point that delegates to fricon-cli binary.
///
/// Returns:
///     Exit code.
#[pyfunction]
#[must_use]
pub fn main(py: Python<'_>) -> i32 {
    main_impl::<fricon_cli::Cli>(py)
}

/// GUI only CLI entry point.
///
/// Returns:
///     Exit code.
#[pyfunction]
#[must_use]
pub fn main_gui(py: Python<'_>) -> i32 {
    main_impl::<fricon_cli::Gui>(py)
}

/// Create a workspace for integration testing.
///
/// This function creates a new workspace at the given path and starts a server.
/// The server will run in the background and the workspace client is returned.
/// It's not exported to the public API and should only be used for testing.
///
/// Note: The server runs in the background. When the workspace client is dropped,
/// the connection is closed but the server continues running. For proper cleanup,
/// you may need to manually stop the server process.
///
/// Parameters:
///     path: The path where to create the workspace.
///
/// Returns:
///     A workspace client connected to the newly created workspace.
#[pyfunction]
pub fn serve_workspace(path: PathBuf) -> Result<crate::workspace::Workspace> {
    let runtime = get_runtime();

    // Create the workspace first
    let root = fricon::WorkspaceRoot::create_new(&path)?;

    // Start the server in the background
    let _manager = runtime.block_on(fricon::AppManager::serve(root))?;

    // Wait a bit for the server to start up
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Try to connect with retries
    let mut retries = 0;
    loop {
        match crate::workspace::Workspace::connect(path.clone()) {
            Ok(workspace) => return Ok(workspace),
            Err(e) => {
                retries += 1;
                if retries > 10 {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

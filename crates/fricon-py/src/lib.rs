#![allow(
    clippy::missing_errors_doc,
    reason = "Python bindings don't require complete error documentation"
)]
#![allow(
    clippy::missing_panics_doc,
    reason = "Python bindings don't require complete panic documentation"
)]
#![allow(
    clippy::doc_markdown,
    reason = "Markdown in docs is acceptable for Python documentation"
)]
#![allow(
    clippy::must_use_candidate,
    reason = "Not all functions need to be marked with must_use in Python bindings"
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "Python bindings may require specific parameter patterns"
)]

mod convert;
#[pymodule]
mod _core {
    #[pymodule_export]
    use super::{
        Dataset, DatasetManager, DatasetWriter, FriconDatasetError, ServerHandle, Trace, Workspace,
        main, main_gui, serve_workspace,
    };
}

use std::{
    env,
    io::{IsTerminal, stderr, stdout},
    mem,
    path::PathBuf,
    time::Duration,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use fricon::{
    Client, ClientError,
    app::AppManager,
    dataset::{
        model::{DatasetMetadata, DatasetRecord, DatasetStatus},
        schema::{DatasetScalar, FixedStepTrace, VariableStepTrace},
    },
};
use fricon_cli::clap::{Parser, error::ErrorKind};
use indexmap::IndexMap;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyRuntimeError},
    prelude::*,
    sync::PyOnceLock,
    types::{PyDict, PyList},
};
use pyo3_async_runtimes::tokio::get_runtime;

create_exception!(fricon._core, FriconDatasetError, PyException);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PythonDatasetErrorCode {
    DatasetNotFound,
    DatasetDeleted,
    DatasetNotTrashed,
    InvalidTag,
    SameTagName,
    SameSourceTarget,
    Internal,
}

impl PythonDatasetErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::DatasetNotFound => "dataset_not_found",
            Self::DatasetDeleted => "dataset_deleted",
            Self::DatasetNotTrashed => "dataset_not_trashed",
            Self::InvalidTag => "invalid_tag",
            Self::SameTagName => "same_tag_name",
            Self::SameSourceTarget => "same_source_target",
            Self::Internal => "internal",
        }
    }
}

fn generic_py_err(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn dataset_py_err(code: PythonDatasetErrorCode, message: impl Into<String>) -> PyErr {
    let message = message.into();
    Python::attach(|py| {
        let err = FriconDatasetError::new_err((message.clone(),));
        let value = err.value(py);
        let _ = value.setattr("code", code.as_str());
        let _ = value.setattr("message", message);
        err
    })
}

fn client_error_details(error: &ClientError) -> Option<(PythonDatasetErrorCode, String)> {
    match error {
        ClientError::DatasetNotFound => Some((
            PythonDatasetErrorCode::DatasetNotFound,
            "Dataset not found".to_string(),
        )),
        ClientError::DatasetDeleted => Some((
            PythonDatasetErrorCode::DatasetDeleted,
            "Dataset has been permanently deleted".to_string(),
        )),
        ClientError::DatasetNotTrashed => Some((
            PythonDatasetErrorCode::DatasetNotTrashed,
            "Dataset must be moved to trash before permanent deletion".to_string(),
        )),
        ClientError::InvalidTag => Some((
            PythonDatasetErrorCode::InvalidTag,
            "Tag name must not be empty".to_string(),
        )),
        ClientError::SameTagName => Some((
            PythonDatasetErrorCode::SameTagName,
            "Old tag name and new tag name must differ".to_string(),
        )),
        ClientError::SameSourceTarget => Some((
            PythonDatasetErrorCode::SameSourceTarget,
            "Source tag and target tag must differ".to_string(),
        )),
        ClientError::DatasetOperationFailed => Some((
            PythonDatasetErrorCode::Internal,
            "Dataset operation failed".to_string(),
        )),
        _ => None,
    }
}

fn map_client_error(error: ClientError) -> PyErr {
    match client_error_details(&error) {
        Some((code, message)) => dataset_py_err(code, message),
        None => generic_py_err(error),
    }
}

/// A client of fricon workspace server.
#[pyclass(module = "fricon._core", from_py_object)]
#[derive(Clone)]
pub struct Workspace {
    client: Client,
}

#[pymethods]
impl Workspace {
    /// Connect to a fricon server.
    ///
    /// Parameters:
    ///     path: The path to the workspace.
    ///
    /// Returns:
    ///     A workspace client.
    #[staticmethod]
    pub fn connect(py: Python<'_>, path: PathBuf) -> Result<Self> {
        let client = py.detach(|| get_runtime().block_on(Client::connect(&path)))?;
        Ok(Self { client })
    }

    /// A dataset manager for this workspace.
    #[getter]
    pub fn dataset_manager(&self) -> DatasetManager {
        DatasetManager {
            workspace: self.clone(),
        }
    }
}

/// Manager of datasets in workspace.
#[pyclass(module = "fricon._core", from_py_object)]
#[derive(Clone)]
pub struct DatasetManager {
    workspace: Workspace,
}

#[pymethods]
impl DatasetManager {
    /// Create a new dataset.
    ///
    /// Parameters:
    ///     name: Name of the dataset.
    ///     description: Description of the dataset.
    ///     tags: Tags of the dataset. Duplicate tags will be added only once.
    ///
    /// Returns:
    ///     A writer of the newly created dataset.
    #[pyo3(signature = (name, *, description=None, tags=None))]
    pub fn create(
        &self,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<DatasetWriter> {
        let description = description.unwrap_or_default();
        let tags = tags.unwrap_or_default();

        Ok(DatasetWriter::new(
            self.workspace.client.clone(),
            name,
            description,
            tags,
        ))
    }

    /// Open a dataset by id.
    ///
    /// Parameters:
    ///     dataset_id: An integer `id` or UUID `uid`
    ///
    /// Returns:
    ///     The requested dataset.
    ///
    /// Raises:
    ///     FriconDatasetError: Dataset not found or otherwise unavailable.
    pub fn open(&self, py: Python<'_>, dataset_id: &Bound<'_, PyAny>) -> PyResult<Dataset> {
        let client = self.workspace.client.clone();
        if let Ok(id) = dataset_id.extract::<i32>() {
            let inner = py
                .detach(|| get_runtime().block_on(client.get_dataset_by_id(id)))
                .map_err(map_client_error)?;
            Ok(Dataset { inner })
        } else if let Ok(uid) = dataset_id.extract::<String>() {
            let inner = py
                .detach(|| get_runtime().block_on(client.get_dataset_by_uid(uid)))
                .map_err(map_client_error)?;
            Ok(Dataset { inner })
        } else {
            Err(generic_py_err("Invalid dataset id."))
        }
    }

    /// List datasets in the workspace.
    ///
    /// Parameters:
    ///     limit: Optional max number of datasets to return.
    ///     offset: Optional offset for pagination.
    ///
    /// Returns:
    ///     A pandas dataframe containing information of datasets.
    #[pyo3(signature = (*, limit = None, offset = None))]
    pub fn list_all(
        &self,
        py: Python<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> PyResult<Py<PyAny>> {
        static FROM_RECORDS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        let client = self.workspace.client.clone();
        let records = py
            .detach(|| get_runtime().block_on(client.list_all_datasets(limit, offset)))
            .map_err(map_client_error)?;
        let py_records = records.into_iter().map(
            |DatasetRecord {
                 id,
                 metadata:
                     DatasetMetadata {
                         uid,
                         name,
                         description,
                         favorite,
                         created_at,
                         deleted_at,
                         tags,
                         ..
                     },
                 ..
             }| {
                let uid = uid.simple().to_string();
                (
                    id,
                    uid,
                    name,
                    description,
                    favorite,
                    created_at,
                    deleted_at,
                    tags,
                )
            },
        );
        let py_records = PyList::new(py, py_records)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("index", "id")?;
        kwargs.set_item(
            "columns",
            [
                "id",
                "uid",
                "name",
                "description",
                "favorite",
                "created_at",
                "deleted_at",
                "tags",
            ],
        )?;
        FROM_RECORDS
            .get_or_try_init(py, || {
                Ok::<_, PyErr>(
                    py.import("pandas")?
                        .getattr("DataFrame")?
                        .getattr("from_records")?
                        .unbind(),
                )
            })?
            .call(py, (py_records,), Some(&kwargs))
    }
}

/// 1-D list of values with optional x-axis values.
#[pyclass(module = "fricon._core", from_py_object)]
#[derive(Debug, Clone)]
pub struct Trace(DatasetScalar);

#[pymethods]
impl Trace {
    /// Create a new trace with variable x steps.
    ///
    /// Parameters:
    ///     x: List of x-axis values.
    ///     y: List of y-axis values.
    ///
    /// Returns:
    ///     A variable-step trace.
    #[staticmethod]
    pub fn variable_step(x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>) -> Result<Self> {
        let x = convert::extract_float_array(x)?;
        let y = convert::extract_scalar_array(y)?;
        let inner = VariableStepTrace::new(x, y)?.into();
        Ok(Self(inner))
    }

    /// Create a new trace with fixed x steps.
    ///
    /// Parameters:
    ///     x0: Starting x-axis value.
    ///     step: Step size of x-axis values.
    ///     y: List of y-axis values.
    ///
    /// Returns:
    ///     A fixed-step trace.
    #[staticmethod]
    pub fn fixed_step(x0: f64, step: f64, y: &Bound<'_, PyAny>) -> Result<Self> {
        let y = convert::extract_scalar_array(y)?;
        let inner = FixedStepTrace::new(x0, step, y).into();
        Ok(Self(inner))
    }
}

/// A dataset.
///
/// Datasets can be created and opened using the
/// [`DatasetManager`][fricon.DatasetManager].
#[pyclass(module = "fricon._core")]
pub struct Dataset {
    inner: fricon::Dataset,
}

fn helper_module(py: Python<'_>) -> PyResult<&Py<PyAny>> {
    static IO_MODULE: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
    IO_MODULE.get_or_try_init(py, || py.import("fricon._helper").map(Into::into))
}

#[pymethods]
impl Dataset {
    /// Load the dataset as a polars LazyFrame.
    ///
    /// Returns:
    ///     A polars LazyFrame.
    pub fn to_polars(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        // Pass dataset directory; helper will gather chunk files.
        helper_module(py)?.call_method1(py, "read_polars", (self.inner.path(),))
    }

    /// Load the dataset as an Arrow Table.
    ///
    /// Returns:
    ///     An Arrow Table.
    pub fn to_arrow(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        helper_module(py)?.call_method1(py, "read_arrow", (self.inner.path(),))
    }

    #[pyo3(signature = (*tag))]
    pub fn add_tags(&mut self, py: Python<'_>, tag: Vec<String>) -> PyResult<()> {
        py.detach(|| get_runtime().block_on(self.inner.add_tags(tag)))
            .map_err(map_client_error)?;
        Ok(())
    }

    #[pyo3(signature = (*tag))]
    pub fn remove_tags(&mut self, py: Python<'_>, tag: Vec<String>) -> PyResult<()> {
        py.detach(|| get_runtime().block_on(self.inner.remove_tags(tag)))
            .map_err(map_client_error)?;
        Ok(())
    }

    #[pyo3(signature = (*, name = None, description = None, favorite = None))]
    pub fn update_metadata(
        &mut self,
        py: Python<'_>,
        name: Option<String>,
        description: Option<String>,
        favorite: Option<bool>,
    ) -> PyResult<()> {
        py.detach(|| {
            get_runtime().block_on(self.inner.update_metadata(name, description, favorite))
        })
        .map_err(map_client_error)?;
        Ok(())
    }

    /// Name of the dataset.
    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Description of the dataset.
    #[getter]
    pub fn description(&self) -> &str {
        self.inner.description()
    }

    /// Favorite status of the dataset.
    #[getter]
    pub const fn favorite(&self) -> bool {
        self.inner.favorite()
    }

    /// Tags of the dataset.
    #[getter]
    pub fn tags(&self) -> &[String] {
        self.inner.tags()
    }

    /// ID of the dataset.
    #[getter]
    pub const fn id(&self) -> i32 {
        self.inner.id()
    }

    /// UID of the dataset.
    #[getter]
    pub fn uid(&self) -> String {
        self.inner.uid().simple().to_string()
    }

    /// Path of the dataset.
    #[getter]
    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }

    /// Creation date of the dataset.
    #[getter]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.inner.created_at()
    }

    /// Permanent deletion timestamp of the dataset payload, if any.
    #[getter]
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.inner.deleted_at()
    }

    /// Whether the dataset payload has been permanently deleted.
    #[getter]
    pub const fn is_deleted(&self) -> bool {
        self.inner.is_deleted()
    }

    /// Status of the dataset.
    #[getter]
    pub fn status(&self) -> String {
        match self.inner.status() {
            DatasetStatus::Writing => "writing".to_string(),
            DatasetStatus::Completed => "completed".to_string(),
            DatasetStatus::Aborted => "aborted".to_string(),
        }
    }
}

/// A handle to manage the lifecycle of a fricon server.
///
/// This handle keeps the server alive and allows for graceful shutdown.
/// When this handle is dropped, the server will be automatically shut down.
#[pyclass(module = "fricon._core")]
pub struct ServerHandle {
    manager: Option<AppManager>,
}

#[pymethods]
impl ServerHandle {
    /// Shutdown the server gracefully.
    ///
    /// This will stop the server and release all resources.
    /// After calling this method, the handle cannot be used again.
    ///
    /// Parameters:
    ///     timeout: Optional timeout in seconds. Defaults to 10 seconds.
    #[pyo3(signature = (timeout = None))]
    pub fn shutdown(&mut self, py: Python<'_>, timeout: Option<f64>) {
        if let Some(manager) = self.manager.take() {
            let timeout_duration = match timeout {
                Some(secs) => Duration::from_secs_f64(secs),
                None => Duration::from_secs(10),
            };
            py.detach(|| get_runtime().block_on(manager.shutdown_with_timeout(timeout_duration)));
        }
    }

    /// Check if the server is still running.
    ///
    /// Returns:
    ///     True if the server is running, False if it has been shut down.
    #[getter]
    pub fn is_running(&self) -> bool {
        self.manager.is_some()
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        let Some(manager) = self.manager.take() else {
            return;
        };
        let mut manager = Some(manager);
        let detached = Python::try_attach(|py| {
            if let Some(manager) = manager.take() {
                py.detach(|| {
                    get_runtime().block_on(manager.shutdown_with_timeout(Duration::from_secs(5)));
                });
            }
        })
        .is_some();
        if !detached && let Some(manager) = manager.take() {
            let _shutdown_task = get_runtime().spawn(async move {
                manager.shutdown_with_timeout(Duration::from_secs(5)).await;
            });
        }
    }
}

enum WriterState {
    NotStarted {
        client: Client,
        name: String,
        description: String,
        tags: Vec<String>,
    },
    Writing(fricon::DatasetWriter),
    Finished,
}

/// Writer for newly created dataset.
///
/// Writers are constructed by calling
/// [`DatasetManager.create`][fricon.DatasetManager.create].
#[pyclass(module = "fricon._core")]
pub struct DatasetWriter {
    state: WriterState,
    dataset: Option<Py<Dataset>>,
}

impl DatasetWriter {
    const fn new(client: Client, name: String, description: String, tags: Vec<String>) -> Self {
        Self {
            state: WriterState::NotStarted {
                client,
                name,
                description,
                tags,
            },
            dataset: None,
        }
    }

    fn complete(&mut self, py: Python<'_>, abort: bool) -> PyResult<Py<Dataset>> {
        if let Some(dataset) = self.dataset.as_ref() {
            return Ok(dataset.clone_ref(py));
        }

        match mem::replace(&mut self.state, WriterState::Finished) {
            WriterState::Writing(writer) => {
                let inner = if abort {
                    py.detach(|| get_runtime().block_on(writer.abort()))
                        .map_err(map_client_error)?
                } else {
                    py.detach(|| get_runtime().block_on(writer.finish()))
                        .map_err(map_client_error)?
                };
                let dataset = Py::new(py, Dataset { inner })?;
                self.dataset = Some(dataset.clone_ref(py));
                Ok(dataset)
            }
            WriterState::NotStarted {
                client,
                name,
                description,
                tags,
            } => {
                self.state = WriterState::NotStarted {
                    client,
                    name,
                    description,
                    tags,
                };
                Err(generic_py_err("No data to finalize."))
            }
            WriterState::Finished => Err(generic_py_err("Writer closed.")),
        }
    }
}

#[pymethods]
impl DatasetWriter {
    /// Write a row of values to the dataset.
    ///
    /// Parameters:
    ///     kwargs: Names and values in the row.
    #[pyo3(signature = (**kwargs))]
    pub fn write(
        &mut self,
        py: Python<'_>,
        kwargs: Option<IndexMap<String, Py<PyAny>>>,
    ) -> PyResult<()> {
        let Some(values) = kwargs else {
            return Err(generic_py_err("No data to write."));
        };
        self.write_dict(py, values)
    }

    /// Write a row of values to the dataset.
    ///
    /// Parameters:
    ///     values: A dictionary of names and values in the row.
    pub fn write_dict(
        &mut self,
        py: Python<'_>,
        values: IndexMap<String, Py<PyAny>>,
    ) -> PyResult<()> {
        if values.is_empty() {
            return Err(generic_py_err("No data to write."));
        }

        match mem::replace(&mut self.state, WriterState::Finished) {
            WriterState::NotStarted {
                client,
                name,
                description,
                tags,
            } => {
                let row = convert::build_row(py, values)?;
                let schema = row.to_schema();
                let writer = py
                    .detach(|| -> std::result::Result<_, ClientError> {
                        let mut writer = get_runtime().block_on(client.create_dataset(
                            name,
                            description,
                            tags,
                            schema,
                        ))?;
                        get_runtime().block_on(writer.write(row))?;
                        Ok(writer)
                    })
                    .map_err(map_client_error)?;
                self.state = WriterState::Writing(writer);
            }
            WriterState::Writing(mut writer) => {
                let row = convert::build_row(py, values)?;
                writer = py
                    .detach(|| -> std::result::Result<_, ClientError> {
                        get_runtime().block_on(writer.write(row))?;
                        Ok(writer)
                    })
                    .map_err(map_client_error)?;
                self.state = WriterState::Writing(writer);
            }
            WriterState::Finished => {
                return Err(generic_py_err("Writer closed."));
            }
        }

        Ok(())
    }

    /// ID of the dataset.
    ///
    /// Raises:
    ///     RuntimeError: Writer is not closed yet.
    #[getter]
    pub fn dataset(&self, py: Python<'_>) -> PyResult<Py<Dataset>> {
        let dataset = self
            .dataset
            .as_ref()
            .ok_or_else(|| generic_py_err("Writer is not closed yet."))?
            .clone_ref(py);
        Ok(dataset)
    }

    /// Finish writing to dataset and return dataset metadata.
    pub fn finish(&mut self, py: Python<'_>) -> PyResult<Py<Dataset>> {
        self.complete(py, false)
    }

    /// Abort writing to dataset and return dataset metadata.
    pub fn abort(&mut self, py: Python<'_>) -> PyResult<Py<Dataset>> {
        self.complete(py, true)
    }

    /// Finish writing to dataset.
    pub fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        if self.dataset.is_some() {
            return Ok(());
        }

        if matches!(self.state, WriterState::Writing(_)) {
            let _ = self.finish(py)?;
        } else {
            self.state = WriterState::Finished;
        }
        Ok(())
    }

    /// Enter context manager.
    pub const fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Exit context manager and close the writer.
    ///
    /// Will call [`close`][fricon.DatasetWriter.close] method.
    pub fn __exit__(
        &mut self,
        py: Python<'_>,
        exc_type: Py<PyAny>,
        _exc_value: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> PyResult<()> {
        if exc_type.is_none(py) {
            self.close(py)
        } else {
            let _ = self.abort(py);
            Ok(())
        }
    }
}

fn ignore_python_sigint(py: Python<'_>) -> PyResult<()> {
    let signal = py.import("signal")?;
    let sigint = signal.getattr("SIGINT")?;
    let default_handler = signal.getattr("SIG_DFL")?;
    _ = signal.call_method1("signal", (sigint, default_handler))?;
    Ok(())
}

fn command_name_from_argv0(argv0: &std::ffi::OsStr) -> String {
    std::path::Path::new(argv0)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| "fricon".to_string(), ToString::to_string)
}

fn has_console_output() -> bool {
    stdout().is_terminal() || stderr().is_terminal()
}

fn parse_error_exit_code(kind: ErrorKind) -> i32 {
    match kind {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
        _ => 2,
    }
}

#[expect(clippy::print_stderr, reason = "Error messages for CLI tool")]
pub fn main_impl<T: Parser + fricon_cli::Main>(py: Python<'_>) -> i32 {
    if ignore_python_sigint(py).is_err() {
        eprintln!("Failed to reset python SIGINT handler.");
        return 1;
    }

    // Skip python executable
    let argv = env::args_os().skip(1);
    let cli = T::parse_from(argv);
    match cli.main() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");
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
#[expect(clippy::print_stderr, reason = "Error messages for CLI tool")]
pub fn main_gui(py: Python<'_>) -> i32 {
    if ignore_python_sigint(py).is_err() {
        eprintln!("Failed to reset python SIGINT handler.");
        return 1;
    }

    // Skip python executable
    let argv: Vec<_> = env::args_os().skip(1).collect();
    let command_name = argv.first().map_or_else(
        || "fricon-gui".to_string(),
        |arg| command_name_from_argv0(arg),
    );
    let cli_help = match fricon_cli::render_help_for_command::<fricon_cli::Gui>(&command_name) {
        Ok(help) => help,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };
    match fricon_cli::Gui::try_parse_from(argv) {
        Ok(cli) => match cli.main_with_help(command_name, cli_help) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Error: {e}");
                1
            }
        },
        Err(parse_error) => {
            if has_console_output() {
                let exit_code = parse_error_exit_code(parse_error.kind());
                eprint!("{parse_error}");
                exit_code
            } else {
                match fricon_cli::launch_gui_with_context(command_name, cli_help, None, false) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        1
                    }
                }
            }
        }
    }
}

/// Create a workspace and start a server for it.
///
/// This function creates a new workspace at the given path and starts a server.
/// The server will run in the background and the workspace client is returned.
///
/// Note: The server runs in the background. When the workspace client is
/// dropped, the connection is closed but the server continues running. For
/// proper cleanup, you may need to manually stop the server process.
///
/// Parameters:
///     path: The path where to create the workspace.
///
/// Returns:
///     A tuple of `(workspace_client, server_handle)`.
#[pyfunction]
pub fn serve_workspace(py: Python<'_>, path: PathBuf) -> Result<(Workspace, ServerHandle)> {
    let runtime = get_runtime();

    // Create the workspace first
    let root = fricon::WorkspaceRoot::create_new(&path)?;

    // Start the server in the background and keep the manager
    let manager = AppManager::new(root)?.start(runtime.handle())?;

    // Connect to the workspace
    let workspace = Workspace::connect(py, path.clone())?;
    let server_handle = ServerHandle {
        manager: Some(manager),
    };
    Ok((workspace, server_handle))
}

#[cfg(test)]
mod tests {
    use fricon::ClientError;
    use fricon_cli::clap::error::ErrorKind;

    use super::{PythonDatasetErrorCode, client_error_details, parse_error_exit_code};

    #[test]
    fn parse_help_and_version_return_success_exit_code() {
        assert_eq!(parse_error_exit_code(ErrorKind::DisplayHelp), 0);
        assert_eq!(parse_error_exit_code(ErrorKind::DisplayVersion), 0);
    }

    #[test]
    fn parse_failure_returns_error_exit_code() {
        assert_eq!(parse_error_exit_code(ErrorKind::MissingRequiredArgument), 2);
    }

    #[test]
    fn client_not_found_maps_to_python_dataset_error() {
        let Some((code, message)) = client_error_details(&ClientError::DatasetNotFound) else {
            panic!("expected dataset semantic mapping");
        };
        assert_eq!(code, PythonDatasetErrorCode::DatasetNotFound);
        assert_eq!(message, "Dataset not found");
    }

    #[test]
    fn client_deleted_maps_to_python_dataset_error() {
        let Some((code, _)) = client_error_details(&ClientError::DatasetDeleted) else {
            panic!("expected dataset semantic mapping");
        };
        assert_eq!(code, PythonDatasetErrorCode::DatasetDeleted);
    }

    #[test]
    fn unknown_errors_stay_on_generic_runtime_error_path() {
        assert!(client_error_details(&ClientError::NotRunning).is_none());
    }
}

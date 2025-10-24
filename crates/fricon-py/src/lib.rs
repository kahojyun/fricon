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
        Dataset, DatasetManager, DatasetWriter, ServerHandle, Trace, Workspace, main, main_gui,
        serve_workspace,
    };
}

use std::{env, mem, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use fricon::{
    Client, DatasetMetadata, DatasetRecord, DatasetScalar, FixedStepTrace, VariableStepTrace,
};
use fricon_cli::clap::Parser;
use indexmap::IndexMap;
use pyo3::{
    prelude::*,
    sync::PyOnceLock,
    types::{PyDict, PyList},
};
use pyo3_async_runtimes::tokio::get_runtime;

/// A client of fricon workspace server.
#[pyclass(module = "fricon._core")]
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
    pub fn connect(path: PathBuf) -> Result<Self> {
        let client = get_runtime().block_on(Client::connect(&path))?;
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
#[pyclass(module = "fricon._core")]
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
    ///     dataset_id: An integer `id` or UUID `uuid`
    ///
    /// Returns:
    ///     The requested dataset.
    ///
    /// Raises:
    ///     RuntimeError: Dataset not found.
    pub fn open(&self, dataset_id: &Bound<'_, PyAny>) -> Result<Dataset> {
        if let Ok(id) = dataset_id.extract::<i32>() {
            let inner = get_runtime().block_on(self.workspace.client.get_dataset_by_id(id))?;
            Ok(Dataset { inner })
        } else if let Ok(uuid) = dataset_id.extract::<String>() {
            let inner = get_runtime().block_on(self.workspace.client.get_dataset_by_uuid(uuid))?;
            Ok(Dataset { inner })
        } else {
            bail!("Invalid dataset id.")
        }
    }

    /// List all datasets in the workspace.
    ///
    /// Returns:
    ///     A pandas dataframe containing information of all datasets.
    pub fn list_all(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        static FROM_RECORDS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        let records = get_runtime().block_on(self.workspace.client.list_all_datasets())?;
        let py_records = records.into_iter().map(
            |DatasetRecord {
                 id,
                 metadata:
                     DatasetMetadata {
                         uuid,
                         name,
                         description,
                         favorite,
                         created_at,
                         tags,
                         ..
                     },
                 ..
             }| {
                let uuid = uuid.simple().to_string();
                (id, uuid, name, description, favorite, created_at, tags)
            },
        );
        let py_records = PyList::new(py, py_records)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("index", "id")?;
        kwargs.set_item(
            "columns",
            [
                "id",
                "uuid",
                "name",
                "description",
                "favorite",
                "created_at",
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
#[pyclass(module = "fricon._core")]
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
    pub fn add_tags(&mut self, tag: Vec<String>) -> Result<()> {
        get_runtime().block_on(self.inner.add_tags(tag))
    }

    #[pyo3(signature = (*tag))]
    pub fn remove_tags(&mut self, tag: Vec<String>) -> Result<()> {
        get_runtime().block_on(self.inner.remove_tags(tag))
    }

    #[pyo3(signature = (*, name = None, description = None, favorite = None))]
    pub fn update_metadata(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        favorite: Option<bool>,
    ) -> Result<()> {
        get_runtime().block_on(self.inner.update_metadata(name, description, favorite))
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

    /// UUID of the dataset.
    #[getter]
    pub fn uuid(&self) -> String {
        self.inner.uuid().simple().to_string()
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

    /// Status of the dataset.
    #[getter]
    pub fn status(&self) -> String {
        match self.inner.status() {
            fricon::DatasetStatus::Writing => "writing".to_string(),
            fricon::DatasetStatus::Completed => "completed".to_string(),
            fricon::DatasetStatus::Aborted => "aborted".to_string(),
        }
    }
}

/// A handle to manage the lifecycle of a fricon server.
///
/// This handle keeps the server alive and allows for graceful shutdown.
/// When this handle is dropped, the server will be automatically shut down.
#[pyclass(module = "fricon._core")]
pub struct ServerHandle {
    manager: Option<fricon::AppManager>,
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
    pub fn shutdown(&mut self, _py: Python<'_>, timeout: Option<f64>) {
        if let Some(manager) = self.manager.take() {
            let timeout_duration = match timeout {
                Some(secs) => Duration::from_secs_f64(secs),
                None => Duration::from_secs(10),
            };
            get_runtime().block_on(manager.shutdown_with_timeout(timeout_duration));
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
        if let Some(manager) = self.manager.take() {
            get_runtime().block_on(manager.shutdown_with_timeout(Duration::from_secs(5)));
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
    ) -> Result<()> {
        let Some(values) = kwargs else {
            bail!("No data to write.")
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
    ) -> Result<()> {
        if values.is_empty() {
            bail!("No data to write.")
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
                let _guard = get_runtime().enter();
                let mut writer = client.create_dataset(name, description, tags, schema)?;
                get_runtime().block_on(writer.write(row))?;
                self.state = WriterState::Writing(writer);
            }
            WriterState::Writing(mut writer) => {
                let row = convert::build_row(py, values)?;
                get_runtime().block_on(writer.write(row))?;
                self.state = WriterState::Writing(writer);
            }
            WriterState::Finished => {
                bail!("Writer closed.")
            }
        }

        Ok(())
    }

    /// ID of the dataset.
    ///
    /// Raises:
    ///     RuntimeError: Writer is not closed yet.
    #[getter]
    pub fn dataset(&self, py: Python<'_>) -> Result<Py<Dataset>> {
        let dataset = self
            .dataset
            .as_ref()
            .context("Writer is not closed yet.")?
            .clone_ref(py);
        Ok(dataset)
    }

    /// Finish writing to dataset.
    pub fn close(&mut self, py: Python<'_>) -> Result<()> {
        if let WriterState::Writing(writer) = mem::replace(&mut self.state, WriterState::Finished) {
            let inner = get_runtime().block_on(writer.finish())?;
            self.dataset = Some(Py::new(py, Dataset { inner })?);
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
        _exc_type: Py<PyAny>,
        _exc_value: Py<PyAny>,
        _traceback: Py<PyAny>,
    ) -> Result<()> {
        self.close(py)
    }
}

#[expect(clippy::print_stderr, reason = "Error messages for CLI tool")]
pub fn main_impl<T: Parser + fricon_cli::Main>(py: Python<'_>) -> i32 {
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
    let argv = env::args_os().skip(1);
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
/// Note: The server runs in the background. When the workspace client is
/// dropped, the connection is closed but the server continues running. For
/// proper cleanup, you may need to manually stop the server process.
///
/// Parameters:
///     path: The path where to create the workspace.
///
/// Returns:
///     A tuple containing (workspace_client, server_handle) where:
///     - workspace_client: A workspace client connected to the newly created
///       workspace
///     - server_handle: A handle to manage the server lifecycle
#[pyfunction]
pub fn serve_workspace(path: PathBuf) -> Result<(Workspace, ServerHandle)> {
    let runtime = get_runtime();

    // Create the workspace first
    let root = fricon::WorkspaceRoot::create_new(&path)?;

    // Start the server in the background and keep the manager
    let manager = runtime.block_on(fricon::AppManager::serve(root))?;

    // Connect to the workspace
    let workspace = Workspace::connect(path.clone())?;
    let server_handle = ServerHandle {
        manager: Some(manager),
    };
    Ok((workspace, server_handle))
}

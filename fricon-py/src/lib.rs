#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use fricon::cli::Cli;
use pyo3::prelude::*;

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use super::{
        complex128, main, trace_, Dataset, DatasetManager, DatasetWriter, Trace, Workspace,
    };
}

/// A client of fricon workspace server.
#[pyclass(module = "fricon._core")]
pub struct Workspace;

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
    pub fn connect(path: PathBuf) -> Self {
        todo!()
    }

    /// A dataset manager for this workspace.
    #[getter]
    pub fn dataset_manager(&self) -> DatasetManager {
        todo!()
    }

    /// Close connection to server.
    pub fn close(&self) {
        todo!()
    }

    /// Enter context manager.
    pub fn __enter__(&self) -> PyObject {
        todo!()
    }

    /// Exit context manager and close connection.
    ///
    /// Will call [`close`][fricon.Workspace.close] method.
    pub fn __exit__(&self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        todo!()
    }
}

/// Manager of datasets in workspace.
#[pyclass(module = "fricon._core")]
pub struct DatasetManager;

#[pymethods]
impl DatasetManager {
    /// Create a new dataset.
    ///
    /// Parameters:
    ///     name: Name of the dataset.
    ///     description: Description of the dataset.
    ///     tags: Tags of the dataset. Duplicate tags will be add only once.
    ///     schema: Schema of the underlying arrow table. Can be only a subset of all columns,
    ///         other fields will be inferred from first row.
    ///     index: Names of index columns.
    ///
    /// Returns:
    ///     A writer of the newly created dataset.
    #[pyo3(signature = (name, *, description=None, tags=None, schema=None, index=None))]
    pub fn create(
        &self,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
        schema: Option<PyObject>,
        index: Option<PyObject>,
    ) -> DatasetWriter {
        todo!()
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
    ///     RuntimeError: Dataset not found.
    pub fn open(&self, dataset_id: PyObject) -> PyResult<Dataset> {
        todo!()
    }

    /// List all datasets in the workspace.
    ///
    /// Returns:
    ///     A pandas dataframe containing information of all datasets.
    pub fn list_all(&self) -> PyObject {
        todo!()
    }
}

/// 1-D list of values with optional x-axis values.
#[pyclass(module = "fricon._core")]
pub struct Trace;

#[pymethods]
impl Trace {
    /// Create a new trace with variable x steps.
    ///
    /// Parameters:
    ///     xs: List of x-axis values.
    ///     ys: List of y-axis values.
    ///
    /// Returns:
    ///     A variable-step trace.
    #[staticmethod]
    pub fn variable_step(xs: PyObject, ys: PyObject) -> Self {
        todo!();
    }

    /// Create a new trace with fixed x steps.
    ///
    /// Parameters:
    ///     x0: Starting x-axis value.
    ///     dx: Step size of x-axis values.
    ///     ys: List of y-axis values.
    ///
    /// Returns:
    ///     A fixed-step trace.
    #[staticmethod]
    pub fn fixed_step(x0: f64, dx: f64, ys: PyObject) -> Self {
        todo!();
    }
}

/// A dataset.
///
/// Datasets can be created and opened using the [`DatasetManager`][fricon.DatasetManager].
#[pyclass(module = "fricon._core")]
pub struct Dataset;

#[pymethods]
impl Dataset {
    /// Name of the dataset.
    #[getter]
    pub fn name(&self) -> &str {
        todo!();
    }

    #[setter]
    pub fn set_name(&mut self, name: &str) {
        todo!();
    }

    /// Description of the dataset.
    #[getter]
    pub fn description(&self) -> &str {
        todo!();
    }

    #[setter]
    pub fn set_description(&mut self, description: &str) {
        todo!();
    }

    /// Tags of the dataset.
    #[getter]
    pub fn tags(&self) -> Vec<String> {
        todo!();
    }

    #[setter]
    pub fn set_tags(&mut self, tags: Vec<String>) {
        todo!();
    }

    /// Favorite status of the dataset.
    #[getter]
    pub fn favorite(&self) -> bool {
        todo!();
    }

    #[setter]
    pub fn set_favorite(&mut self, favorite: bool) {
        todo!();
    }

    /// Load the dataset as a pandas DataFrame.
    ///
    /// Arrow data types cannot be directly converted to pandas data types, so in some cases the
    /// conversion may be slow or fail. Consider using `to_polars` or `to_arrow` for better
    /// performance.
    ///
    /// See Also:
    ///     [`to_polars`][fricon.Dataset.to_polars], [`to_arrow`][fricon.Dataset.to_arrow]
    ///
    /// Returns:
    ///     A pandas DataFrame.
    pub fn to_pandas(&self) -> PyObject {
        todo!();
    }

    /// Load the dataset as a polars DataFrame.
    ///
    /// `polars` supports memory mapping, so it is faster than `pandas` for large datasets.
    ///
    /// See Also:
    ///     [`to_pandas`][fricon.Dataset.to_pandas], [`to_arrow`][fricon.Dataset.to_arrow]
    ///
    /// Returns:
    ///     A polars DataFrame.
    pub fn to_polars(&self) -> PyObject {
        todo!();
    }

    /// Load the dataset as an Arrow Table.
    ///
    /// See Also:
    ///     [`to_pandas`][fricon.Dataset.to_pandas], [`to_polars`][fricon.Dataset.to_polars]
    ///
    /// Returns:
    ///     An Arrow Table.
    pub fn to_arrow(&self) -> PyObject {
        todo!();
    }

    /// Open a dataset.
    ///
    /// Parameters:
    ///     path: Path to the dataset.
    ///
    /// Returns:
    ///     Opened dataset.
    #[staticmethod]
    pub fn open(path: PathBuf) -> Self {
        todo!();
    }

    /// Id of the dataset.
    #[getter]
    pub fn id(&self) -> usize {
        todo!();
    }

    /// UUID of the dataset.
    #[getter]
    pub fn uid(&self) -> String {
        todo!();
    }

    /// Path of the dataset.
    #[getter]
    pub fn path(&self) -> &Path {
        todo!();
    }

    /// Creation date of the dataset.
    #[getter]
    pub fn created_at(&self) -> DateTime<Utc> {
        todo!();
    }

    /// Arrow schema of the dataset.
    #[getter]
    pub fn schema(&self) -> PyObject {
        todo!();
    }

    /// Index columns of the dataset.
    #[getter]
    pub fn index(&self) -> Vec<String> {
        todo!();
    }

    /// Close the dataset.
    pub fn close(&self) {
        todo!()
    }

    /// Enter context manager.
    pub fn __enter__(&self) -> PyObject {
        todo!()
    }

    /// Exit context manager and close the dataset.
    ///
    /// Will call [`close`][fricon.Dataset.close] method.
    pub fn __exit__(&self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        todo!()
    }
}

/// Writer for newly created dataset.
///
/// Writers are constructed by calling [`DatasetManager.create`][fricon.DatasetManager.create].
#[pyclass(module = "fricon._core")]
pub struct DatasetWriter;

#[pymethods]
impl DatasetWriter {
    /// Write a row of values to the dataset.
    ///
    /// Parameters:
    ///     kwargs: Names and values in the row.
    #[pyo3(signature = (**kwargs))]
    pub fn write(&self, kwargs: Option<PyObject>) {
        todo!()
    }

    /// Write a row of values to the dataset.
    ///
    /// Parameters:
    ///     values: A dictionary of names and values in the row.
    pub fn write_dict(&self, values: PyObject) {
        todo!()
    }

    /// Get the newly created dataset.
    ///
    /// Returns:
    ///     Dataset.
    ///
    /// Raises:
    ///     RuntimeError: Writer is not closed yet.
    pub fn to_dataset(&self) -> Dataset {
        todo!()
    }

    /// Finish writing to dataset.
    pub fn close(&self) {
        todo!()
    }

    /// Enter context manager.
    pub fn __enter__(&self) -> PyObject {
        todo!()
    }

    /// Exit context manager and close the writer.
    ///
    /// Will call [`close`][fricon.DatasetWriter.close] method.
    pub fn __exit__(&self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        todo!()
    }
}

/// Get a pyarrow data type representing 128 bit compelex number.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn complex128() -> PyObject {
    todo!();
}

/// Get a pyarrow data type representing [`Trace`][fricon.Trace].
///
/// Parameters:
///     item: Data type of the y values.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn trace_(item: PyObject) -> PyObject {
    todo!();
}

#[pyfunction]
#[must_use]
pub fn main(py: Python<'_>) -> i32 {
    fn inner(cli: Cli) -> Result<()> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async { fricon::main(cli).await })
    }
    fn ignore_python_sigint(py: Python<'_>) -> PyResult<()> {
        let signal = py.import_bound("signal")?;
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
    let cli = match Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            return e.exit_code();
        }
    };
    match inner(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e:?}");
            1
        }
    }
}

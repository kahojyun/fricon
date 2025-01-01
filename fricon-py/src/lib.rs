#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use arrow::{
    array::{
        make_array, Array, ArrayData, BooleanArray, Float64Array, Int64Array, RecordBatch,
        StringArray, StructArray,
    },
    datatypes::{DataType, Field, Fields, Schema},
    pyarrow::PyArrowType,
};
use chrono::{DateTime, Utc};
use clap::Parser;
use fricon::{
    cli::Cli,
    client::{self, Client},
    paths::WorkDirectory,
};
use num::complex::Complex64;
use pyo3::{
    prelude::*,
    types::{PyBool, PyComplex, PyDict, PyFloat, PyInt, PyString},
};
use pyo3_async_runtimes::tokio::get_runtime;

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use super::{
        complex128, main, trace_, Dataset, DatasetManager, DatasetWriter, Trace, Workspace,
    };
}

/// A client of fricon workspace server.
#[pyclass(module = "fricon._core")]
#[derive(Clone)]
pub struct Workspace {
    root: WorkDirectory,
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
    #[expect(clippy::needless_pass_by_value)]
    pub fn connect(path: PathBuf) -> Result<Self> {
        let root = WorkDirectory::new(&path)?;
        let ipc_file = root.ipc_file();
        let client = get_runtime().block_on(Client::connect(ipc_file))?;
        Ok(Self { root, client })
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
        schema: Option<PyArrowType<Schema>>,
        index: Option<Vec<String>>,
    ) -> Result<DatasetWriter> {
        let description = description.unwrap_or_default();
        let tags = tags.unwrap_or_default();
        let schema = schema.map(|s| s.0).unwrap_or_else(|| Schema::empty());
        let index = index.unwrap_or_default();
        let writer = get_runtime().block_on(self.workspace.client.create_dataset(
            name,
            description,
            tags,
            index,
        ))?;
        Ok(DatasetWriter::new(writer, Arc::new(schema)))
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
    pub fn open(&self, dataset_id: &Bound<'_, PyAny>) -> PyResult<Dataset> {
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

    /// Arrow data type of the trace.
    #[getter]
    pub fn data_type(&self) -> PyArrowType<DataType> {
        todo!();
    }

    /// Convert to an arrow array.
    ///
    /// Returns:
    ///     Arrow array.
    pub fn to_arrow_array(&self) -> PyArrowType<ArrayData> {
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
pub struct DatasetWriter {
    writer: Option<client::DatasetWriter>,
    id: Option<i64>,
    first_row: bool,
    schema: Arc<Schema>,
}

impl DatasetWriter {
    const fn new(writer: client::DatasetWriter, schema: Arc<Schema>) -> Self {
        Self {
            writer: Some(writer),
            id: None,
            first_row: true,
            schema,
        }
    }
}

fn infer_datatype(value: &Bound<'_, PyAny>) -> Result<DataType> {
    // Check bool first because bool is a subclass of int.
    if value.is_instance_of::<PyBool>() {
        Ok(DataType::Boolean)
    } else if value.is_instance_of::<PyInt>() {
        Ok(DataType::Int64)
    } else if value.is_instance_of::<PyFloat>() {
        Ok(DataType::Float64)
    } else if value.is_instance_of::<PyComplex>() {
        Ok(get_complex_type())
    } else if value.is_instance_of::<PyString>() {
        Ok(DataType::Utf8)
    } else if let Ok(trace) = value.downcast_exact::<Trace>() {
        Ok(trace.borrow().data_type().0)
    } else {
        bail!("Unsupported data type.");
    }
}

fn infer_schema(
    py: Python<'_>,
    initial_schema: &Schema,
    values: &HashMap<String, PyObject>,
) -> Result<Schema> {
    for field in initial_schema.fields() {
        if !values.contains_key(field.name()) {
            bail!("Missing field: {}", field.name());
        }
    }
    let mut new_fields = vec![];
    for (name, value) in values {
        if initial_schema.field_with_name(name).is_ok() {
            continue;
        }
        let value = value.bind(py);
        let datatype = infer_datatype(value)
            .with_context(|| format!("Failed to infer data type for '{name}'."))?;
        let field = Field::new(name, datatype, false);
        new_fields.push(field);
    }
    let new_schema = Schema::new(new_fields);
    let merged = Schema::try_merge([initial_schema.clone(), new_schema])?;
    Ok(merged)
}

fn build_list(field: &Arc<Field>, value: &Bound<'_, PyAny>) -> Result<Arc<dyn Array>> {
    todo!();
}

fn build_array(value: &Bound<'_, PyAny>, data_type: &DataType) -> Result<Arc<dyn Array>> {
    if let Ok(PyArrowType(data)) = value.extract::<PyArrowType<ArrayData>>() {
        ensure!(
            data.data_type() == data_type,
            "Different data type: schema: {data_type}, value: {}",
            data.data_type()
        );
        return Ok(make_array(data));
    }
    match data_type {
        DataType::Boolean => {
            let Ok(value) = value.extract::<bool>() else {
                bail!("Not a boolean value.")
            };
            let array = BooleanArray::new_scalar(value).into_inner();
            Ok(Arc::new(array))
        }
        DataType::Int64 => {
            let Ok(value) = value.extract::<i64>() else {
                bail!("Failed to extract int64 value.")
            };
            let array = Int64Array::new_scalar(value).into_inner();
            Ok(Arc::new(array))
        }
        DataType::Float64 => {
            let Ok(value) = value.extract::<f64>() else {
                bail!("Failed to extract float64 value.")
            };
            let array = Float64Array::new_scalar(value).into_inner();
            Ok(Arc::new(array))
        }
        DataType::Utf8 => {
            let Ok(value) = value.extract::<String>() else {
                bail!("Failed to extract float64 value.")
            };
            let array = StringArray::new_scalar(value).into_inner();
            Ok(Arc::new(array))
        }
        t @ DataType::Struct(fields) if *t == get_complex_type() => {
            let Ok(value) = value.extract::<Complex64>() else {
                bail!("Failed to extract complex value.")
            };
            let real = Float64Array::new_scalar(value.re).into_inner();
            let imag = Float64Array::new_scalar(value.im).into_inner();
            let array =
                StructArray::new(fields.clone(), vec![Arc::new(real), Arc::new(imag)], None);
            Ok(Arc::new(array))
        }
        t @ DataType::Struct(fields) => {
            let Ok(value) = value.downcast_exact::<Trace>() else {
                bail!("Failed to extract `Trace` value.")
            };
            let value = value.borrow();
            if *t != value.data_type().0 {
                bail!("Incompatible data type.")
            }
            let array = value.to_arrow_array().0;
            Ok(make_array(array))
        }
        DataType::List(field) => build_list(field, value),
        _ => {
            bail!("Unsupported data type {data_type}, please manually construct a `pyarrow.Array`.")
        }
    }
}

fn build_record_batch(
    py: Python<'_>,
    schema: Arc<Schema>,
    values: &HashMap<String, PyObject>,
) -> Result<RecordBatch> {
    ensure!(
        schema.fields().len() == values.len(),
        "Values not compatible with schema."
    );
    let mut columns = vec![];
    for field in schema.fields() {
        let name = field.name();
        let Some(value) = values.get(name) else {
            bail!("Missing value {name}")
        };
        columns.push(build_array(value.bind(py), field.data_type())?);
    }
    Ok(RecordBatch::try_new(schema, columns)?)
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
        kwargs: Option<HashMap<String, PyObject>>,
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
    #[expect(clippy::needless_pass_by_value)]
    pub fn write_dict(&mut self, py: Python<'_>, values: HashMap<String, PyObject>) -> Result<()> {
        if values.is_empty() {
            bail!("No data to write.")
        }
        let Some(writer) = &mut self.writer else {
            bail!("Writer closed.");
        };
        if self.first_row {
            self.schema = Arc::new(infer_schema(py, &self.schema, &values)?);
            self.first_row = false;
        }
        let batch = build_record_batch(py, self.schema.clone(), &values)?;
        writer.blocking_write(batch)?;
        Ok(())
    }

    /// Id of the dataset.
    ///
    /// Raises:
    ///     RuntimeError: Writer is not closed yet.
    #[getter]
    pub fn id(&self) -> Result<i64> {
        self.id.ok_or_else(|| anyhow!("Writer not closed."))
    }

    /// Finish writing to dataset.
    pub fn close(&mut self) -> Result<()> {
        let writer = self.writer.take();
        if let Some(writer) = writer {
            let id = get_runtime().block_on(writer.finish())?;
            self.id = Some(id);
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
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> Result<()> {
        self.close()
    }
}

fn get_complex_type() -> DataType {
    static COMPLEX: LazyLock<DataType> = LazyLock::new(|| {
        let fields = vec![
            Field::new("real", DataType::Float64, false),
            Field::new("imag", DataType::Float64, false),
        ];
        DataType::Struct(Fields::from(fields))
    });
    COMPLEX.clone()
}

fn get_trace_type(item: DataType, fixed_step: bool) -> DataType {
    let y_field = Field::new("ys", DataType::new_list(item, false), false);
    if fixed_step {
        let fields = vec![
            Field::new("x0", DataType::Float64, false),
            Field::new("dx", DataType::Float64, false),
            y_field,
        ];
        DataType::Struct(Fields::from(fields))
    } else {
        let x_field = Field::new("xs", DataType::new_list(DataType::Float64, false), false);
        let fields = vec![x_field, y_field];
        DataType::Struct(Fields::from(fields))
    }
}

/// Get a pyarrow data type representing 128 bit compelex number.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn complex128() -> PyArrowType<DataType> {
    PyArrowType(get_complex_type())
}

/// Get a pyarrow data type representing [`Trace`][fricon.Trace].
///
/// Parameters:
///     item: Data type of the y values.
///     fixed_step: Whether the trace has fixed x steps.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn trace_(item: PyArrowType<DataType>, fixed_step: bool) -> PyArrowType<DataType> {
    PyArrowType(get_trace_type(item.0, fixed_step))
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

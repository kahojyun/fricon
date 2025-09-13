use anyhow::{Context, Result, bail};
use arrow::datatypes::Schema;
use indexmap::IndexMap;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::get_runtime;

use crate::conversion::{build_record_batch, infer_dataset_schema_from_values};
use crate::dataset::Dataset;

/// Writer for newly created dataset.
///
/// Writers are constructed by calling [`DatasetManager.create`][fricon.DatasetManager.create].
#[pyclass(module = "fricon._core")]
pub struct DatasetWriter {
    writer: Option<fricon::DatasetWriter>,
    dataset: Option<Py<Dataset>>,
    first_row: bool,
    schema: std::sync::Arc<Schema>,
}

impl DatasetWriter {
    pub const fn new(writer: fricon::DatasetWriter, schema: std::sync::Arc<Schema>) -> Self {
        Self {
            writer: Some(writer),
            dataset: None,
            first_row: true,
            schema,
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
        kwargs: Option<IndexMap<String, PyObject>>,
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
    pub fn write_dict(&mut self, py: Python<'_>, values: IndexMap<String, PyObject>) -> Result<()> {
        if values.is_empty() {
            bail!("No data to write.")
        }
        let Some(writer) = &mut self.writer else {
            bail!("Writer closed.");
        };
        if self.first_row {
            // Infer fricon DatasetSchema first, then convert to Arrow
            self.schema = std::sync::Arc::new(infer_dataset_schema_from_values(py, &values)?);
            self.first_row = false;
        }
        let batch = build_record_batch(py, self.schema.clone(), &values)?;
        get_runtime().block_on(writer.write(batch))?;
        Ok(())
    }

    /// Id of the dataset.
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
        let writer = self.writer.take();
        if let Some(writer) = writer {
            let inner = get_runtime().block_on(writer.finish())?;
            self.dataset = Some(Py::new(py, Dataset::new(inner))?);
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
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> Result<()> {
        self.close(py)
    }
}

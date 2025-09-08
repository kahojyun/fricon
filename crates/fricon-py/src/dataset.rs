use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc};
use pyo3::{prelude::*, sync::GILOnceCell};
use pyo3_async_runtimes::tokio::get_runtime;

/// A dataset.
///
/// Datasets can be created and opened using the [`DatasetManager`][fricon.DatasetManager].
#[pyclass(module = "fricon._core")]
pub struct Dataset {
    pub(crate) inner: fricon::Dataset,
}

impl Dataset {
    pub(crate) fn new(inner: fricon::Dataset) -> Self {
        Self { inner }
    }
}

fn helper_module(py: Python<'_>) -> PyResult<&PyObject> {
    static IO_MODULE: GILOnceCell<PyObject> = GILOnceCell::new();
    IO_MODULE.get_or_try_init(py, || py.import("fricon._helper").map(Into::into))
}

#[pymethods]
impl Dataset {
    /// Load the dataset as a polars LazyFrame.
    ///
    /// Returns:
    ///     A polars LazyFrame.
    pub fn to_polars(&self, py: Python<'_>) -> PyResult<PyObject> {
        // Pass dataset directory; helper will gather chunk files.
        helper_module(py)?.call_method1(py, "read_polars", (self.inner.path(),))
    }

    /// Load the dataset as an Arrow Table.
    ///
    /// Returns:
    ///     An Arrow Table.
    pub fn to_arrow(&self, py: Python<'_>) -> PyResult<PyObject> {
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

    /// Id of the dataset.
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

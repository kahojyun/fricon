use std::path::PathBuf;

use anyhow::{Result, bail};
use arrow::datatypes::Schema;
use pyo3::{
    prelude::*,
    sync::GILOnceCell,
    types::{PyDict, PyList},
};
use pyo3_async_runtimes::tokio::get_runtime;

use crate::dataset::Dataset;
use crate::writer::DatasetWriter;
use fricon::{DatasetMetadata, DatasetRecord};

/// A client of fricon workspace server.
#[pyclass(module = "fricon._core")]
#[derive(Clone)]
pub struct Workspace {
    client: fricon::Client,
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
        let client = get_runtime().block_on(fricon::Client::connect(&path))?;
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
    ///     tags: Tags of the dataset. Duplicate tags will be add only once.
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
        let _ = index_columns; // TODO: support index columns
        let description = description.unwrap_or_default();
        let tags = tags.unwrap_or_default();

        // Enter Tokio runtime context to handle tokio::spawn calls in DatasetWriter::new
        let runtime = get_runtime();
        let _guard = runtime.enter();

        let writer = self
            .workspace
            .client
            .create_dataset(name, description, tags)?;

        // Start with empty schema - will be inferred on first write
        Ok(DatasetWriter::new(writer, std::sync::Arc::new(Schema::empty())))
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
            Ok(Dataset::new(inner))
        } else if let Ok(uuid) = dataset_id.extract::<String>() {
            let inner = get_runtime().block_on(self.workspace.client.get_dataset_by_uuid(uuid))?;
            Ok(Dataset::new(inner))
        } else {
            bail!("Invalid dataset id.")
        }
    }

    /// List all datasets in the workspace.
    ///
    /// Returns:
    ///     A pandas dataframe containing information of all datasets.
    pub fn list_all(&self, py: Python<'_>) -> PyResult<PyObject> {
        static FROM_RECORDS: GILOnceCell<PyObject> = GILOnceCell::new();

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

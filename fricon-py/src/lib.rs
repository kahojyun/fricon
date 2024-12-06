#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use fricon::proto::{
    data_storage_service_client::DataStorageServiceClient, CreateRequest, GetRequest, GetResponse,
    Metadata, WriteRequest, WriteResponse,
};
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{timezone_utc, PyDateTime},
};
use pyo3_async_runtimes::tokio::future_into_py;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{metadata::MetadataValue, transport::Channel};

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use super::connect;
}

type TonicClient = DataStorageServiceClient<Channel>;

#[pyfunction]
pub fn connect(py: Python<'_>, addr: String) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(py, async move {
        let client = DataStorageServiceClient::connect(addr)
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to connect: {e}")))?;
        Ok(Client { inner: client })
    })
}

#[pyclass(frozen)]
pub struct Client {
    inner: TonicClient,
}

#[pymethods]
impl Client {
    /// Create a new dataset.
    ///
    /// A background task is spawned to write data to the dataset.
    ///
    /// Args:
    ///     name (str): The name of the dataset.
    ///     description (str | None): The description of the dataset.
    ///     tags (List[str] | None): The tags of the dataset.
    ///
    /// Returns:
    ///     DatasetWriter: The dataset writer.
    #[pyo3(signature = (name, description=None, tags=None))]
    pub fn create_dataset<'py>(
        slf: &Bound<'py, Self>,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let mut inner = slf.get().inner.clone();
        future_into_py(slf.py(), async move {
            let metadata = Metadata {
                name: Some(name),
                description,
                tags: tags.unwrap_or_default(),
            };
            let request = CreateRequest {
                metadata: Some(metadata),
            };
            let response = inner
                .create(request)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to create dataset: {e}")))?;
            let response = response.into_inner();
            let write_token = response.write_token;
            let (tx, rx) = mpsc::channel(128);
            let rx = ReceiverStream::new(rx);
            let mut write_stream_request = tonic::Request::new(rx);
            write_stream_request
                .metadata_mut()
                .insert_bin("fricon-token-bin", MetadataValue::from_bytes(&write_token));
            let (result_tx, result_rx) = oneshot::channel();
            tokio::spawn(async move {
                let result = inner
                    .write(write_stream_request)
                    .await
                    .map(tonic::Response::into_inner)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to write data: {e}")));
                let _ = result_tx.send(result);
            });
            let writer = DatasetWriter {
                tx: Some(tx),
                result_rx: Some(result_rx),
                uid: None,
            };
            Ok(writer)
        })
    }

    /// Get a dataset by its UID.
    fn get_dataset<'py>(slf: &Bound<'py, Self>, uid: String) -> PyResult<Bound<'py, PyAny>> {
        let py = slf.py();
        let mut inner = slf.get().inner.clone();
        future_into_py(py, async move {
            let request = GetRequest { uid };
            let response = inner
                .get(request)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get dataset: {e}")))?;
            let GetResponse {
                path,
                metadata:
                    Some(Metadata {
                        name: Some(name),
                        description,
                        tags,
                    }),
                created_at: Some(created_at),
            } = response.into_inner()
            else {
                return Err(PyRuntimeError::new_err("Failed to get dataset"));
            };

            let created_at = Python::with_gil(|py| {
                #[allow(clippy::cast_precision_loss)]
                PyDateTime::from_timestamp(py, created_at.seconds as f64, Some(&timezone_utc(py)))
                    .map(Bound::unbind)
            })?;
            let info = DatasetInfo {
                name,
                description,
                tags,
                path,
                created_at,
            };
            Ok(info)
        })
    }
}

#[pyclass]
pub struct DatasetWriter {
    tx: Option<mpsc::Sender<WriteRequest>>,
    result_rx: Option<oneshot::Receiver<PyResult<WriteResponse>>>,
    uid: Option<String>,
}

#[pymethods]
impl DatasetWriter {
    /// Write data to the dataset.
    ///
    /// Will block if the internal buffer is full.
    ///
    /// Args:
    ///     data (bytes): The data to write.
    ///
    /// Raises:
    ///     RuntimeError: If the dataset writer is closed or connection is lost.
    pub fn write(&self, data: Vec<u8>) -> PyResult<()> {
        self.tx.as_ref().map_or_else(
            || Err(PyRuntimeError::new_err("DatasetWriter is closed")),
            |tx| {
                tx.blocking_send(WriteRequest { record_batch: data })
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to write data: {e}")))
            },
        )
    }

    /// Close the dataset writer.
    ///
    /// Raises:
    ///     RuntimeError: If the dataset writer is closed or connection is lost.
    pub fn aclose(slf: Bound<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        let py = slf.py();
        let result_rx = {
            let mut slf = slf.borrow_mut();
            slf.tx = None;
            slf.result_rx.take()
        }
        .ok_or_else(|| PyRuntimeError::new_err("DatasetWriter is closed"))?;
        let slf = slf.unbind();
        future_into_py(py, async move {
            let result = result_rx.await.unwrap();
            let response = result
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to close writer: {e}")))?;
            Python::with_gil(|py| slf.bind(py).borrow_mut().uid = Some(response.uid));
            Ok(())
        })
    }

    /// Get the UID of the dataset.
    ///
    /// Returns:
    ///     str: The UID of the dataset.
    ///
    /// Raises:
    ///     RuntimeError: If the dataset writer is not closed yet.
    #[getter]
    pub fn uid(&self) -> PyResult<String> {
        self.uid
            .clone()
            .ok_or_else(|| PyRuntimeError::new_err("DatasetWriter is not closed"))
    }
}

#[pyclass(frozen, get_all)]
struct DatasetInfo {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    path: String,
    created_at: Py<PyDateTime>,
}

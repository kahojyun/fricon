#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use anyhow::{bail, Context, Result};
use arrow::{array::RecordBatch, ipc::writer::StreamWriter, pyarrow::PyArrowType};
use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use fricon::{
    cli::Cli,
    proto::{
        data_storage_service_client::DataStorageServiceClient,
        fricon_service_client::FriconServiceClient, CreateRequest, GetRequest, GetResponse,
        Metadata, VersionRequest, WriteRequest, WriteResponse,
    },
    VERSION,
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use pyo3_async_runtimes::tokio::future_into_py;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{metadata::MetadataValue, transport::Channel};

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use super::{connect, lib_main};
}

#[pyfunction]
#[must_use]
pub fn lib_main(py: Python<'_>) -> i32 {
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

#[pyfunction]
pub fn connect(py: Python<'_>, addr: String) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(py, async move { Ok(Client::connect(addr).await?) })
}

#[pyclass(frozen)]
pub struct Client {
    inner: DataStorageServiceClient<Channel>,
}

impl Client {
    async fn connect(addr: String) -> Result<Self> {
        let channel = Channel::from_shared(addr)?.connect().await?;
        let mut fricon_client = FriconServiceClient::new(channel.clone());
        let server_version = fricon_client
            .version(VersionRequest {})
            .await?
            .into_inner()
            .version;
        let client_version = VERSION;
        if server_version != client_version {
            bail!("Server version mismatch: client={client_version}, server={server_version}");
        }
        let inner = DataStorageServiceClient::new(channel);
        Ok(Self { inner })
    }
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
        &self,
        py: Python<'py>,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        async fn create(
            mut client: DataStorageServiceClient<Channel>,
            name: String,
            description: Option<String>,
            tags: Option<Vec<String>>,
        ) -> Result<DatasetWriter> {
            let metadata = Metadata {
                name: Some(name),
                description,
                tags: tags.unwrap_or_default(),
            };
            let request = CreateRequest {
                metadata: Some(metadata),
            };
            let response = client
                .create(request)
                .await
                .context("Failed to create dataset.")?;
            let response = response.into_inner();
            let write_token = response.write_token;
            let (tx, rx) = mpsc::channel(128);
            let rx = ReceiverStream::new(rx).map(|b: RecordBatch| {
                let mut writer = StreamWriter::try_new(
                    Vec::with_capacity(b.get_array_memory_size()),
                    &b.schema(),
                )
                .expect("Failed to create writer");
                writer.write(&b).expect("Failed to write data");
                let buffer = writer.into_inner().expect("Failed to get inner buffer");
                WriteRequest {
                    record_batch: buffer,
                }
            });
            let mut write_stream_request = tonic::Request::new(rx);
            write_stream_request
                .metadata_mut()
                .insert_bin("fricon-token-bin", MetadataValue::from_bytes(&write_token));
            let (result_tx, result_rx) = oneshot::channel();
            tokio::spawn(async move {
                let result = client
                    .write(write_stream_request)
                    .await
                    .map(tonic::Response::into_inner)
                    .context("Failed to write data.");
                let _ = result_tx.send(result);
            });
            let writer = DatasetWriter {
                tx: Some(tx),
                result_rx: Some(result_rx),
                uid: None,
            };
            Ok(writer)
        }

        let inner = self.inner.clone();
        future_into_py(py, async move {
            Ok(create(inner, name, description, tags).await?)
        })
    }

    /// Get a dataset by its UID.
    fn get_dataset<'py>(&self, py: Python<'py>, uid: String) -> PyResult<Bound<'py, PyAny>> {
        async fn get_dataset_info(
            mut inner: DataStorageServiceClient<Channel>,
            uid: String,
        ) -> Result<DatasetInfo> {
            let request = GetRequest { uid };
            let response = inner.get(request).await.context("Failed to get dataset.")?;
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
                bail!("Invalid dataset.");
            };

            let created_at = Utc.timestamp_opt(created_at.seconds, 0).unwrap();

            let info = DatasetInfo {
                name,
                description,
                tags,
                path,
                created_at,
            };
            Ok(info)
        }

        let inner = self.inner.clone();
        future_into_py(py, async move { Ok(get_dataset_info(inner, uid).await?) })
    }
}

#[pyclass]
pub struct DatasetWriter {
    tx: Option<mpsc::Sender<RecordBatch>>,
    result_rx: Option<oneshot::Receiver<Result<WriteResponse>>>,
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
    pub fn write(&self, data: PyArrowType<RecordBatch>) -> Result<()> {
        let Some(tx) = &self.tx else {
            bail!("DatasetWriter is closed.")
        };
        tx.blocking_send(data.0).context("Failed to write data.")
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
            let response = result.context("Failed to close writer.")?;
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
    created_at: DateTime<Utc>,
}

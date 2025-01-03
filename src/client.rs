use anyhow::{anyhow, bail, ensure, Context, Result};
use arrow::{array::RecordBatch, ipc::writer::StreamWriter};
use bytes::Bytes;
use futures::prelude::*;
use hyper_util::rt::TokioIo;
use semver::Version;
use tokio::{io, sync::mpsc, task::JoinHandle};
use tokio_util::io::{ReaderStream, SyncIoBridge};
use tonic::{metadata::MetadataValue, transport::Channel, Request};
use tower::service_fn;
use tracing::error;

use crate::{
    dataset::Dataset,
    ipc::Ipc,
    paths::IpcFile,
    proto::{
        data_storage_service_client::DataStorageServiceClient,
        fricon_service_client::FriconServiceClient, CreateRequest, VersionRequest, WriteRequest,
        WriteResponse, WRITE_TOKEN,
    },
    VERSION,
};

#[derive(Debug, Clone)]
pub struct Client {
    channel: Channel,
}

impl Client {
    /// # Errors
    ///
    /// 1. Cannot connect to the IPC socket.
    /// 2. Server version mismatch.
    pub async fn connect(path: IpcFile) -> Result<Self> {
        let channel = connect_ipc_channel(path).await?;
        check_server_version(channel.clone()).await?;
        Ok(Self { channel })
    }

    /// # Errors
    ///
    /// Server Errors
    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index: Vec<String>,
    ) -> Result<DatasetWriter> {
        let request = CreateRequest {
            name: Some(name),
            description: Some(description),
            tags,
            index,
        };
        let mut client = DataStorageServiceClient::new(self.channel.clone());
        let response = client.create(request).await?;
        let write_token = response
            .into_inner()
            .write_token
            .ok_or_else(|| anyhow!("No write token returned."))?;
        Ok(DatasetWriter::new(client, write_token))
    }

    pub async fn get_dataset_by_id(&self, id: i64) -> Dataset {
        todo!()
    }

    pub async fn get_dataset_by_uid(&self, uid: String) -> Dataset {
        todo!()
    }
}

pub struct DatasetWriter {
    tx: mpsc::Sender<RecordBatch>,
    writer_handle: Option<JoinHandle<Result<()>>>,
    connection_handle: JoinHandle<Result<WriteResponse>>,
}

impl DatasetWriter {
    fn new(mut client: DataStorageServiceClient<Channel>, token: Bytes) -> Self {
        let (tx, mut rx) = mpsc::channel::<RecordBatch>(16);
        let (dtx, drx) = io::duplex(1024 * 1024);
        let writer_handle = tokio::task::spawn_blocking(move || {
            let Some(batch) = rx.blocking_recv() else {
                bail!("No record batch received.")
            };
            let dtx = SyncIoBridge::new(dtx);
            let mut writer = StreamWriter::try_new(dtx, &batch.schema())?;
            writer.write(&batch)?;
            while let Some(batch) = rx.blocking_recv() {
                writer.write(&batch)?;
            }
            writer.finish()?;
            Ok(())
        });
        let connection_handle = tokio::spawn(async move {
            let request_stream = ReaderStream::new(drx).map(|chunk| {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        error!("Writer failed: {:?}", e);
                        Bytes::new()
                    }
                };
                WriteRequest { chunk: Some(chunk) }
            });
            let mut request = Request::new(request_stream);
            request
                .metadata_mut()
                .insert_bin(WRITE_TOKEN, MetadataValue::from_bytes(&token));
            let response = client.write(request).await?;
            Ok(response.into_inner())
        });
        Self {
            tx,
            writer_handle: Some(writer_handle),
            connection_handle,
        }
    }

    /// # Errors
    ///
    /// Writer failed because:
    ///
    /// 1. Record batch schema mismatch.
    /// 2. Connection error.
    pub async fn write(&mut self, data: RecordBatch) -> Result<()> {
        if self.tx.send(data).await == Ok(()) {
            Ok(())
        } else {
            let Some(writer_handle) = self.writer_handle.take() else {
                bail!("Writer already finished.");
            };
            let writer_result = writer_handle.await.context("Writer panicked.")?;
            writer_result.context("Writer failed.")
        }
    }

    /// # Errors
    ///
    /// Writer failed because:
    ///
    /// 1. Record batch schema mismatch.
    /// 2. Connection error.
    pub async fn finish(self) -> Result<i64> {
        let id = self
            .connection_handle
            .await??
            .id
            .ok_or_else(|| anyhow!("No dataset id returned."))?;
        Ok(id)
    }
}

async fn connect_ipc_channel(path: IpcFile) -> Result<Channel> {
    let channel = Channel::from_static("http://ignored.com:50051")
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = path.connect().await?;
                anyhow::Ok(TokioIo::new(stream))
            }
        }))
        .await?;
    Ok(channel)
}

async fn check_server_version(channel: Channel) -> Result<()> {
    let request = VersionRequest {};
    let response = FriconServiceClient::new(channel).version(request).await?;
    let server_version = response.into_inner().version;
    let server_version: Version = server_version.parse()?;
    let client_version: Version = VERSION.parse()?;
    ensure!(
        client_version == server_version,
        "Server and client version mismatch. Server: {server_version}, Client: {client_version}"
    );
    Ok(())
}

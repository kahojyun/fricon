use anyhow::{bail, ensure, Result};
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
    ipc::IpcConnect,
    paths::IpcFile,
    proto::{
        data_storage_service_client::DataStorageServiceClient,
        fricon_service_client::FriconServiceClient, CreateRequest, Metadata, VersionRequest,
        WriteRequest, WriteResponse, WRITE_TOKEN,
    },
    VERSION,
};

#[derive(Debug, Clone)]
pub struct Client {
    channel: Channel,
}

impl Client {
    pub async fn connect(path: IpcFile) -> Result<Self> {
        let channel = connect_ipc_channel(path).await?;
        check_server_version(channel.clone()).await?;
        Ok(Self { channel })
    }

    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<DatasetWriter> {
        let request = CreateRequest {
            metadata: Some(Metadata {
                name: Some(name),
                description: Some(description),
                tags,
            }),
        };
        let mut client = DataStorageServiceClient::new(self.channel.clone());
        let response = client.create(request).await?;
        let write_token = response.into_inner().write_token;
        Ok(DatasetWriter::new(client, write_token))
    }
}

pub struct DatasetWriter {
    tx: mpsc::Sender<RecordBatch>,
    handle: JoinHandle<Result<WriteResponse>>,
}

impl DatasetWriter {
    fn new(mut client: DataStorageServiceClient<Channel>, token: Bytes) -> Self {
        let (tx, mut rx) = mpsc::channel::<RecordBatch>(16);
        let (dtx, drx) = io::duplex(1024 * 1024);
        tokio::task::spawn_blocking(move || {
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
        let handle = tokio::spawn(async move {
            let request_stream = ReaderStream::new(drx).map(|chunk| {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        error!("Writer failed: {:?}", e);
                        Bytes::new()
                    }
                };
                WriteRequest { chunk }
            });
            let mut request = Request::new(request_stream);
            request
                .metadata_mut()
                .insert_bin(WRITE_TOKEN, MetadataValue::from_bytes(&token));
            let response = client.write(request).await?;
            Ok(response.into_inner())
        });
        Self { tx, handle }
    }

    pub fn blocking_write(&self, data: RecordBatch) -> Result<()> {
        self.tx.blocking_send(data)?;
        Ok(())
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

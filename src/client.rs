use anyhow::{bail, ensure, Context, Result};
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
    ipc::Ipc,
    paths::IpcFile,
    proto::{
        data_storage_service_client::DataStorageServiceClient,
        fricon_service_client::FriconServiceClient, get_request::IdEnum, AddTagsRequest,
        CreateRequest, GetRequest, ListRequest, RemoveTagsRequest, ReplaceTagsRequest,
        UpdateDescriptionRequest, UpdateFavoriteRequest, UpdateNameRequest, VersionRequest,
        WriteRequest, WriteResponse, WRITE_TOKEN,
    },
    VERSION,
};

pub use crate::{
    dataset::{Info, DATASET_NAME},
    db::DatasetRecord,
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
    /// Server errors
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
        let mut client = self.data_storage_service_client();
        let response = client.create(request).await?;
        let write_token = response
            .into_inner()
            .write_token
            .context("No write token returned.")?;
        Ok(DatasetWriter::new(client, write_token))
    }

    /// # Errors
    ///
    /// 1. Not found.
    /// 2. Server errors
    pub async fn get_dataset_by_id(&self, id: i64) -> Result<DatasetRecord> {
        self.get_dataset_by_id_enum(IdEnum::Id(id)).await
    }

    /// # Errors
    ///
    /// 1. Not found.
    /// 2. Server errors
    pub async fn get_dataset_by_uid(&self, uid: String) -> Result<DatasetRecord> {
        self.get_dataset_by_id_enum(IdEnum::Uid(uid)).await
    }

    pub async fn list_all_datasets(&self) -> Result<Vec<DatasetRecord>> {
        let request = ListRequest {};
        let response = self.data_storage_service_client().list(request).await?;
        let records = response.into_inner().datasets;
        records.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn get_dataset_by_id_enum(&self, id: IdEnum) -> Result<DatasetRecord> {
        let request = GetRequest { id_enum: Some(id) };
        let response = self.data_storage_service_client().get(request).await?;
        let record = response
            .into_inner()
            .dataset
            .context("No dataset returned.")?;
        record.try_into().context("Invalid dataset record.")
    }

    pub async fn replace_dataset_tags(&self, id: i64, tags: Vec<String>) -> Result<()> {
        let request = ReplaceTagsRequest { id: Some(id), tags };
        let _response = self
            .data_storage_service_client()
            .replace_tags(request)
            .await?;
        Ok(())
    }

    pub async fn add_dataset_tags(&self, id: i64, tags: Vec<String>) -> Result<()> {
        let request = AddTagsRequest { id: Some(id), tags };
        let _response = self.data_storage_service_client().add_tags(request).await?;
        Ok(())
    }

    pub async fn remove_dataset_tags(&self, id: i64, tags: Vec<String>) -> Result<()> {
        let request = RemoveTagsRequest { id: Some(id), tags };
        let _response = self
            .data_storage_service_client()
            .remove_tags(request)
            .await?;
        Ok(())
    }

    pub async fn update_dataset_name(&self, id: i64, name: String) -> Result<()> {
        let request = UpdateNameRequest {
            id: Some(id),
            name: Some(name),
        };
        let _response = self
            .data_storage_service_client()
            .update_name(request)
            .await?;
        Ok(())
    }

    pub async fn update_dataset_description(&self, id: i64, description: String) -> Result<()> {
        let request = UpdateDescriptionRequest {
            id: Some(id),
            description: Some(description),
        };
        let _response = self
            .data_storage_service_client()
            .update_description(request)
            .await?;
        Ok(())
    }

    pub async fn update_dataset_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        let request = UpdateFavoriteRequest {
            id: Some(id),
            favorite: Some(favorite),
        };
        let _response = self
            .data_storage_service_client()
            .update_favorite(request)
            .await?;
        Ok(())
    }

    fn data_storage_service_client(&self) -> DataStorageServiceClient<Channel> {
        DataStorageServiceClient::new(self.channel.clone())
    }
}

pub struct WriterHandle {
    tx: mpsc::Sender<RecordBatch>,
    handle: JoinHandle<Result<()>>,
}

pub struct DatasetWriter {
    handle: Option<WriterHandle>,
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
            handle: Some(WriterHandle {
                tx,
                handle: writer_handle,
            }),
            connection_handle,
        }
    }

    /// # Errors
    ///
    /// Writer failed because:
    ///
    /// 1. Record batch schema mismatch.
    /// 2. Connection error.
    ///
    /// # Panics
    pub async fn write(&mut self, data: RecordBatch) -> Result<()> {
        let Some(WriterHandle { tx, .. }) = self.handle.as_mut() else {
            bail!("Writer closed.");
        };
        if tx.send(data).await == Ok(()) {
            Ok(())
        } else {
            let WriterHandle { handle, .. } = self.handle.take().expect("Not none here.");
            let writer_result = handle.await.context("Writer panicked.")?;
            writer_result.context("Writer failed.")
        }
    }

    /// # Errors
    ///
    /// Writer failed because:
    ///
    /// 1. Record batch schema mismatch.
    /// 2. Connection error.
    pub async fn finish(mut self) -> Result<i64> {
        let WriterHandle { tx, handle } = self.handle.take().context("Already finished.")?;
        drop(tx);
        handle
            .await
            .context("Writer panicked.")?
            .context("Writer failed.")?;
        let id = self
            .connection_handle
            .await
            .context("Connector panicked.")?
            .context("Connection failed.")?
            .id
            .context("No dataset id returned.")?;
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

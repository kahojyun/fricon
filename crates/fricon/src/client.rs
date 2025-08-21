pub use crate::server::DatasetRecord;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use arrow::{array::RecordBatch, ipc::writer::StreamWriter};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::prelude::*;
use hyper_util::rt::TokioIo;
use semver::Version;
use tokio::{io, sync::mpsc, task::JoinHandle};
use tokio_util::io::{ReaderStream, SyncIoBridge};
use tonic::{Request, metadata::MetadataValue, transport::Channel};
use tower::service_fn;
use tracing::error;
use uuid::Uuid;

use crate::{
    VERSION, dataset, ipc,
    proto::{
        self, AddTagsRequest, CreateRequest, GetRequest, RemoveTagsRequest, SearchRequest,
        UpdateRequest, VersionRequest, WriteRequest, WriteResponse,
        data_storage_service_client::DataStorageServiceClient,
        fricon_service_client::FriconServiceClient, get_request::IdEnum,
    },
    workspace::{WorkspacePaths, WorkspaceRoot},
};

#[derive(Debug, Clone)]
pub struct Client {
    channel: Channel,
    workspace_paths: WorkspacePaths,
}

impl Client {
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path)?;
        WorkspaceRoot::validate(path.clone())?;
        let workspace_paths = WorkspacePaths::new(path);
        let channel = connect_ipc_channel(workspace_paths.ipc_file()).await?;
        check_server_version(channel.clone()).await?;
        Ok(Self {
            channel,
            workspace_paths,
        })
    }

    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        index_columns: Vec<String>,
    ) -> Result<DatasetWriter> {
        let request = CreateRequest {
            name,
            description,
            tags,
            index_columns,
        };
        let response = self.data_storage().create(request).await?;
        let write_token = response.into_inner().write_token;
        Ok(DatasetWriter::new(self.clone(), write_token))
    }

    pub async fn get_dataset_by_id(&self, id: i32) -> Result<Dataset> {
        self.get_dataset_by_id_enum(IdEnum::Id(id)).await
    }

    pub async fn get_dataset_by_uuid(&self, uuid: String) -> Result<Dataset> {
        self.get_dataset_by_id_enum(IdEnum::Uuid(uuid)).await
    }

    pub async fn list_all_datasets(&self) -> Result<Vec<DatasetRecord>> {
        // TODO: Implement pagination
        let request = SearchRequest::default();
        let response = self.data_storage().search(request).await?;
        let records = response.into_inner().datasets;
        records.into_iter().map(TryInto::try_into).collect()
    }

    async fn get_dataset_by_id_enum(&self, id: IdEnum) -> Result<Dataset> {
        let request = GetRequest { id_enum: Some(id) };
        let response = self.data_storage().get(request).await?;
        let record = response
            .into_inner()
            .dataset
            .context("No dataset returned.")?;
        Ok(Dataset {
            client: self.clone(),
            record: record.try_into().context("Invalid dataset record.")?,
        })
    }

    fn data_storage(&self) -> DataStorageServiceClient<Channel> {
        DataStorageServiceClient::new(self.channel.clone())
    }
}

struct WriterHandle {
    tx: mpsc::Sender<RecordBatch>,
    handle: JoinHandle<Result<()>>,
}

pub struct DatasetWriter {
    handle: Option<WriterHandle>,
    connection_handle: JoinHandle<Result<WriteResponse>>,
    client: Client,
}

impl DatasetWriter {
    fn new(client: Client, token: Bytes) -> Self {
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
        let connection_handle = {
            let client = client.clone();
            tokio::spawn(async move {
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
                    .insert_bin(proto::WRITE_TOKEN_KEY, MetadataValue::from_bytes(&token));
                let response = client.data_storage().write(request).await?;
                Ok(response.into_inner())
            })
        };
        Self {
            handle: Some(WriterHandle {
                tx,
                handle: writer_handle,
            }),
            connection_handle,
            client,
        }
    }

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

    pub async fn finish(mut self) -> Result<Dataset> {
        let WriterHandle { tx, handle } = self.handle.take().context("Already finished.")?;
        drop(tx);
        handle
            .await
            .context("Writer panicked.")?
            .context("Writer failed.")?;
        let dataset = self
            .connection_handle
            .await
            .context("Connector panicked.")?
            .context("Connection failed.")?
            .dataset
            .context("No dataset returned.")?;
        Ok(Dataset {
            client: self.client,
            record: dataset
                .try_into()
                .context("Failed to convert dataset record")?,
        })
    }
}

async fn connect_ipc_channel(path: PathBuf) -> Result<Channel> {
    let channel = Channel::from_static("http://ignored.com:50051")
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = ipc::connect(path).await?;
                anyhow::Ok(TokioIo::new(stream))
            }
        }))
        .await?;
    Ok(channel)
}

pub struct Dataset {
    client: Client,
    record: DatasetRecord,
}

impl Dataset {
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.client
            .workspace_paths
            .dataset_path_from_uuid(self.record.metadata.uuid)
    }

    #[must_use]
    pub fn arrow_file(&self) -> PathBuf {
        self.path().join(dataset::DATASET_NAME)
    }

    #[must_use]
    pub const fn id(&self) -> i32 {
        self.record.id
    }

    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.record.metadata.uuid
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.record.metadata.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.record.metadata.description
    }

    #[must_use]
    pub const fn favorite(&self) -> bool {
        self.record.metadata.favorite
    }

    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.record.metadata.tags
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.record.metadata.created_at
    }

    #[must_use]
    pub fn index_columns(&self) -> &[String] {
        &self.record.metadata.index_columns
    }

    pub async fn add_tags(&self, tags: Vec<String>) -> Result<()> {
        let request = AddTagsRequest {
            id: self.record.id,
            tags,
        };
        let _response = self.client.data_storage().add_tags(request).await?;
        Ok(())
    }

    pub async fn remove_tags(&self, tags: Vec<String>) -> Result<()> {
        let request = RemoveTagsRequest {
            id: self.record.id,
            tags,
        };
        let _response = self.client.data_storage().remove_tags(request).await?;
        Ok(())
    }

    pub async fn update_metadata(
        &self,
        name: Option<String>,
        description: Option<String>,
        favorite: Option<bool>,
    ) -> Result<()> {
        let request = UpdateRequest {
            id: self.record.id,
            name,
            description,
            favorite,
        };
        let _response = self.client.data_storage().update(request).await?;
        Ok(())
    }
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

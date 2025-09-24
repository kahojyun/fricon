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
use tokio::{
    io,
    sync::mpsc,
    task::{JoinHandle, spawn_blocking},
};
use tokio_util::io::{ReaderStream, SyncIoBridge};
use tonic::{Request, transport::Channel};
use tower::service_fn;
use tracing::error;
use uuid::Uuid;

use crate::{
    VERSION,
    database::DatasetStatus,
    dataset_manager::DatasetRecord,
    ipc,
    proto::{
        self, AddTagsRequest, CreateMetadata, CreateRequest, CreateResponse, GetRequest,
        RemoveTagsRequest, SearchRequest, UpdateRequest, VersionRequest,
        create_request::CreateMessage, dataset_service_client::DatasetServiceClient,
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

    pub fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<DatasetWriter> {
        Ok(DatasetWriter::new(self.clone(), name, description, tags))
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
        let response = self.dataset_service().search(request).await?;
        let records = response.into_inner().datasets;
        records.into_iter().map(TryInto::try_into).collect()
    }

    async fn get_dataset_by_id_enum(&self, id: IdEnum) -> Result<Dataset> {
        let request = GetRequest { id_enum: Some(id) };
        let response = self.dataset_service().get(request).await?;
        let record = response
            .into_inner()
            .dataset
            .context("No dataset returned.")?;
        Ok(Dataset {
            client: self.clone(),
            record: record.try_into().context("Invalid dataset record.")?,
        })
    }

    fn dataset_service(&self) -> DatasetServiceClient<Channel> {
        DatasetServiceClient::new(self.channel.clone())
    }
}

struct WriterHandle {
    tx: mpsc::Sender<RecordBatch>,
    handle: JoinHandle<Result<()>>,
}

pub struct DatasetWriter {
    handle: Option<WriterHandle>,
    connection_handle: JoinHandle<Result<CreateResponse>>,
    client: Client,
}

impl DatasetWriter {
    fn new(client: Client, name: String, description: String, tags: Vec<String>) -> Self {
        let (tx, mut rx) = mpsc::channel::<RecordBatch>(16);
        let (dtx, drx) = io::duplex(1024 * 1024);

        let request_stream = build_request_stream(name, description, tags, ReaderStream::new(drx));

        let writer_handle = spawn_blocking(move || {
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
                let request = Request::new(request_stream);
                let response = client.dataset_service().create(request).await?;
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

fn build_request_stream(
    name: String,
    description: String,
    tags: Vec<String>,
    bytes_stream: impl Stream<Item = io::Result<Bytes>>,
) -> impl Stream<Item = CreateRequest> {
    let first_message = CreateMessage::Metadata(CreateMetadata {
        name,
        description,
        tags,
    });
    let payload_stream = bytes_stream.map(|chunk| match chunk {
        Ok(chunk) => CreateMessage::Payload(chunk),
        Err(e) => {
            error!("Reader failed: {:?}", e);
            CreateMessage::Abort(proto::CreateAbort {
                reason: format!("Reader failed: {e:?}"),
            })
        }
    });
    stream::once(async move { first_message })
        .chain(payload_stream)
        .map(|msg| CreateRequest {
            create_message: Some(msg),
        })
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
    pub fn status(&self) -> DatasetStatus {
        self.record.metadata.status
    }

    pub async fn add_tags(&self, tags: Vec<String>) -> Result<()> {
        let request = AddTagsRequest {
            id: self.record.id,
            tags,
        };
        let _response = self.client.dataset_service().add_tags(request).await?;
        Ok(())
    }

    pub async fn remove_tags(&self, tags: Vec<String>) -> Result<()> {
        let request = RemoveTagsRequest {
            id: self.record.id,
            tags,
        };
        let _response = self.client.dataset_service().remove_tags(request).await?;
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
        let _response = self.client.dataset_service().update(request).await?;
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

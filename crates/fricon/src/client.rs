use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, bail, ensure};
use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::SchemaRef;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures::prelude::*;
use hyper_util::rt::TokioIo;
use semver::Version;
use tokio::{
    io::duplex,
    sync::{mpsc, oneshot},
    task::{JoinHandle, spawn_blocking},
};
use tokio_util::io::{ReaderStream, SyncIoBridge};
use tonic::{Request, transport::Channel};
use tower::service_fn;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    DEFAULT_DATASET_LIST_LIMIT, VERSION,
    database::DatasetStatus,
    dataset::{DatasetArray, DatasetRow, DatasetSchema},
    dataset_manager::DatasetRecord,
    ipc,
    proto::{
        AddTagsRequest, CreateFinish, CreateMetadata, CreateRequest, CreateResponse, GetRequest,
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
    #[instrument(skip(path), fields(workspace.path = ?path.as_ref()))]
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path)?;
        WorkspaceRoot::validate(path.clone())?;
        let workspace_paths = WorkspacePaths::new(path);
        debug!(path = ?workspace_paths.root(), "Connecting to fricon server");
        let channel = connect_ipc_channel(workspace_paths.ipc_file()).await?;
        check_server_version(channel.clone()).await?;
        info!(path = ?workspace_paths.root(), "Connected to fricon server");
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
        schema: DatasetSchema,
    ) -> Result<DatasetWriter> {
        Ok(DatasetWriter::new(
            self.clone(),
            name,
            description,
            tags,
            schema,
        ))
    }

    pub async fn get_dataset_by_id(&self, id: i32) -> Result<Dataset> {
        self.get_dataset_by_id_enum(IdEnum::Id(id)).await
    }

    pub async fn get_dataset_by_uid(&self, uid: String) -> Result<Dataset> {
        self.get_dataset_by_id_enum(IdEnum::Uid(uid)).await
    }

    pub async fn list_all_datasets(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DatasetRecord>> {
        let limit = limit.unwrap_or(DEFAULT_DATASET_LIST_LIMIT).max(0);
        let page_size = i32::try_from(limit).unwrap_or(i32::MAX);
        let page_token = offset.unwrap_or(0).max(0).to_string();
        let request = SearchRequest {
            page_size,
            page_token,
        };
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
    is_finished: Arc<AtomicBool>,
}

#[derive(Debug)]
enum StreamControl {
    Finish,
    Abort,
}

pub struct DatasetWriter {
    schema: DatasetSchema,
    arrow_schema: SchemaRef,
    writer_handle: Option<WriterHandle>,
    connection_handle: Option<JoinHandle<Result<CreateResponse>>>,
    client: Client,
    control_tx: Option<oneshot::Sender<StreamControl>>,
}

impl DatasetWriter {
    fn new(
        client: Client,
        name: String,
        description: String,
        tags: Vec<String>,
        schema: DatasetSchema,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<RecordBatch>(16);
        let (dtx, drx) = duplex(1024 * 1024);
        let (control_tx, control_rx) = oneshot::channel();
        let is_finished = Arc::new(AtomicBool::new(false));

        let arrow_schema = Arc::new(schema.to_arrow_schema());
        let writer_handle = spawn_blocking({
            let arrow_schema = arrow_schema.clone();
            let is_finished = is_finished.clone();
            move || {
                let dtx = SyncIoBridge::new(dtx);
                let mut writer = StreamWriter::try_new(dtx, &arrow_schema)?;
                while let Some(batch) = rx.blocking_recv() {
                    writer.write(&batch)?;
                }
                if is_finished.load(Ordering::SeqCst) {
                    writer.finish()?;
                }
                Ok(())
            }
        });
        let connection_handle = tokio::spawn({
            let client = client.clone();
            let request_stream =
                build_request_stream(name, description, tags, ReaderStream::new(drx), control_rx);
            async move {
                let request = Request::new(request_stream);
                let response = client.dataset_service().create(request).await?;
                Ok(response.into_inner())
            }
        });
        Self {
            schema,
            arrow_schema,
            writer_handle: Some(WriterHandle {
                tx,
                handle: writer_handle,
                is_finished: is_finished.clone(),
            }),
            connection_handle: Some(connection_handle),
            client,
            control_tx: Some(control_tx),
        }
    }

    pub async fn write(&mut self, row: DatasetRow) -> Result<()> {
        let Some(WriterHandle { tx, .. }) = self.writer_handle.as_mut() else {
            bail!("Writer closed.");
        };
        let row_schema = row.to_schema();
        if row_schema != self.schema {
            bail!(
                "Schema mismatch. expected {:?}, got {:?}",
                self.schema,
                row_schema
            );
        }
        let columns = self
            .schema
            .columns()
            .iter()
            .map(|(name, _)| DatasetArray::from(row.0[name].clone()).into())
            .collect();
        let batch = RecordBatch::try_new(self.arrow_schema.clone(), columns)
            .context("Failed to create RecordBatch")?;
        if tx.send(batch).await.is_ok() {
            Ok(())
        } else {
            let WriterHandle { handle, .. } = self
                .writer_handle
                .take()
                .expect("Handle should be available since tx.send failed");
            let writer_result = handle.await.context("Writer panicked.")?;
            writer_result.context("Writer failed.")
        }
    }

    #[instrument(skip(self))]
    pub async fn finish(self) -> Result<Dataset> {
        self.complete(StreamControl::Finish, true).await
    }

    #[instrument(skip(self))]
    pub async fn abort(self) -> Result<Dataset> {
        self.complete(StreamControl::Abort, false).await
    }

    async fn complete(mut self, control: StreamControl, finished: bool) -> Result<Dataset> {
        let WriterHandle {
            tx,
            handle,
            is_finished,
        } = self.writer_handle.take().context("Already finished.")?;
        is_finished.store(finished, Ordering::SeqCst);
        drop(tx);
        handle
            .await
            .context("Writer panicked.")?
            .context("Writer failed.")?;

        if let Some(control_tx) = self.control_tx.take() {
            let _ = control_tx.send(control);
        }

        let connection_handle = self.connection_handle.take().context("Already finished.")?;
        let dataset = connection_handle
            .await
            .context("Connector panicked.")?
            .context("Connection failed.")?
            .dataset
            .context("No dataset returned.")?;
        let record: crate::dataset_manager::DatasetRecord = dataset
            .try_into()
            .context("Failed to convert dataset record")?;
        info!(dataset.id = record.id, "Dataset write finished");
        Ok(Dataset {
            client: self.client,
            record,
        })
    }

    #[must_use]
    pub fn schema(&self) -> &DatasetSchema {
        &self.schema
    }
}

fn build_request_stream(
    name: String,
    description: String,
    tags: Vec<String>,
    bytes_stream: impl Stream<Item = io::Result<Bytes>>,
    control_rx: oneshot::Receiver<StreamControl>,
) -> impl Stream<Item = CreateRequest> {
    let first_message = CreateMessage::Metadata(CreateMetadata {
        name,
        description,
        tags,
    });
    let payload_stream = bytes_stream.filter_map(|chunk| async {
        match chunk {
            Ok(chunk) => Some(CreateMessage::Payload(chunk)),
            Err(e) => {
                error!(error = %e, "Dataset payload reader failed");
                None // Drop error chunks, the stream will end early and server will treat it as abort
            }
        }
    });
    stream::once(async move { first_message })
        .chain(payload_stream)
        .chain(
            stream::once(async move {
                match control_rx.await {
                    Ok(StreamControl::Finish) => Some(CreateMessage::Finish(CreateFinish {})),
                    Ok(StreamControl::Abort) | Err(_) => None,
                }
            })
            .filter_map(|msg| async move { msg }),
        )
        .map(|msg| CreateRequest {
            create_message: Some(msg),
        })
}

async fn connect_ipc_channel(path: PathBuf) -> Result<Channel> {
    let channel = Channel::from_static("https://ignored.com:50051")
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
            .dataset_path_from_uid(self.record.metadata.uid)
    }

    #[must_use]
    pub const fn id(&self) -> i32 {
        self.record.id
    }

    #[must_use]
    pub fn uid(&self) -> Uuid {
        self.record.metadata.uid
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

#[instrument(skip(channel))]
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
    debug!(server_version = %server_version, client_version = %client_version, "Server version check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures::{StreamExt, stream};
    use tokio::sync::oneshot;

    use super::{StreamControl, build_request_stream};
    use crate::proto::create_request::CreateMessage;

    #[tokio::test]
    async fn build_request_stream_sends_finish_message_on_finish() {
        let (tx, rx) = oneshot::channel();
        tx.send(StreamControl::Finish).expect("send control");
        let stream = build_request_stream(
            "dataset".to_string(),
            "desc".to_string(),
            vec!["tag".to_string()],
            stream::iter(vec![Ok::<Bytes, std::io::Error>(Bytes::from_static(
                b"payload",
            ))]),
            rx,
        );

        let messages: Vec<_> = stream
            .map(|req| req.create_message.expect("message"))
            .collect()
            .await;

        assert!(matches!(messages[0], CreateMessage::Metadata(_)));
        assert!(matches!(messages[1], CreateMessage::Payload(_)));
        assert!(matches!(messages[2], CreateMessage::Finish(_)));
    }

    #[tokio::test]
    async fn build_request_stream_omits_finish_message_on_abort() {
        let (tx, rx) = oneshot::channel();
        tx.send(StreamControl::Abort).expect("send control");
        let stream = build_request_stream(
            "dataset".to_string(),
            "desc".to_string(),
            vec![],
            stream::iter(vec![Ok::<Bytes, std::io::Error>(Bytes::from_static(
                b"payload",
            ))]),
            rx,
        );

        let messages: Vec<_> = stream
            .map(|req| req.create_message.expect("message"))
            .collect()
            .await;

        assert_eq!(messages.len(), 2);
        assert!(matches!(messages[0], CreateMessage::Metadata(_)));
        assert!(matches!(messages[1], CreateMessage::Payload(_)));
    }
}

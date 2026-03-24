use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::{ArrowError, SchemaRef};
use async_stream::stream;
use bytes::{BufMut, Bytes, BytesMut};
use chrono::{DateTime, Utc};
use futures::prelude::*;
use hyper_util::rt::TokioIo;
use semver::Version;
use thiserror::Error;
use tokio::{sync::mpsc, task::JoinHandle};
use tonic::{Code, Request, Status, transport::Channel};
use tower::service_fn;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    DEFAULT_DATASET_LIST_LIMIT, VERSION,
    dataset::{
        model::{DatasetRecord, DatasetStatus},
        schema::{DatasetArray, DatasetRow, DatasetSchema},
    },
    proto::{
        AddTagsRequest, CreateAbort, CreateFinish, CreateMetadata, CreateRequest, CreateResponse,
        GetRequest, RemoveTagsRequest, SearchRequest, UpdateRequest, VersionRequest,
        create_request::CreateMessage, dataset_service_client::DatasetServiceClient,
        fricon_service_client::FriconServiceClient, get_request::IdEnum,
    },
    transport::{ipc, ipc::error::ConnectError},
    workspace::{WorkspaceError, WorkspacePaths, WorkspaceRoot},
};

/// Errors that can occur in [`Client`], [`DatasetWriter`], and [`Dataset`]
/// operations.
#[derive(Debug, Error)]
pub enum ClientError {
    /// No fricon server is running at the given workspace path.
    #[error("No fricon server is running at the workspace path")]
    NotRunning,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("RPC error: {0}")]
    Status(#[from] Status),
    #[error("Version parse error: {0}")]
    VersionParse(#[from] semver::Error),
    #[error("Server version {server} does not match client version {client}")]
    VersionMismatch { server: Version, client: Version },
    #[error("Arrow error: {0}")]
    Arrow(#[from] ArrowError),
    /// Proto message conversion failed.
    #[error("Proto conversion failed: {0}")]
    ProtoConversion(#[from] crate::transport::grpc::codec::CodecError),
    /// The dataset writer has been closed already (via finish or abort).
    #[error("Dataset writer is already closed")]
    WriterClosed,
    /// finish/abort was called more than once.
    #[error("Dataset write operation has already finished or been aborted")]
    AlreadyFinished,
    #[error("Schema mismatch: expected {expected:?}, got {got:?}")]
    SchemaMismatch {
        expected: Box<DatasetSchema>,
        got: Box<DatasetSchema>,
    },
    #[error("Expected dataset in response but none was returned")]
    MissingResponse,
    #[error("Connector task panicked")]
    ConnectorPanic,
}

const MAX_PAYLOAD_CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistingUiProbeResult {
    NotRunning,
    UiShown,
    UiUnavailable,
}

#[derive(Debug, Clone)]
pub struct Client {
    channel: Channel,
    workspace_paths: WorkspacePaths,
}

impl Client {
    #[instrument(skip(path), fields(workspace.path = ?path.as_ref()))]
    pub async fn probe_existing_ui(
        path: impl AsRef<Path>,
    ) -> Result<ExistingUiProbeResult, ClientError> {
        match Self::connect(path).await {
            Ok(client) => match client.show_ui().await {
                Ok(()) => Ok(ExistingUiProbeResult::UiShown),
                Err(ClientError::Status(s)) if s.code() == Code::FailedPrecondition => {
                    Ok(ExistingUiProbeResult::UiUnavailable)
                }
                Err(err) => Err(err),
            },
            Err(ClientError::NotRunning) => Ok(ExistingUiProbeResult::NotRunning),
            Err(err) => Err(err),
        }
    }

    #[instrument(skip(path), fields(workspace.path = ?path.as_ref()))]
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, ClientError> {
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

    #[expect(
        clippy::unused_async,
        reason = "The async constructor is the intended public API after the refactor"
    )]
    pub async fn create_dataset(
        &self,
        name: String,
        description: String,
        tags: Vec<String>,
        schema: DatasetSchema,
    ) -> Result<DatasetWriter, ClientError> {
        Ok(DatasetWriter::new(
            self.clone(),
            name,
            description,
            tags,
            schema,
            tokio::runtime::Handle::current(),
        ))
    }

    pub async fn get_dataset_by_id(&self, id: i32) -> Result<Dataset, ClientError> {
        self.get_dataset_by_id_enum(IdEnum::Id(id)).await
    }

    pub async fn get_dataset_by_uid(&self, uid: String) -> Result<Dataset, ClientError> {
        self.get_dataset_by_id_enum(IdEnum::Uid(uid)).await
    }

    pub async fn list_all_datasets(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DatasetRecord>, ClientError> {
        let limit = limit.unwrap_or(DEFAULT_DATASET_LIST_LIMIT).max(0);
        let page_size = i32::try_from(limit).unwrap_or(i32::MAX);
        let page_token = offset.unwrap_or(0).max(0).to_string();
        let request = SearchRequest {
            page_size,
            page_token,
        };
        let response = self.dataset_service().search(request).await?;
        let records = response.into_inner().datasets;
        records
            .into_iter()
            .map(|r| r.try_into().map_err(ClientError::ProtoConversion))
            .collect()
    }

    async fn get_dataset_by_id_enum(&self, id: IdEnum) -> Result<Dataset, ClientError> {
        let request = GetRequest { id_enum: Some(id) };
        let response = self.dataset_service().get(request).await?;
        let record = response
            .into_inner()
            .dataset
            .ok_or(ClientError::MissingResponse)?;
        let record: DatasetRecord = record.try_into().map_err(ClientError::ProtoConversion)?;
        Ok(Dataset {
            client: self.clone(),
            record,
        })
    }

    pub async fn show_ui(&self) -> Result<(), ClientError> {
        let request = crate::proto::ShowUiRequest {};
        let mut client = FriconServiceClient::new(self.channel.clone());
        client.show_ui(request).await?;
        Ok(())
    }

    fn dataset_service(&self) -> DatasetServiceClient<Channel> {
        DatasetServiceClient::new(self.channel.clone())
    }
}

fn connect_target_missing(err: &tonic::transport::Error) -> bool {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(error) = current {
        if error
            .downcast_ref::<ConnectError>()
            .is_some_and(|connect_error| matches!(connect_error, ConnectError::NotFound(_)))
        {
            return true;
        }
        current = error.source();
    }
    false
}

#[derive(Debug)]
enum StreamMessage {
    Batch(RecordBatch),
    Finish,
    Abort,
}

pub struct DatasetWriter {
    schema: DatasetSchema,
    arrow_schema: SchemaRef,
    tx: Option<mpsc::Sender<StreamMessage>>,
    connection_handle: Option<JoinHandle<Result<CreateResponse, ClientError>>>,
    runtime: tokio::runtime::Handle,
    client: Client,
}

impl DatasetWriter {
    fn new(
        client: Client,
        name: String,
        description: String,
        tags: Vec<String>,
        schema: DatasetSchema,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<StreamMessage>(16);

        let arrow_schema = Arc::new(schema.to_arrow_schema());
        let connection_handle = runtime.spawn({
            let client = client.clone();
            let request_stream =
                build_request_stream(name, description, tags, arrow_schema.clone(), rx);
            async move {
                let request = Request::new(request_stream);
                let response = client.dataset_service().create(request).await?;
                Ok(response.into_inner())
            }
        });
        Self {
            schema,
            arrow_schema,
            tx: Some(tx),
            connection_handle: Some(connection_handle),
            runtime,
            client,
        }
    }

    pub async fn write(&mut self, row: DatasetRow) -> Result<(), ClientError> {
        let Some(tx) = self.tx.as_mut() else {
            return Err(ClientError::WriterClosed);
        };
        let row_schema = row.to_schema();
        if row_schema != self.schema {
            return Err(ClientError::SchemaMismatch {
                expected: Box::new(self.schema.clone()),
                got: Box::new(row_schema),
            });
        }
        let columns = self
            .schema
            .columns()
            .iter()
            .map(|(name, _)| DatasetArray::from(row.0[name].clone()).into())
            .collect();
        let batch = RecordBatch::try_new(self.arrow_schema.clone(), columns)?;
        if tx.send(StreamMessage::Batch(batch)).await.is_ok() {
            Ok(())
        } else {
            let connection_handle = self
                .connection_handle
                .take()
                .ok_or(ClientError::ConnectorPanic)?;
            let connection_result = connection_handle
                .await
                .map_err(|_| ClientError::ConnectorPanic)?;
            connection_result?;
            Err(ClientError::WriterClosed)
        }
    }

    #[instrument(skip(self))]
    pub async fn finish(self) -> Result<Dataset, ClientError> {
        self.complete(StreamMessage::Finish).await
    }

    #[instrument(skip(self))]
    pub async fn abort(self) -> Result<Dataset, ClientError> {
        self.complete(StreamMessage::Abort).await
    }

    async fn complete(mut self, message: StreamMessage) -> Result<Dataset, ClientError> {
        let tx = self.tx.take().ok_or(ClientError::AlreadyFinished)?;
        let _ = tx.send(message).await;
        drop(tx);

        let connection_handle = self
            .connection_handle
            .take()
            .ok_or(ClientError::AlreadyFinished)?;
        let response = connection_handle
            .await
            .map_err(|_| ClientError::ConnectorPanic)??;
        let dataset = response.dataset.ok_or(ClientError::MissingResponse)?;
        let record: DatasetRecord = dataset.try_into().map_err(ClientError::ProtoConversion)?;
        info!(dataset.id = record.id, "Dataset write finished");
        Ok(Dataset {
            client: self.client.clone(),
            record,
        })
    }

    #[must_use]
    pub fn schema(&self) -> &DatasetSchema {
        &self.schema
    }
}

impl Drop for DatasetWriter {
    fn drop(&mut self) {
        let Some(tx) = self.tx.take() else {
            return;
        };

        warn!("DatasetWriter dropped without finish/abort; sending abort");
        let _ = tx.try_send(StreamMessage::Abort);
        drop(tx);

        let Some(connection_handle) = self.connection_handle.take() else {
            return;
        };

        self.runtime.spawn(async move {
            match connection_handle.await {
                Ok(Ok(response)) => {
                    if let Some(dataset) = response.dataset {
                        debug!(dataset.id = dataset.id, "Dataset stream aborted on drop");
                    } else {
                        debug!("Dataset stream aborted on drop");
                    }
                }
                Ok(Err(error)) => {
                    debug!(error = %error, "Dataset stream drop cleanup ended with connection error");
                }
                Err(error) => {
                    debug!(error = %error, "Dataset stream drop cleanup task failed");
                }
            }
        });
    }
}

fn build_request_stream(
    name: String,
    description: String,
    tags: Vec<String>,
    arrow_schema: SchemaRef,
    mut message_rx: mpsc::Receiver<StreamMessage>,
) -> impl Stream<Item = CreateRequest> {
    stream! {
        yield CreateRequest {
            create_message: Some(CreateMessage::Metadata(CreateMetadata {
                name,
                description,
                tags,
            })),
        };

        let buffer_writer = BytesMut::with_capacity(8192).writer();
        let mut writer = match StreamWriter::try_new(buffer_writer, &arrow_schema) {
            Ok(writer) => writer,
            Err(e) => {
                error!(error = %e, "Failed to initialize dataset stream writer");
                yield CreateRequest {
                    create_message: Some(CreateMessage::Abort(CreateAbort {})),
                };
                return;
            }
        };

        let schema_chunk = writer.get_mut().get_mut().split().freeze();
        for payload_chunk in split_payload_chunk(schema_chunk) {
            yield CreateRequest {
                create_message: Some(CreateMessage::Payload(payload_chunk)),
            };
        }

        while let Some(message) = message_rx.recv().await {
            match message {
                StreamMessage::Batch(batch) => {
                    if let Err(e) = writer.write(&batch) {
                        error!(error = %e, "Failed to write batch to dataset stream");
                        yield CreateRequest {
                            create_message: Some(CreateMessage::Abort(CreateAbort {})),
                        };
                        return;
                    }
                    let chunk = writer.get_mut().get_mut().split().freeze();
                    for payload_chunk in split_payload_chunk(chunk) {
                        yield CreateRequest {
                            create_message: Some(CreateMessage::Payload(payload_chunk)),
                        };
                    }
                }
                StreamMessage::Finish => {
                    if let Err(e) = writer.finish() {
                        error!(error = %e, "Failed to finish dataset stream writer");
                        yield CreateRequest {
                            create_message: Some(CreateMessage::Abort(CreateAbort {})),
                        };
                        return;
                    }
                    let eos_chunk = writer.get_mut().get_mut().split().freeze();
                    for payload_chunk in split_payload_chunk(eos_chunk) {
                        yield CreateRequest {
                            create_message: Some(CreateMessage::Payload(payload_chunk)),
                        };
                    }
                    yield CreateRequest {
                        create_message: Some(CreateMessage::Finish(CreateFinish {})),
                    };
                    return;
                }
                StreamMessage::Abort => {
                    yield CreateRequest {
                        create_message: Some(CreateMessage::Abort(CreateAbort {})),
                    };
                    return;
                }
            }
        }

        warn!("Dataset stream closed without finish/abort; sending abort");
        yield CreateRequest {
            create_message: Some(CreateMessage::Abort(CreateAbort {})),
        };
    }
}

fn split_payload_chunk(chunk: Bytes) -> impl Iterator<Item = Bytes> {
    let mut remaining = chunk;
    std::iter::from_fn(move || {
        if remaining.is_empty() {
            None
        } else {
            let size = remaining.len().min(MAX_PAYLOAD_CHUNK_SIZE);
            Some(remaining.split_to(size))
        }
    })
}

async fn connect_ipc_channel(path: PathBuf) -> Result<Channel, ClientError> {
    let channel = Channel::from_static("https://ignored.com:50051")
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = ipc::connect(path).await?;
                Ok::<_, ConnectError>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|error| {
            if connect_target_missing(&error) {
                ClientError::NotRunning
            } else {
                ClientError::Transport(error)
            }
        })?;
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
    pub const fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.record.metadata.deleted_at
    }

    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.record.metadata.deleted_at.is_some()
    }

    #[must_use]
    pub fn status(&self) -> DatasetStatus {
        self.record.metadata.status
    }

    pub async fn add_tags(&self, tags: Vec<String>) -> Result<(), ClientError> {
        let request = AddTagsRequest {
            id: self.record.id,
            tags,
        };
        let _response = self.client.dataset_service().add_tags(request).await?;
        Ok(())
    }

    pub async fn remove_tags(&self, tags: Vec<String>) -> Result<(), ClientError> {
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
    ) -> Result<(), ClientError> {
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
async fn check_server_version(channel: Channel) -> Result<(), ClientError> {
    let request = VersionRequest {};
    let response = FriconServiceClient::new(channel).version(request).await?;
    let server_version = response.into_inner().version;
    let server_version: Version = server_version.parse()?;
    let client_version: Version = VERSION.parse()?;
    if client_version != server_version {
        return Err(ClientError::VersionMismatch {
            server: server_version,
            client: client_version,
        });
    }
    debug!(server_version = %server_version, client_version = %client_version, "Server version check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Int64Array, RecordBatch};
    use arrow_schema::{DataType, Field, Schema};
    use bytes::Bytes;
    use futures::StreamExt;
    use itertools::Itertools;
    use tokio::sync::mpsc;

    use super::{MAX_PAYLOAD_CHUNK_SIZE, StreamMessage, build_request_stream, split_payload_chunk};
    use crate::proto::create_request::CreateMessage;

    fn one_col_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1_i64]))]).expect("batch")
    }

    #[tokio::test]
    async fn build_request_stream_sends_finish_message_on_finish() {
        let (message_tx, message_rx) = mpsc::channel(2);
        message_tx
            .send(StreamMessage::Batch(one_col_batch()))
            .await
            .expect("send batch");
        message_tx
            .send(StreamMessage::Finish)
            .await
            .expect("send finish");
        drop(message_tx);
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        let stream = build_request_stream(
            "dataset".to_string(),
            "desc".to_string(),
            vec!["tag".to_string()],
            schema,
            message_rx,
        );

        let messages: Vec<_> = stream
            .map(|req| req.create_message.expect("message"))
            .collect()
            .await;

        assert!(matches!(messages[0], CreateMessage::Metadata(_)));
        assert!(matches!(messages[1], CreateMessage::Payload(_)));
        assert!(matches!(
            messages[messages.len() - 1],
            CreateMessage::Finish(_)
        ));
    }

    #[tokio::test]
    async fn build_request_stream_sends_abort_message_on_abort() {
        let (message_tx, message_rx) = mpsc::channel(2);
        message_tx
            .send(StreamMessage::Batch(one_col_batch()))
            .await
            .expect("send batch");
        message_tx
            .send(StreamMessage::Abort)
            .await
            .expect("send abort");
        drop(message_tx);
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        let stream = build_request_stream(
            "dataset".to_string(),
            "desc".to_string(),
            vec![],
            schema,
            message_rx,
        );

        let messages: Vec<_> = stream
            .map(|req| req.create_message.expect("message"))
            .collect()
            .await;

        assert!(matches!(messages[0], CreateMessage::Metadata(_)));
        assert!(matches!(messages[1], CreateMessage::Payload(_)));
        assert!(matches!(
            messages[messages.len() - 1],
            CreateMessage::Abort(_)
        ));
    }

    #[tokio::test]
    async fn build_request_stream_sends_abort_when_channel_closes_without_terminal() {
        let (message_tx, message_rx) = mpsc::channel(2);
        message_tx
            .send(StreamMessage::Batch(one_col_batch()))
            .await
            .expect("send batch");
        drop(message_tx);
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        let stream = build_request_stream(
            "dataset".to_string(),
            "desc".to_string(),
            vec![],
            schema,
            message_rx,
        );

        let messages: Vec<_> = stream
            .map(|req| req.create_message.expect("message"))
            .collect()
            .await;

        assert!(matches!(messages[0], CreateMessage::Metadata(_)));
        assert!(matches!(messages[1], CreateMessage::Payload(_)));
        assert!(matches!(
            messages[messages.len() - 1],
            CreateMessage::Abort(_)
        ));
    }

    #[test]
    fn split_payload_chunk_limits_each_piece_to_1mb() {
        let payload = Bytes::from(vec![0_u8; MAX_PAYLOAD_CHUNK_SIZE * 2 + 17]);
        let chunks = split_payload_chunk(payload).collect_vec();

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), MAX_PAYLOAD_CHUNK_SIZE);
        assert_eq!(chunks[1].len(), MAX_PAYLOAD_CHUNK_SIZE);
        assert_eq!(chunks[2].len(), 17);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk.len() <= MAX_PAYLOAD_CHUNK_SIZE)
        );
    }
}

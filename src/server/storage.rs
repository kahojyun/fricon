use std::{collections::HashMap, sync::Mutex};

use anyhow::{anyhow, bail, Context};
use arrow::ipc::reader::StreamReader;
use bytes::Bytes;
use chrono::DateTime;
use futures::prelude::*;
use prost_types::Timestamp;
use tokio::runtime::Handle;
use tokio_util::{io::SyncIoBridge, task::TaskTracker};
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    dataset::Info,
    db::{self, DatasetRecord},
    proto::{
        self, data_storage_service_server::DataStorageService, get_request::IdEnum, CreateRequest,
        CreateResponse, GetRequest, GetResponse, ListRequest, ListResponse, WriteRequest,
        WriteResponse, WRITE_TOKEN,
    },
    workspace::Workspace,
};

#[derive(Debug)]
pub struct Storage {
    workspace: Workspace,
    creating: Creating,
    tracker: TaskTracker,
}

#[derive(Debug)]
struct Metadata {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    index: Vec<String>,
}

#[derive(Debug, Default)]
struct Creating(Mutex<HashMap<Uuid, Metadata>>);

impl Storage {
    pub fn new(workspace: Workspace, tracker: TaskTracker) -> Self {
        Self {
            workspace,
            creating: Creating::default(),
            tracker,
        }
    }
}

impl Creating {
    fn insert(&self, token: Uuid, metadata: Metadata) {
        let mut inner = self.0.lock().unwrap();
        inner.insert(token, metadata);
    }

    fn remove(&self, token: &Uuid) -> Option<Metadata> {
        let mut inner = self.0.lock().unwrap();
        inner.remove(token)
    }
}

impl From<DatasetRecord> for proto::Dataset {
    fn from(
        DatasetRecord {
            id,
            info:
                Info {
                    uid,
                    name,
                    description,
                    favorite,
                    index_columns,
                    path,
                    created_at,
                    tags,
                },
        }: DatasetRecord,
    ) -> Self {
        Self {
            id: Some(id),
            uid: Some(uid.simple().to_string()),
            name: Some(name),
            description: Some(description),
            favorite: Some(favorite),
            index_columns,
            path: Some(path.0),
            created_at: {
                Some(Timestamp {
                    seconds: created_at.timestamp(),
                    #[expect(
                        clippy::cast_possible_wrap,
                        reason = "Nanos are always less than 2e9."
                    )]
                    nanos: created_at.timestamp_subsec_nanos() as i32,
                })
            },
            tags,
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(value: proto::Dataset) -> Result<Self, Self::Error> {
        let id = value.id.context("id is required")?;
        let uid = value.uid.context("uid is required")?.parse()?;
        let name = value.name.context("name is required")?;
        let description = value.description.context("description is required")?;
        let favorite = value.favorite.context("favorite is required")?;
        let index_columns = value.index_columns;
        let path = value.path.context("path is required")?;
        let created_at = value.created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(clippy::cast_sign_loss)]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
        let tags = value.tags;
        Ok(Self {
            id,
            info: Info {
                uid,
                name,
                description,
                favorite,
                index_columns,
                path: path.into(),
                created_at,
                tags,
            },
        })
    }
}

#[tonic::async_trait]
impl DataStorageService for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let msg = request.into_inner();
        let metadata = Metadata {
            name: msg
                .name
                .ok_or_else(|| Status::invalid_argument("name is required"))?,
            description: msg.description,
            tags: msg.tags,
            index: msg.index,
        };
        let uuid = Uuid::new_v4();
        trace!("generated uuid: {:?}", uuid);
        self.creating.insert(uuid, metadata);
        let write_token = Some(Bytes::copy_from_slice(uuid.as_bytes()));
        Ok(Response::new(CreateResponse { write_token }))
    }

    async fn write(
        &self,
        request: Request<Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>> {
        let token = request
            .metadata()
            .get_bin(WRITE_TOKEN)
            .ok_or_else(|| Status::unauthenticated("write token is required"))?
            .to_bytes()
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let token = Uuid::from_slice(&token)
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let metadata = self
            .creating
            .remove(&token)
            .ok_or_else(|| Status::invalid_argument("invalid write token"))?;
        let name = metadata.name;
        let description = metadata.description.unwrap_or_default();
        let tags = metadata.tags;
        let index = metadata.index;
        let in_stream = request.into_inner();
        let workspace = self.workspace.clone();
        // TODO: Check error handling
        let writer_task = self.tracker.spawn_blocking(move || {
            let bytes_stream = in_stream
                .map(|msg| match msg {
                    Ok(WriteRequest { chunk: Some(chunk) }) => Ok(chunk),
                    Ok(WriteRequest { chunk: None }) => Err(anyhow!("Invalid chunk")),
                    Err(e) => Err(e.into()),
                })
                .map_err(|e| {
                    error!("Client connection error: {:?}", e);
                    std::io::Error::new(std::io::ErrorKind::Other, e)
                });
            let reader = SyncIoBridge::new(tokio_util::io::StreamReader::new(bytes_stream));
            let mut reader = StreamReader::try_new(reader, None)?;
            let Some(batch) = reader.next().transpose()? else {
                bail!("No data received.");
            };
            let handle = Handle::current();
            let mut writer = handle.block_on(workspace.create_dataset(
                name,
                description,
                tags,
                index,
                &batch.schema(),
            ))?;
            writer.write(batch)?;
            for batch in reader {
                let batch = match batch {
                    Ok(batch) => batch,
                    Err(e) => {
                        error!("Failed to read ipc stream from client: {:?}", e);
                        if let Err(e) = writer.finish() {
                            error!("Failed to finish writing ipc file: {:?}", e);
                        }
                        return Err(e.into());
                    }
                };
                writer.write(batch)?;
            }
            writer.finish()
        });
        let dataset = writer_task
            .await
            .map_err(|e| {
                error!("writer task panicked: {:?}", e);
                Status::internal(e.to_string())
            })?
            .map_err(|e| {
                error!("write failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        let id = Some(dataset.id());
        Ok(Response::new(WriteResponse { id }))
    }

    async fn list(
        &self,
        _request: tonic::Request<ListRequest>,
    ) -> Result<tonic::Response<ListResponse>, tonic::Status> {
        let dataset_index = self.workspace.dataset_index();
        let records = dataset_index.list_all().await.map_err(|e| {
            error!("Failed to list datasets: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let datasets = records.into_iter().map(Into::into).collect();
        Ok(Response::new(ListResponse { datasets }))
    }

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        let dataset_index = self.workspace.dataset_index();
        let id = request.into_inner().id_enum.ok_or_else(|| {
            error!("id_enum is required");
            Status::invalid_argument("id_enum is required")
        })?;
        let record = match id {
            IdEnum::Id(id) => dataset_index.get_by_id(id).await,
            IdEnum::Uid(uid) => {
                let uid: Uuid = uid.parse().map_err(|e| {
                    error!("Failed to parse uid: {:?}", e);
                    Status::invalid_argument("invalid uid")
                })?;
                dataset_index.get_by_uid(uid).await
            }
        }
        .map_err(|e| {
            if matches!(e, db::Error::NotFound) {
                Status::not_found("dataset not found")
            } else {
                error!("Failed to get dataset: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        let dataset = Some(record.into());
        Ok(Response::new(GetResponse { dataset }))
    }
}

use std::{collections::HashMap, sync::Mutex};

use anyhow::{Context, bail};
use arrow::ipc::reader::StreamReader;
use bytes::Bytes;
use chrono::DateTime;
use futures::prelude::*;
use prost_types::Timestamp;
use tokio_util::{io::SyncIoBridge, task::TaskTracker};
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    database::{self, DatasetRecord},
    dataset,
    paths::DatasetPath,
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DatasetMetadata,
        DeleteRequest, DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest,
        RemoveTagsResponse, SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        WriteRequest, WriteResponse, data_storage_service_server::DataStorageService,
        get_request::IdEnum,
    },
    workspace::Workspace,
};

#[derive(Debug)]
pub struct Storage {
    workspace: Workspace,
    pending_create: PendingCreate,
    tracker: TaskTracker,
}

#[derive(Debug, Default)]
struct PendingCreate(Mutex<HashMap<Uuid, CreateRequest>>);

impl Storage {
    pub fn new(workspace: Workspace, tracker: TaskTracker) -> Self {
        Self {
            workspace,
            pending_create: PendingCreate::default(),
            tracker,
        }
    }
}

impl PendingCreate {
    fn insert(&self, token: Uuid, request: CreateRequest) {
        let mut inner = self.0.lock().unwrap();
        inner.insert(token, request);
    }

    fn remove(&self, token: &Uuid) -> Option<CreateRequest> {
        let mut inner = self.0.lock().unwrap();
        inner.remove(token)
    }
}

impl From<dataset::Metadata> for proto::DatasetMetadata {
    fn from(
        dataset::Metadata {
            uid,
            name,
            description,
            favorite,
            index_columns,
            created_at,
            tags,
        }: dataset::Metadata,
    ) -> Self {
        let created_at = Timestamp {
            seconds: created_at.timestamp(),
            #[expect(clippy::cast_possible_wrap, reason = "Nanos are always less than 2e9.")]
            nanos: created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            uid: uid.simple().to_string(),
            name,
            description,
            favorite,
            index_columns,
            created_at: Some(created_at),
            tags,
        }
    }
}

impl TryFrom<proto::DatasetMetadata> for dataset::Metadata {
    type Error = anyhow::Error;

    fn try_from(
        DatasetMetadata {
            uid,
            name,
            description,
            favorite,
            index_columns,
            created_at,
            tags,
        }: proto::DatasetMetadata,
    ) -> Result<Self, Self::Error> {
        let uid = uid.parse()?;
        let created_at = created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(clippy::cast_sign_loss)]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
        Ok(Self {
            uid,
            name,
            description,
            favorite,
            index_columns,
            created_at,
            tags,
        })
    }
}

impl From<DatasetRecord> for proto::Dataset {
    fn from(DatasetRecord { id, path, metadata }: DatasetRecord) -> Self {
        Self {
            id,
            path: path.0,
            metadata: Some(metadata.into()),
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(
        proto::Dataset { id, path, metadata }: proto::Dataset,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id,
            path: DatasetPath(path),
            metadata: metadata
                .context("metadata field is required.")?
                .try_into()?,
        })
    }
}

// TODO: Use workspace methods
#[tonic::async_trait]
impl DataStorageService for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let request = request.into_inner();
        let uuid = Uuid::new_v4();
        trace!("generated uuid: {:?}", uuid);
        self.pending_create.insert(uuid, request);
        let write_token = Bytes::copy_from_slice(uuid.as_bytes());
        Ok(Response::new(CreateResponse { write_token }))
    }

    async fn write(
        &self,
        request: Request<Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>> {
        let token = request
            .metadata()
            .get_bin(proto::WRITE_TOKEN_KEY)
            .ok_or_else(|| Status::unauthenticated("write token is required"))?
            .to_bytes()
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let token = Uuid::from_slice(&token)
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let CreateRequest {
            name,
            description,
            tags,
            index_columns,
        } = self
            .pending_create
            .remove(&token)
            .ok_or_else(|| Status::invalid_argument("invalid write token"))?;
        let request_stream = request.into_inner();
        let bytes_stream = request_stream.map(|request| {
            request.map(|x| x.chunk).map_err(|e| {
                error!("Client connection error: {e:?}");
                std::io::Error::other(e)
            })
        });
        let async_reader = tokio_util::io::StreamReader::new(bytes_stream);
        let sync_reader = SyncIoBridge::new(async_reader);
        let mut writer = self
            .workspace
            .create_dataset(name, description, tags, index_columns)
            .await
            .map_err(|e| {
                error!("Failed to create dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        // TODO: Check error handling
        let writer_task = self.tracker.spawn_blocking(move || {
            let reader = StreamReader::try_new(sync_reader, None)?;
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
        let id = dataset.id().expect("dataset id should be present");
        let dataset = self.workspace.database().get_by_id(id).await.map_err(|e| {
            error!("Failed to get dataset by id: {:?}", e);
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(WriteResponse {
            dataset: Some(dataset.into()),
        }))
    }

    // TODO: Add search implementation
    async fn search(
        &self,
        _request: tonic::Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, tonic::Status> {
        let dataset_index = self.workspace.database();
        let records = dataset_index.list_all().await.map_err(|e| {
            error!("Failed to list datasets: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let datasets = records.into_iter().map(Into::into).collect();
        Ok(Response::new(SearchResponse {
            datasets,
            ..Default::default()
        }))
    }

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        let dataset_index = self.workspace.database();
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
            if matches!(e, database::Error::NotFound) {
                Status::not_found("dataset not found")
            } else {
                error!("Failed to get dataset: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        let dataset = Some(record.into());
        Ok(Response::new(GetResponse { dataset }))
    }

    async fn add_tags(
        &self,
        request: Request<AddTagsRequest>,
    ) -> Result<Response<AddTagsResponse>> {
        let AddTagsRequest { id, tags } = request.into_inner();
        self.workspace
            .database()
            .add_dataset_tags(id, &tags)
            .await
            .map_err(|e| {
                error!("Failed to add tags: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(AddTagsResponse {}))
    }

    async fn remove_tags(
        &self,
        request: Request<RemoveTagsRequest>,
    ) -> Result<Response<RemoveTagsResponse>> {
        let RemoveTagsRequest { id, tags } = request.into_inner();
        self.workspace
            .database()
            .remove_dataset_tags(id, &tags)
            .await
            .map_err(|e| {
                error!("Failed to remove tags: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(RemoveTagsResponse {}))
    }

    async fn update(&self, request: Request<UpdateRequest>) -> Result<Response<UpdateResponse>> {
        let UpdateRequest {
            id,
            name,
            description,
            favorite,
        } = request.into_inner();
        self.workspace
            .database()
            .update_dataset(id, name.as_deref(), description.as_deref(), favorite)
            .await
            .map_err(|e| {
                error!("Failed to update dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(UpdateResponse {}))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        self.workspace
            .database()
            .delete_dataset(id)
            .await
            .map_err(|e| {
                error!("Failed to delete dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(DeleteResponse {}))
    }
}

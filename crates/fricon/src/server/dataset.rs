use std::{collections::HashMap, sync::Mutex};

use anyhow::{Context, bail};
use arrow::ipc::reader::StreamReader;
use bytes::Bytes;
use chrono::DateTime;
use futures::prelude::*;
use prost_types::Timestamp;
use tokio_util::io::SyncIoBridge;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    app::AppHandle,
    database::{self, DatasetUpdate},
    dataset,
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DatasetMetadata,
        DeleteRequest, DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest,
        RemoveTagsResponse, SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        WriteRequest, WriteResponse, dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

pub struct Storage {
    app: AppHandle,
    pending_create: PendingCreate,
}

#[derive(Debug, Default)]
struct PendingCreate(Mutex<HashMap<Uuid, CreateRequest>>);

impl Storage {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            pending_create: PendingCreate::default(),
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
            uuid,
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
            uuid: uuid.simple().to_string(),
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
            uuid,
            name,
            description,
            favorite,
            index_columns,
            created_at,
            tags,
        }: proto::DatasetMetadata,
    ) -> Result<Self, Self::Error> {
        let uuid = uuid.parse()?;
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
            uuid,
            name,
            description,
            favorite,
            index_columns,
            created_at,
            tags,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: dataset::Metadata,
}

impl From<(database::Dataset, Vec<database::Tag>)> for DatasetRecord {
    fn from(
        (
            database::Dataset {
                id,
                uuid,
                name,
                description,
                favorite,
                index_columns,
                created_at,
            },
            tags,
        ): (database::Dataset, Vec<database::Tag>),
    ) -> Self {
        Self {
            id,
            metadata: dataset::Metadata {
                uuid: uuid.0,
                name,
                description,
                favorite,
                index_columns: index_columns.0,
                created_at: created_at.and_utc(),
                tags: tags.into_iter().map(|tag| tag.name).collect(),
            },
        }
    }
}

impl From<dataset::Dataset> for DatasetRecord {
    fn from(dataset: dataset::Dataset) -> Self {
        Self {
            id: dataset.id(),
            metadata: dataset.metadata(),
        }
    }
}

impl From<DatasetRecord> for proto::Dataset {
    fn from(DatasetRecord { id, metadata }: DatasetRecord) -> Self {
        Self {
            id,
            metadata: Some(metadata.into()),
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(proto::Dataset { id, metadata }: proto::Dataset) -> Result<Self, Self::Error> {
        Ok(Self {
            id,
            metadata: metadata
                .context("metadata field is required.")?
                .try_into()?,
        })
    }
}

// TODO: Use workspace methods
#[tonic::async_trait]
impl DatasetService for Storage {
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
            .app
            .create_dataset(name, description, tags, index_columns)
            .await
            .map_err(|e| {
                error!("Failed to create dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        // TODO: Check error handling
        let writer_task = self.app.tracker().spawn_blocking(move || {
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
        let record = DatasetRecord::from(dataset);
        Ok(Response::new(WriteResponse {
            dataset: Some(record.into()),
        }))
    }

    // TODO: Add search implementation
    async fn search(
        &self,
        _request: tonic::Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, tonic::Status> {
        let records = self.app.list_datasets().await.map_err(|e| {
            error!("Failed to list datasets: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let datasets = records
            .into_iter()
            .map(Into::<DatasetRecord>::into)
            .map(Into::<proto::Dataset>::into)
            .collect();
        Ok(Response::new(SearchResponse {
            datasets,
            ..Default::default()
        }))
    }

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        let id = request.into_inner().id_enum.ok_or_else(|| {
            error!("id_enum is required");
            Status::invalid_argument("id_enum is required")
        })?;
        let dataset = match id {
            IdEnum::Id(id) => self.app.get_dataset(id).await,
            IdEnum::Uuid(uuid) => {
                let uuid: Uuid = uuid.parse().map_err(|e| {
                    error!("Failed to parse uuid: {:?}", e);
                    Status::invalid_argument("invalid uuid")
                })?;
                self.app.get_dataset_by_uuid(uuid).await
            }
        }
        .map_err(|e| {
            if let Some(diesel::result::Error::NotFound) = e.downcast_ref() {
                Status::not_found("dataset not found")
            } else {
                error!("Failed to get dataset: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        let record = DatasetRecord::from(dataset);
        Ok(Response::new(GetResponse {
            dataset: Some(record.into()),
        }))
    }

    async fn add_tags(
        &self,
        request: Request<AddTagsRequest>,
    ) -> Result<Response<AddTagsResponse>> {
        let AddTagsRequest { id, tags } = request.into_inner();
        let mut dataset = self.app.get_dataset(id).await.map_err(|e| {
            error!("Failed to get dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        dataset.add_tags(tags).await.map_err(|e| {
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
        let mut dataset = self.app.get_dataset(id).await.map_err(|e| {
            error!("Failed to get dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        dataset.remove_tags(tags).await.map_err(|e| {
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
        let mut dataset = self.app.get_dataset(id).await.map_err(|e| {
            error!("Failed to get dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        dataset
            .update_info(DatasetUpdate {
                name,
                description,
                favorite,
            })
            .await
            .map_err(|e| {
                error!("Failed to update dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(UpdateResponse {}))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        let mut dataset = self.app.get_dataset(id).await.map_err(|e| {
            error!("Failed to get dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        dataset.delete().await.map_err(|e| {
            error!("Failed to delete dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(DeleteResponse {}))
    }
}

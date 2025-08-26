use std::{collections::HashMap, sync::Mutex};

use anyhow::{Context, bail};
use arrow::ipc::reader::StreamReader;
use bytes::Bytes;
use chrono::DateTime;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use futures::prelude::*;
use prost_types::Timestamp;
use tokio_util::io::SyncIoBridge;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::database::{DatasetStatus, PoolExt};

use crate::{
    app::AppHandle,
    database::{self},
    dataset,
    dataset_manager::{DatasetId, DatasetManager},
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DatasetMetadata,
        DeleteRequest, DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest,
        RemoveTagsResponse, SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        WriteRequest, WriteResponse, dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

pub struct Storage {
    app: AppHandle,
    manager: DatasetManager,
    pending_create: PendingCreate,
}

#[derive(Debug, Default)]
struct PendingCreate(Mutex<HashMap<Uuid, CreateRequest>>);

impl Storage {
    pub fn new(app: AppHandle) -> Self {
        Self {
            manager: DatasetManager::new(app.clone()),
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

/// Convert from Rust `DatasetStatus` to protobuf `DatasetStatus`
impl From<DatasetStatus> for proto::DatasetStatus {
    fn from(status: DatasetStatus) -> Self {
        match status {
            DatasetStatus::Pending => proto::DatasetStatus::Pending,
            DatasetStatus::Writing => proto::DatasetStatus::Writing,
            DatasetStatus::Completed => proto::DatasetStatus::Completed,
            DatasetStatus::Aborted => proto::DatasetStatus::Aborted,
        }
    }
}

/// Convert from protobuf `DatasetStatus` to Rust `DatasetStatus`
impl TryFrom<proto::DatasetStatus> for DatasetStatus {
    type Error = anyhow::Error;

    fn try_from(status: proto::DatasetStatus) -> Result<Self, Self::Error> {
        match status {
            proto::DatasetStatus::Unspecified => bail!("Cannot convert unspecified dataset status"),
            proto::DatasetStatus::Pending => Ok(DatasetStatus::Pending),
            proto::DatasetStatus::Writing => Ok(DatasetStatus::Writing),
            proto::DatasetStatus::Completed => Ok(DatasetStatus::Completed),
            proto::DatasetStatus::Aborted => Ok(DatasetStatus::Aborted),
        }
    }
}

impl From<dataset::Metadata> for proto::DatasetMetadata {
    fn from(
        dataset::Metadata {
            uuid,
            name,
            description,
            favorite,
            status,
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
            status: proto::DatasetStatus::from(status) as i32,
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
            status,
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
        let proto_status =
            proto::DatasetStatus::try_from(status).context("Invalid dataset status")?;
        let status = DatasetStatus::try_from(proto_status)?;
        Ok(Self {
            uuid,
            name,
            description,
            favorite,
            status,
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
                status,
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
                status,
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
        // Update status to Writing before starting
        let dataset_id = writer.id();
        let app_clone = self.app.clone();
        let _ = app_clone
            .database()
            .interact(move |conn| {
                use crate::database::schema::datasets::dsl::datasets;
                Ok(diesel::update(datasets.find(dataset_id))
                    .set(crate::database::schema::datasets::status.eq("writing"))
                    .execute(conn))
            })
            .await;

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
        let result = writer_task.await.map_err(|e| {
            error!("writer task panicked: {:?}", e);
            Status::internal(e.to_string())
        })?;

        let dataset = match result {
            Ok(dataset) => {
                // Update status to Completed on success
                let dataset_id = dataset.id();
                let app_clone = self.app.clone();
                let _ = app_clone
                    .database()
                    .interact(move |conn| {
                        use crate::database::schema::datasets::dsl::datasets;
                        Ok(diesel::update(datasets.find(dataset_id))
                            .set(crate::database::schema::datasets::status.eq("completed"))
                            .execute(conn))
                    })
                    .await;
                dataset
            }
            Err(e) => {
                // Update status to Aborted on failure
                let app_clone = self.app.clone();
                let _ = app_clone
                    .database()
                    .interact(move |conn| {
                        use crate::database::schema::datasets::dsl::datasets;
                        Ok(diesel::update(datasets.find(dataset_id))
                            .set(crate::database::schema::datasets::status.eq("aborted"))
                            .execute(conn))
                    })
                    .await;
                error!("write failed: {:?}", e);
                return Err(Status::internal(e.to_string()));
            }
        };
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
        let records = self.manager.list_datasets().await.map_err(|e| {
            error!("Failed to list datasets: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let datasets = records
            .into_iter()
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
        let dataset_id = match id {
            IdEnum::Id(id) => DatasetId::Id(id),
            IdEnum::Uuid(uuid) => {
                let uuid: Uuid = uuid.parse().map_err(|e| {
                    error!("Failed to parse uuid: {:?}", e);
                    Status::invalid_argument("invalid uuid")
                })?;
                DatasetId::Uuid(uuid)
            }
        };
        let record = self.manager.get_dataset(dataset_id).await.map_err(|e| {
            // Check if it's a not found error
            if let crate::dataset_manager::DatasetManagerError::Database { source } = &e {
                if let Some(diesel::result::Error::NotFound) = source.downcast_ref() {
                    Status::not_found("dataset not found")
                } else {
                    error!("Failed to get dataset: {:?}", e);
                    Status::internal(e.to_string())
                }
            } else {
                error!("Failed to get dataset: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        Ok(Response::new(GetResponse {
            dataset: Some(record.into()),
        }))
    }

    async fn add_tags(
        &self,
        request: Request<AddTagsRequest>,
    ) -> Result<Response<AddTagsResponse>> {
        let AddTagsRequest { id, tags } = request.into_inner();
        self.manager.add_tags(id, tags).await.map_err(|e| {
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
        self.manager.remove_tags(id, tags).await.map_err(|e| {
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
        let update = crate::dataset_manager::DatasetUpdate {
            name,
            description,
            favorite,
        };
        self.manager.update_dataset(id, update).await.map_err(|e| {
            error!("Failed to update dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(UpdateResponse {}))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        self.manager.delete_dataset(id).await.map_err(|e| {
            error!("Failed to delete dataset: {:?}", e);
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(DeleteResponse {}))
    }
}

use anyhow::bail;
use arrow::{array::RecordBatch, error::ArrowError, ipc::reader::StreamReader};
use bytes::Bytes;
use futures::prelude::*;
use tokio::sync::mpsc;
use tokio_util::io::SyncIoBridge;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    app::AppHandle,
    database::DatasetStatus,
    dataset_manager::{CreateDatasetRequest, DatasetId, DatasetManager, DatasetManagerError},
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DeleteRequest,
        DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest, RemoveTagsResponse,
        SearchRequest, SearchResponse, UpdateRequest, UpdateResponse, WriteRequest, WriteResponse,
        dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

impl From<crate::dataset_manager::DatasetRecord> for crate::proto::Dataset {
    fn from(record: crate::dataset_manager::DatasetRecord) -> Self {
        Self {
            id: record.id,
            metadata: Some(record.metadata.into()),
        }
    }
}

impl TryFrom<crate::proto::Dataset> for crate::dataset_manager::DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(dataset: crate::proto::Dataset) -> Result<Self, Self::Error> {
        use anyhow::Context;
        Ok(Self {
            id: dataset.id,
            metadata: dataset
                .metadata
                .context("metadata field is required")?
                .try_into()?,
        })
    }
}

impl From<crate::dataset_manager::DatasetMetadata> for crate::proto::DatasetMetadata {
    fn from(metadata: crate::dataset_manager::DatasetMetadata) -> Self {
        use prost_types::Timestamp;
        let created_at = Timestamp {
            seconds: metadata.created_at.timestamp(),
            #[expect(clippy::cast_possible_wrap, reason = "Nanos are always less than 2e9.")]
            nanos: metadata.created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            uuid: metadata.uuid.simple().to_string(),
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            index_columns: metadata.index_columns,
            created_at: Some(created_at),
            tags: metadata.tags,
            status: crate::proto::DatasetStatus::from(metadata.status) as i32,
        }
    }
}

impl TryFrom<crate::proto::DatasetMetadata> for crate::dataset_manager::DatasetMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: crate::proto::DatasetMetadata) -> Result<Self, Self::Error> {
        use anyhow::{Context, bail};
        use chrono::DateTime;

        let uuid = metadata.uuid.parse()?;
        let created_at = metadata.created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(clippy::cast_sign_loss)]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
        let proto_status = crate::proto::DatasetStatus::try_from(metadata.status)
            .context("Invalid dataset status")?;
        let status = DatasetStatus::try_from(proto_status)?;

        Ok(Self {
            uuid,
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            status,
            index_columns: metadata.index_columns,
            created_at,
            tags: metadata.tags,
        })
    }
}

pub struct Storage {
    manager: DatasetManager,
}

impl Storage {
    pub fn new(app: AppHandle) -> Self {
        Self {
            manager: DatasetManager::new(app),
        }
    }
}

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

#[tonic::async_trait]
impl DatasetService for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let CreateRequest {
            name,
            description,
            tags,
            index_columns,
        } = request.into_inner();

        let create_request = CreateDatasetRequest {
            name,
            description,
            tags,
            index_columns,
        };

        let token = self
            .manager
            .create_dataset(create_request)
            .await
            .map_err(|e| {
                error!("Failed to create dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;

        trace!("generated uuid: {:?}", token);
        let write_token = Bytes::copy_from_slice(token.as_bytes());
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

        let request_stream = request.into_inner();
        let bytes_stream = request_stream.map(|request| {
            request.map(|x| x.chunk).map_err(|e| {
                error!("Client connection error: {e:?}");
                std::io::Error::other(e)
            })
        });
        let async_reader = tokio_util::io::StreamReader::new(bytes_stream);
        let sync_reader = SyncIoBridge::new(async_reader);

        let (batch_tx, batch_rx) =
            mpsc::channel::<Result<RecordBatch, arrow::error::ArrowError>>(16);
        let batch_stream = tokio_stream::wrappers::ReceiverStream::new(batch_rx);

        let read_task = self.manager.app().tracker().spawn(async move {
            let result = {
                let batch_tx = batch_tx.clone();
                tokio::task::spawn_blocking(move || {
                    let reader = StreamReader::try_new(sync_reader, None)?;
                    for batch_result in reader {
                        let batch = batch_result?;
                        if batch_tx.blocking_send(Ok(batch)).is_err() {
                            break;
                        }
                    }
                    Ok::<_, ArrowError>(())
                })
                .await
            };

            match result {
                Ok(Err(e)) => {
                    batch_tx.send(Err(e)).await.ok();
                }
                Err(err) => {
                    batch_tx
                        .send(Err(ArrowError::ExternalError(Box::new(err))))
                        .await
                        .ok();
                }
                _ => {}
            }
        });

        let write_result = self.manager.write_dataset(token, batch_stream).await;

        if let Err(e) = read_task.await {
            error!("Read task failed: {:?}", e);
        }

        let record = write_result.map_err(|e| {
            error!("Failed to write dataset: {:?}", e);
            match e {
                DatasetManagerError::InvalidToken => {
                    Status::invalid_argument("invalid or expired write token")
                }
                DatasetManagerError::NotWritable { status } => Status::failed_precondition(
                    format!("dataset not writable: status is {status:?}"),
                ),
                _ => Status::internal(e.to_string()),
            }
        })?;

        Ok(Response::new(WriteResponse {
            dataset: Some(record.into()),
        }))
    }

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
            error!("Failed to get dataset: {:?}", e);
            match e {
                DatasetManagerError::NotFound { .. } => Status::not_found("dataset not found"),
                _ => Status::internal(e.to_string()),
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

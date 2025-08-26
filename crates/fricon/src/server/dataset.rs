use anyhow::bail;
use arrow::{array::RecordBatch, ipc::reader::StreamReader};
use bytes::Bytes;
use futures::prelude::*;
use tokio_util::io::SyncIoBridge;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    app::AppHandle,
    database::DatasetStatus,
    dataset_manager::{CreateDatasetRequest, DatasetId, DatasetManager},
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DeleteRequest,
        DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest, RemoveTagsResponse,
        SearchRequest, SearchResponse, UpdateRequest, UpdateResponse, WriteRequest, WriteResponse,
        dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

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

// TODO: Use workspace methods
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

        // Create a RecordBatch stream from the IPC stream using spawn_blocking
        let manager_clone = self.manager.clone();
        let write_result = manager_clone
            .app()
            .tracker()
            .spawn_blocking(
                move || -> Result<Vec<RecordBatch>, arrow::error::ArrowError> {
                    let reader = StreamReader::try_new(sync_reader, None)?;
                    let mut batches = Vec::new();
                    for batch_result in reader {
                        let batch = batch_result?;
                        batches.push(batch);
                    }
                    Ok(batches)
                },
            )
            .await
            .map_err(|e| {
                error!("Failed to read IPC stream: {:?}", e);
                Status::internal(e.to_string())
            })?;

        let batches = write_result.map_err(|e| {
            error!("Failed to parse Arrow batches: {:?}", e);
            Status::internal(e.to_string())
        })?;

        // Convert batches to stream
        let batch_stream =
            futures::stream::iter(batches.into_iter().map(Ok::<_, arrow::error::ArrowError>));

        // Use DatasetManager to write the dataset
        let record = self
            .manager
            .write_dataset(token, batch_stream)
            .await
            .map_err(|e| {
                error!("Failed to write dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
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

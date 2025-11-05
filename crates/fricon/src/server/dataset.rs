use std::io::{Error as IoError, ErrorKind};

use anyhow::bail;
use arrow_ipc::reader::StreamReader;
use futures::prelude::*;
use tokio_util::io::{StreamReader as TokioStreamReader, SyncIoBridge};
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace, warn};
use uuid::Uuid;

use crate::{
    app::AppHandle,
    database::DatasetStatus,
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetManager, DatasetManagerError, DatasetMetadata,
        DatasetRecord, DatasetUpdate,
    },
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateAbort, CreateMetadata, CreateRequest,
        CreateResponse, DeleteRequest, DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest,
        RemoveTagsResponse, SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        create_request::CreateMessage, dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

impl From<DatasetRecord> for proto::Dataset {
    fn from(record: DatasetRecord) -> Self {
        Self {
            id: record.id,
            metadata: Some(record.metadata.into()),
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(dataset: proto::Dataset) -> Result<Self, Self::Error> {
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

impl From<DatasetMetadata> for proto::DatasetMetadata {
    fn from(metadata: DatasetMetadata) -> Self {
        use prost_types::Timestamp;

        let created_at = Timestamp {
            seconds: metadata.created_at.timestamp(),
            #[expect(
                clippy::cast_possible_wrap,
                reason = "Nanos are always less than 2e9 and within i32 range"
            )]
            nanos: metadata.created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            uid: metadata.uid.simple().to_string(),
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            created_at: Some(created_at),
            tags: metadata.tags,
            status: proto::DatasetStatus::from(metadata.status) as i32,
        }
    }
}

impl TryFrom<proto::DatasetMetadata> for DatasetMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: proto::DatasetMetadata) -> Result<Self, Self::Error> {
        use anyhow::{Context, bail};
        use chrono::DateTime;

        let uid = metadata.uid.parse()?;
        let created_at = metadata.created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(
            clippy::cast_sign_loss,
            reason = "Negative values are explicitly checked and rejected above"
        )]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
        let proto_status =
            proto::DatasetStatus::try_from(metadata.status).context("Invalid dataset status")?;
        let status = DatasetStatus::try_from(proto_status)?;

        Ok(Self {
            uid,
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            status,
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
            proto::DatasetStatus::Writing => Ok(DatasetStatus::Writing),
            proto::DatasetStatus::Completed => Ok(DatasetStatus::Completed),
            proto::DatasetStatus::Aborted => Ok(DatasetStatus::Aborted),
        }
    }
}

#[tonic::async_trait]
impl DatasetService for Storage {
    async fn create(
        &self,
        request: Request<Streaming<CreateRequest>>,
    ) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let mut stream = request.into_inner();
        let first_message = stream
            .next()
            .await
            .ok_or_else(|| Status::invalid_argument("request stream is empty"))?
            .map_err(|e| {
                error!("Failed to read first message: {:?}", e);
                Status::internal("failed to read first message")
            })?;
        let Some(CreateMessage::Metadata(CreateMetadata {
            name,
            description,
            tags,
        })) = first_message.create_message
        else {
            error!("First message must be CreateMetadata");
            return Err(Status::invalid_argument(
                "first message must be CreateMetadata",
            ));
        };

        let bytes_stream = stream.map(|request| {
            let request = request.map_err(|e| {
                error!("Client connection error: {e:?}");
                IoError::other(e)
            })?;
            match request.create_message {
                Some(CreateMessage::Payload(data)) => Ok(data),
                Some(CreateMessage::Metadata(_)) => {
                    error!("Unexpected CreateMetadata message after the first message");
                    Err(IoError::new(
                        ErrorKind::InvalidInput,
                        "unexpected CreateMetadata message after the first message",
                    ))
                }
                Some(CreateMessage::Abort(CreateAbort { reason })) => {
                    warn!("Client aborted the upload: {}", reason);
                    Err(IoError::new(
                        ErrorKind::UnexpectedEof,
                        format!("client aborted the upload: {reason}"),
                    ))
                }
                None => {
                    error!("Empty CreateRequest message");
                    Err(IoError::new(
                        ErrorKind::InvalidInput,
                        "empty CreateRequest message",
                    ))
                }
            }
        });
        let sync_reader = SyncIoBridge::new(TokioStreamReader::new(bytes_stream));
        let batch_reader = || {
            StreamReader::try_new(sync_reader, None).map_err(|e| {
                DatasetManagerError::BatchStreamError {
                    message: e.to_string(),
                }
            })
        };
        let create_request = CreateDatasetRequest {
            name,
            description,
            tags,
        };
        let record = self
            .manager
            .create_dataset(create_request, batch_reader)
            .await
            .map_err(|e| {
                error!("Failed to write dataset: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(CreateResponse {
            dataset: Some(record.into()),
        }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let id = request.into_inner().id_enum.ok_or_else(|| {
            error!("id_enum is required");
            Status::invalid_argument("id_enum is required")
        })?;
        let dataset_id = match id {
            IdEnum::Id(id) => DatasetId::Id(id),
            IdEnum::Uid(uid) => {
                let uid: Uuid = uid.parse().map_err(|e| {
                    error!("Failed to parse uid: {:?}", e);
                    Status::invalid_argument("invalid uid")
                })?;
                DatasetId::Uid(uid)
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
        let update = DatasetUpdate {
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

    async fn search(
        &self,
        _request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
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
}

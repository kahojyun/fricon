use anyhow::bail;
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

use super::create_stream;
use crate::{
    database::DatasetStatus,
    dataset_manager::{
        DatasetId, DatasetListQuery, DatasetManager, DatasetManagerError, DatasetMetadata,
        DatasetRecord, DatasetUpdate,
    },
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DeleteRequest,
        DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest, RemoveTagsResponse,
        SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        dataset_service_server::DatasetService, get_request::IdEnum,
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

pub(crate) struct Storage {
    manager: DatasetManager,
    shutdown_token: CancellationToken,
}

impl Storage {
    pub(crate) fn new(manager: DatasetManager, shutdown_token: CancellationToken) -> Self {
        Self {
            manager,
            shutdown_token,
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
    #[instrument(skip_all, fields(rpc.method = "dataset.create"))]
    async fn create(
        &self,
        request: Request<Streaming<CreateRequest>>,
    ) -> Result<Response<CreateResponse>> {
        debug!("RPC create requested");
        let stream = request.into_inner();
        let create =
            create_stream::parse_create_stream(stream, self.shutdown_token.clone()).await?;
        debug!(name = %create.request.name, "RPC create: received dataset stream");
        let record = self
            .manager
            .create_dataset(create.request, create.events_rx)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to write dataset");
                Status::internal(e.to_string())
            })?;
        debug!(
            dataset.id = record.id,
            "RPC create: dataset stored successfully"
        );
        Ok(Response::new(CreateResponse {
            dataset: Some(record.into()),
        }))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.get"))]
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let id = request.into_inner().id_enum.ok_or_else(|| {
            warn!("id_enum is required");
            Status::invalid_argument("id_enum is required")
        })?;
        let dataset_id = match id {
            IdEnum::Id(id) => DatasetId::Id(id),
            IdEnum::Uid(uid) => {
                let uid: Uuid = uid.parse().map_err(|e| {
                    warn!(error = %e, "Failed to parse dataset uid");
                    Status::invalid_argument("invalid uid")
                })?;
                DatasetId::Uid(uid)
            }
        };
        let record = self.manager.get_dataset(dataset_id).await.map_err(|e| {
            error!(error = %e, "Failed to get dataset");
            match e {
                DatasetManagerError::NotFound { .. } => Status::not_found("dataset not found"),
                _ => Status::internal(e.to_string()),
            }
        })?;
        debug!(dataset.id = record.id, "RPC get: dataset retrieved");
        Ok(Response::new(GetResponse {
            dataset: Some(record.into()),
        }))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.add_tags"))]
    async fn add_tags(
        &self,
        request: Request<AddTagsRequest>,
    ) -> Result<Response<AddTagsResponse>> {
        let AddTagsRequest { id, tags } = request.into_inner();
        self.manager.add_tags(id, tags).await.map_err(|e| {
            error!(error = %e, dataset.id = id, "Failed to add tags");
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(AddTagsResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.remove_tags"))]
    async fn remove_tags(
        &self,
        request: Request<RemoveTagsRequest>,
    ) -> Result<Response<RemoveTagsResponse>> {
        let RemoveTagsRequest { id, tags } = request.into_inner();
        self.manager.remove_tags(id, tags).await.map_err(|e| {
            error!(error = %e, dataset.id = id, "Failed to remove tags");
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(RemoveTagsResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.update"))]
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
            error!(error = %e, dataset.id = id, "Failed to update dataset");
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(UpdateResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.delete"))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        self.manager.delete_dataset(id).await.map_err(|e| {
            error!(error = %e, dataset.id = id, "Failed to delete dataset");
            Status::internal(e.to_string())
        })?;
        debug!(dataset.id = id, "RPC delete: dataset deleted");
        Ok(Response::new(DeleteResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.search"))]
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let request = request.into_inner();
        let limit = if request.page_size > 0 {
            Some(i64::from(request.page_size))
        } else {
            None
        };
        let offset = if request.page_token.trim().is_empty() {
            None
        } else {
            Some(
                request
                    .page_token
                    .parse::<i64>()
                    .map_err(|_| Status::invalid_argument("invalid page_token"))?,
            )
        };
        let records = self
            .manager
            .list_datasets(DatasetListQuery {
                limit,
                offset,
                ..DatasetListQuery::default()
            })
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to list datasets");
                Status::internal(e.to_string())
            })?;
        let next_page_token = limit.and_then(|limit| {
            let record_len = i64::try_from(records.len()).unwrap_or(i64::MAX);

            if record_len < limit {
                None
            } else {
                Some(offset.unwrap_or(0).saturating_add(limit).to_string())
            }
        });
        let datasets = records
            .into_iter()
            .map(Into::<proto::Dataset>::into)
            .collect();
        Ok(Response::new(SearchResponse {
            datasets,
            next_page_token: next_page_token.unwrap_or_default(),
        }))
    }
}

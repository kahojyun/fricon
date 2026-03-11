use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

use super::create_stream;
use crate::{
    dataset::{
        catalog::{CatalogError, DatasetCatalogService},
        ingest::DatasetIngestService,
        model::{DatasetId, DatasetListQuery, DatasetUpdate},
    },
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DeleteRequest,
        DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest, RemoveTagsResponse,
        SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

pub(crate) struct Storage {
    catalog: DatasetCatalogService,
    ingest: DatasetIngestService,
    shutdown_token: CancellationToken,
}

impl Storage {
    pub(crate) fn new(
        catalog: DatasetCatalogService,
        ingest: DatasetIngestService,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            catalog,
            ingest,
            shutdown_token,
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
        let record_result = self
            .ingest
            .create_dataset(create.request, create.events_rx)
            .await;
        let producer_result = create.events_task.await.map_err(|e| {
            error!(error = %e, "Create stream event producer task panicked");
            Status::internal("create stream event producer failed unexpectedly")
        })?;

        let record = match record_result {
            Ok(record) => {
                if let Err(status) = producer_result {
                    error!(status.code = ?status.code(), status.message = status.message(), "Create stream event producer failed");
                    return Err(status);
                }
                record
            }
            Err(e) => {
                error!(error = %e, "Failed to write dataset");
                return Err(Status::internal(e.to_string()));
            }
        };
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
        let record = self.catalog.get_dataset(dataset_id).await.map_err(|e| {
            error!(error = %e, "Failed to get dataset");
            match e {
                CatalogError::NotFound { .. } => Status::not_found("dataset not found"),
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
        self.catalog.add_tags(id, tags).await.map_err(|e| {
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
        self.catalog.remove_tags(id, tags).await.map_err(|e| {
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
        self.catalog.update_dataset(id, update).await.map_err(|e| {
            error!(error = %e, dataset.id = id, "Failed to update dataset");
            Status::internal(e.to_string())
        })?;
        Ok(Response::new(UpdateResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.delete"))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        self.catalog.delete_dataset(id).await.map_err(|e| {
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
            .catalog
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

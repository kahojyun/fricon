//! gRPC adapter for dataset RPCs.
//!
//! This module translates between tonic request/response types and the app's
//! dataset operations. It owns request parsing, error/status mapping, and
//! stream assembly for create/search/get/update/delete endpoints.

use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

use super::create_stream;
use crate::{
    app::AppHandle,
    dataset::{
        DatasetId, DatasetListQuery, DatasetUpdate, catalog::CatalogError, ingest::IngestError,
        read::ReadError,
    },
    proto::{
        self, AddTagsRequest, AddTagsResponse, CreateRequest, CreateResponse, DeleteRequest,
        DeleteResponse, GetRequest, GetResponse, RemoveTagsRequest, RemoveTagsResponse,
        SearchRequest, SearchResponse, UpdateRequest, UpdateResponse,
        dataset_service_server::DatasetService, get_request::IdEnum,
    },
};

/// Tonic dataset-service adapter backed by [`AppHandle`].
///
/// This type owns RPC-level request parsing and status mapping while the app
/// layer owns business behavior.
pub(crate) struct Storage {
    app: AppHandle,
    shutdown_token: CancellationToken,
}

impl Storage {
    /// Build the dataset gRPC adapter around the shared application handle.
    pub(crate) fn new(app: AppHandle, shutdown_token: CancellationToken) -> Self {
        Self {
            app,
            shutdown_token,
        }
    }
}

impl From<CatalogError> for Status {
    fn from(error: CatalogError) -> Self {
        match error {
            CatalogError::NotFound { .. } => Status::not_found("dataset not found"),
            CatalogError::EmptyTag | CatalogError::SameTagName | CatalogError::SameSourceTarget => {
                Status::invalid_argument(error.to_string())
            }
            CatalogError::Deleted { .. }
            | CatalogError::NotTrashed
            | CatalogError::StateDropped
            | CatalogError::TaskPanic { .. }
            | CatalogError::TaskCancelled { .. } => Status::failed_precondition(error.to_string()),
            CatalogError::DatasetFs(_)
            | CatalogError::Database(_)
            | CatalogError::Portability(_) => Status::internal("dataset operation failed"),
        }
    }
}

impl From<IngestError> for Status {
    fn from(error: IngestError) -> Self {
        match error {
            IngestError::NotFound { .. } => Status::not_found("dataset not found"),
            IngestError::StateDropped
            | IngestError::TaskPanic { .. }
            | IngestError::TaskCancelled { .. } => Status::failed_precondition(error.to_string()),
            IngestError::Dataset(_) | IngestError::DatasetFs(_) | IngestError::Database(_) => {
                Status::internal("dataset ingestion failed")
            }
        }
    }
}

impl From<ReadError> for Status {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::NotFound { .. } => Status::not_found("dataset not found"),
            ReadError::Deleted { .. }
            | ReadError::EmptyDataset
            | ReadError::StateDropped
            | ReadError::TaskPanic { .. }
            | ReadError::TaskCancelled { .. } => Status::failed_precondition(error.to_string()),
            ReadError::Dataset(_) | ReadError::DatasetFs(_) | ReadError::Database(_) => {
                Status::internal("dataset read failed")
            }
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
            .app
            .create_dataset_from_receiver(create.request, create.events_rx)
            .await;
        let producer_result = create.events_task.await.map_err(|e| {
            error!(error = %e, "Create stream event producer task panicked");
            Status::internal("create stream event producer failed unexpectedly")
        })?;
        let record =
            record_result.inspect_err(|e| error!(error = %e, "Failed to write dataset"))?;
        if let Err(status) = producer_result {
            error!(
                status.code = ?status.code(),
                status.message = status.message(),
                "Create stream event producer failed"
            );
            return Err(status);
        }
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
        let record = self
            .app
            .get_dataset(dataset_id)
            .await
            .inspect_err(|e| error!(error = %e, "Failed to get dataset"))?;
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
        self.app
            .add_dataset_tags(id, tags)
            .await
            .inspect_err(|e| error!(error = %e, dataset.id = id, "Failed to add tags"))?;
        Ok(Response::new(AddTagsResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.remove_tags"))]
    async fn remove_tags(
        &self,
        request: Request<RemoveTagsRequest>,
    ) -> Result<Response<RemoveTagsResponse>> {
        let RemoveTagsRequest { id, tags } = request.into_inner();
        self.app
            .remove_dataset_tags(id, tags)
            .await
            .inspect_err(|e| error!(error = %e, dataset.id = id, "Failed to remove tags"))?;
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
        self.app
            .update_dataset(id, update)
            .await
            .inspect_err(|e| error!(error = %e, dataset.id = id, "Failed to update dataset"))?;
        Ok(Response::new(UpdateResponse {}))
    }

    #[instrument(skip_all, fields(rpc.method = "dataset.delete"))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>> {
        let DeleteRequest { id } = request.into_inner();
        self.app
            .delete_dataset(id)
            .await
            .inspect_err(|e| error!(error = %e, dataset.id = id, "Failed to delete dataset"))?;
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
            .app
            .list_datasets(DatasetListQuery {
                limit,
                offset,
                ..DatasetListQuery::default()
            })
            .await
            .inspect_err(|e| error!(error = %e, "Failed to list datasets"))?;
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

#[cfg(test)]
mod tests {
    use tonic::{Code, Status};

    use crate::{
        database::core::DatabaseError,
        dataset::{
            catalog::CatalogError, ingest::IngestError, read::ReadError, schema::DatasetError,
        },
        transport::grpc::codec::CodecError,
    };

    #[test]
    fn catalog_not_found_maps_to_not_found() {
        let status = Status::from(CatalogError::NotFound {
            id: "42".to_string(),
        });
        assert_eq!(status.code(), Code::NotFound);
        assert_eq!(status.message(), "dataset not found");
    }

    #[test]
    fn catalog_empty_tag_maps_to_invalid_argument() {
        let status = Status::from(CatalogError::EmptyTag);
        assert_eq!(status.code(), Code::InvalidArgument);
        assert_eq!(status.message(), "Tag name must not be empty");
    }

    #[test]
    fn catalog_delete_precondition_maps_to_failed_precondition() {
        let status = Status::from(CatalogError::NotTrashed);
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert_eq!(
            status.message(),
            "Dataset must be moved to trash before permanent deletion"
        );
    }

    #[test]
    fn catalog_internal_failure_maps_to_internal() {
        let status = Status::from(CatalogError::Database(DatabaseError::Query(
            diesel::result::Error::NotFound,
        )));
        assert_eq!(status.code(), Code::Internal);
        assert_eq!(status.message(), "dataset operation failed");
    }

    #[test]
    fn ingest_precondition_maps_to_failed_precondition() {
        let status = Status::from(IngestError::TaskCancelled {
            operation: "joining ingest task",
        });
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert_eq!(
            status.message(),
            "Background task was cancelled while joining ingest task"
        );
    }

    #[test]
    fn ingest_internal_failure_maps_to_internal() {
        let status = Status::from(IngestError::Dataset(DatasetError::SchemaMismatch));
        assert_eq!(status.code(), Code::Internal);
        assert_eq!(status.message(), "dataset ingestion failed");
    }

    #[test]
    fn read_not_found_maps_to_not_found() {
        let status = Status::from(ReadError::NotFound {
            id: "7".to_string(),
        });
        assert_eq!(status.code(), Code::NotFound);
        assert_eq!(status.message(), "dataset not found");
    }

    #[test]
    fn read_deleted_maps_to_failed_precondition() {
        let status = Status::from(ReadError::Deleted {
            id: "9".to_string(),
        });
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert_eq!(
            status.message(),
            "Dataset payload has been permanently deleted: 9"
        );
    }

    #[test]
    fn existing_successful_codec_behavior_remains_unchanged() {
        let error = CodecError::MissingField("dataset");
        assert_eq!(error.to_string(), "Missing required field: dataset");
    }
}

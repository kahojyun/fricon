mod db;
mod proto;

use log::{error, trace};
use sqlx::SqlitePool;
use tonic::{Request, Response, Result};

use self::{
    db::{create, update},
    proto::{
        data_storage_server::DataStorage, CreateRequest, CreateResponse, UpdateMetadataRequest,
        UpdateMetadataResponse,
    },
};

pub use self::proto::data_storage_server::DataStorageServer;

#[derive(Debug)]
pub struct Storage {
    pub pool: SqlitePool,
}

#[tonic::async_trait]
impl DataStorage for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let metadata = request.into_inner().metadata;
        let name = metadata.as_ref().and_then(|x| x.name.as_deref());
        let description = metadata.as_ref().and_then(|x| x.description.as_deref());
        let tags = metadata.as_ref().map(|x| x.tags.as_slice());
        let id = create(name, description, tags, &self.pool)
            .await
            .map_err(|e| {
                error!("create failed: {:?}", e);
                tonic::Status::internal(e.to_string())
            })?;
        Ok(Response::new(CreateResponse { id }))
    }

    async fn update_metadata(
        &self,
        request: Request<UpdateMetadataRequest>,
    ) -> Result<Response<UpdateMetadataResponse>> {
        trace!("update_metadata: {:?}", request);
        let msg = request.into_inner();
        let id = msg.id;
        let metadata = msg.metadata;
        let name = metadata.as_ref().and_then(|x| x.name.as_deref());
        let description = metadata.as_ref().and_then(|x| x.description.as_deref());
        let tags = metadata.as_ref().map(|x| x.tags.as_slice());
        update(id, name, description, tags, &self.pool)
            .await
            .map_err(|e| {
                error!("create failed: {:?}", e);
                tonic::Status::internal(e.to_string())
            })?;
        Ok(Response::new(UpdateMetadataResponse {}))
    }
}

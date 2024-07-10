mod proto;

use std::collections::HashMap;
use std::sync::Mutex;

use log::{error, trace};
use sqlx::SqlitePool;
use tonic::{Request, Response, Result, Status, Streaming};
use uuid::Uuid;

use self::proto::{
    data_storage_server::DataStorage, CreateRequest, CreateResponse, UpdateMetadataRequest,
    UpdateMetadataResponse, WriteRequest, WriteResponse,
};
use crate::db::{create, find_by_uid, update, Error as DbError};

pub use self::proto::data_storage_server::DataStorageServer;

#[derive(Debug)]
pub struct Storage {
    pool: SqlitePool,
    creating: Creating,
}

#[derive(Debug)]
struct Metadata {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Default)]
struct Creating(Mutex<HashMap<Uuid, Metadata>>);

impl Storage {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            creating: Creating::default(),
        }
    }
}

impl Creating {
    fn insert(&self, token: Uuid, metadata: Metadata) {
        let mut inner = self.0.lock().unwrap();
        inner.insert(token, metadata);
    }

    fn remove(&self, token: &Uuid) -> Option<Metadata> {
        let mut inner = self.0.lock().unwrap();
        inner.remove(token)
    }
}

#[tonic::async_trait]
impl DataStorage for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let msg = request.into_inner();
        let metadata = msg
            .metadata
            .ok_or(Status::invalid_argument("metadata is required"))?;
        let metadata = Metadata {
            name: metadata
                .name
                .ok_or(Status::invalid_argument("name is required"))?,
            description: metadata.description,
            tags: metadata.tags,
        };
        let uuid = Uuid::new_v4();
        self.creating.insert(uuid, metadata);
        let write_token = uuid.into();
        Ok(Response::new(CreateResponse { write_token }))
    }

    async fn update_metadata(
        &self,
        request: Request<UpdateMetadataRequest>,
    ) -> Result<Response<UpdateMetadataResponse>> {
        trace!("update_metadata: {:?}", request);
        let msg = request.into_inner();
        let uid = msg.uid;
        let uid = Uuid::try_parse(&uid).map_err(|_| Status::invalid_argument("invalid uid"))?;
        let id = find_by_uid(uid, &self.pool).await.map_err(|e| match e {
            DbError::NotFound => Status::not_found("not found"),
            DbError::Other(e) => {
                error!("find_by_uid failed: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        let metadata = msg.metadata;
        let name = metadata.as_ref().and_then(|x| x.name.as_deref());
        let description = metadata.as_ref().and_then(|x| x.description.as_deref());
        let tags = metadata.as_ref().map(|x| x.tags.as_slice());
        update(id, name, description, tags, &self.pool)
            .await
            .map_err(|e| {
                error!("update failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        Ok(Response::new(UpdateMetadataResponse {}))
    }

    async fn write(
        &self,
        request: Request<Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>> {
        let token = request
            .metadata()
            .get_bin("fricon-token-bin")
            .ok_or(Status::unauthenticated("write token is required"))?
            .as_encoded_bytes();
        let token =
            Uuid::from_slice(token).map_err(|_| Status::invalid_argument("invalid write token"))?;
        let metadata = self
            .creating
            .remove(&token)
            .ok_or(Status::invalid_argument("invalid write token"))?;
        let name = metadata.name.as_str();
        let description = metadata.description.as_deref();
        let tags = metadata.tags.as_slice();
        let uid = Uuid::new_v4();
        let _id = create(uid, name, description, tags, &self.pool)
            .await
            .map_err(|e| {
                error!("create failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        let mut in_stream = request.into_inner();
        tokio::spawn(async move {
            loop {
                let result = in_stream.message().await;
                match result {
                    Ok(Some(_)) => todo!(),
                    Ok(None) => break,
                    Err(e) => error!("write failed: {:?}", e),
                }
            }
        })
        .await
        .map_err(|e| {
            error!("write failed: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let uid = uid.to_string();
        Ok(Response::new(WriteResponse { uid }))
    }
}

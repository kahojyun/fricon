pub mod proto;

use std::{collections::HashMap, sync::Mutex};

use arrow::{array::RecordBatchWriter, ipc::reader::StreamReader};
use chrono::NaiveDate;
use log::{error, trace};
use sqlx::SqlitePool;
use tokio::fs;
use tonic::{Request, Response, Result, Status, Streaming};
use uuid::Uuid;

use self::proto::{
    data_storage_service_server::DataStorageService, CreateRequest, CreateResponse, GetRequest,
    GetResponse, WriteRequest, WriteResponse,
};
use crate::{
    dataset::create as create_dataset,
    db::{create, fetch_by_uid, Error as DbError},
    dir::Workspace,
};

pub use self::proto::data_storage_service_server::DataStorageServiceServer;

#[derive(Debug)]
pub struct Storage {
    workspace: Workspace,
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
    pub fn new(workspace: Workspace, pool: SqlitePool) -> Self {
        Self {
            workspace,
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
impl DataStorageService for Storage {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>> {
        trace!("create: {:?}", request);
        let msg = request.into_inner();
        let metadata = msg
            .metadata
            .ok_or_else(|| Status::invalid_argument("metadata is required"))?;
        let metadata = Metadata {
            name: metadata
                .name
                .ok_or_else(|| Status::invalid_argument("name is required"))?,
            description: metadata.description,
            tags: metadata.tags,
        };
        let uuid = Uuid::new_v4();
        trace!("generated uuid: {:?}", uuid);
        self.creating.insert(uuid, metadata);
        let write_token = uuid.into();
        Ok(Response::new(CreateResponse { write_token }))
    }

    async fn write(
        &self,
        request: Request<Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>> {
        let token = request
            .metadata()
            .get_bin("fricon-token-bin")
            .ok_or_else(|| Status::unauthenticated("write token is required"))?
            .to_bytes()
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let token = Uuid::from_slice(&token)
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let metadata = self
            .creating
            .remove(&token)
            .ok_or_else(|| Status::invalid_argument("invalid write token"))?;
        let name = metadata.name.as_str();
        let description = metadata.description.as_deref().unwrap_or("");
        let tags = metadata.tags.as_slice();
        let date = chrono::Local::now().date_naive();
        let uid = Uuid::new_v4();
        let path = format_dataset_path(date, uid);
        let _id = create(uid, name, description, &path, tags, &self.pool)
            .await
            .map_err(|e| {
                error!("create failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        let dataset_path = self.workspace.root().data_dir().join(path);
        fs::create_dir_all(&dataset_path).await.map_err(|e| {
            error!("create directory failed: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let mut in_stream = request.into_inner();
        tokio::spawn(async move {
            let mut writer = None;
            loop {
                let result = in_stream.message().await;
                match result {
                    Ok(Some(msg)) => {
                        let batch_data = msg.record_batch;
                        let reader = StreamReader::try_new(batch_data.as_slice(), None).unwrap();
                        for batch in reader {
                            let batch = batch.unwrap();
                            if writer.is_none() {
                                writer = Some(create_dataset(&dataset_path, &batch.schema()));
                            }
                            writer.as_mut().unwrap().write(&batch).unwrap();
                        }
                    }
                    Ok(None) => break,
                    Err(e) => error!("write failed: {:?}", e),
                }
            }
            if let Some(writer) = writer {
                writer.close().unwrap();
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

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>> {
        let uid = request.into_inner().uid;
        let uid = Uuid::parse_str(&uid).map_err(|_| Status::invalid_argument("invalid uid"))?;
        let dataset_record = fetch_by_uid(uid, &self.pool).await.map_err(|e| match e {
            DbError::NotFound => Status::not_found("dataset not found"),
            DbError::Other(e) => {
                error!("get failed: {:?}", e);
                Status::internal(e.to_string())
            }
        })?;
        let metadata = proto::Metadata {
            name: Some(dataset_record.name),
            description: Some(dataset_record.description),
            tags: dataset_record.tags,
        };
        let created_at = prost_types::Timestamp {
            seconds: dataset_record.created_at.and_utc().timestamp(),
            nanos: 0,
        };
        let response = GetResponse {
            metadata: Some(metadata),
            created_at: Some(created_at),
            path: dataset_record.path,
        };
        Ok(Response::new(response))
    }
}

fn format_dataset_path(date: NaiveDate, uid: Uuid) -> String {
    format!("{date}/{uid}")
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_format_dataset_path() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let uid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = format_dataset_path(date, uid);
        assert_eq!(path, "2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
    }
}

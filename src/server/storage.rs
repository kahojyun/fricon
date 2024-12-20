use std::{collections::HashMap, sync::Mutex};

use arrow::ipc::reader::StreamReader;
use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tokio_util::task::TaskTracker;
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    db::{self, create, fetch_by_uid},
    fs::{dataset, DatasetPath},
    proto::{
        self, data_storage_service_server::DataStorageService, CreateRequest, CreateResponse,
        GetRequest, GetResponse, WriteRequest, WriteResponse,
    },
    workspace::Workspace,
};

#[derive(Debug)]
pub struct Storage {
    workspace: Workspace,
    pool: SqlitePool,
    creating: Creating,
    tracker: TaskTracker,
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
    pub fn new(workspace: Workspace, pool: SqlitePool, tracker: TaskTracker) -> Self {
        Self {
            workspace,
            pool,
            creating: Creating::default(),
            tracker,
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

    #[expect(clippy::too_many_lines)]
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
        let path = DatasetPath::new(date, uid);
        let _id = create(
            uid,
            name,
            description,
            path.0.to_str().unwrap(),
            tags,
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!("create failed: {:?}", e);
            Status::internal(e.to_string())
        })?;
        let dataset_metadata = dataset::Metadata {
            uid,
            info: dataset::Info {
                name: name.to_string(),
                description: description.to_string(),
                tags: tags.iter().map(std::string::ToString::to_string).collect(),
            },
        };
        let dataset_path = self.workspace.root().data_dir().join(&path);
        let mut in_stream = request.into_inner();
        let (tx, mut rx) = mpsc::channel::<Vec<_>>(128);
        let writer_task = self.tracker.spawn_blocking(move || {
            let mut writer = None;
            while let Some(batch_data) = rx.blocking_recv() {
                let reader = StreamReader::try_new(batch_data.as_slice(), None)?;
                for batch in reader {
                    let batch = batch?;
                    if writer.is_none() {
                        writer = Some(dataset::create_new(
                            &dataset_path,
                            &dataset_metadata,
                            &batch.schema(),
                        )?);
                    }
                    writer.as_mut().unwrap().write(batch)?;
                }
            }
            if let Some(writer) = writer {
                writer.finish()?;
            }
            anyhow::Ok(())
        });
        let connection_task = self.tracker.spawn(async move {
            loop {
                let result = in_stream.message().await;
                match result {
                    Ok(Some(msg)) => {
                        let batch_data = msg.record_batch;
                        tx.send(batch_data).await?;
                    }
                    Ok(None) => break,
                    Err(e) => {
                        error!("write failed: {:?}", e);
                        break;
                    }
                }
            }
            anyhow::Ok(())
        });
        writer_task
            .await
            .map_err(|e| {
                error!("write failed: {:?}", e);
                Status::internal(e.to_string())
            })?
            .map_err(|e| {
                error!("write failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        connection_task
            .await
            .map_err(|e| {
                error!("write failed: {:?}", e);
                Status::internal(e.to_string())
            })?
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
            db::Error::NotFound => Status::not_found("dataset not found"),
            db::Error::Other(e) => {
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

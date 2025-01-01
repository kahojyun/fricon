use std::{collections::HashMap, sync::Mutex};

use anyhow::bail;
use arrow::ipc::reader::StreamReader;
use bytes::Bytes;
use futures::prelude::*;
use tokio::runtime::Handle;
use tokio_util::{io::SyncIoBridge, task::TaskTracker};
use tonic::{Request, Response, Result, Status, Streaming};
use tracing::{error, trace};
use uuid::Uuid;

use crate::{
    proto::{
        data_storage_service_server::DataStorageService, CreateRequest, CreateResponse, GetRequest,
        GetResponse, WriteRequest, WriteResponse, WRITE_TOKEN,
    },
    workspace::Workspace,
};

#[derive(Debug)]
pub struct Storage {
    workspace: Workspace,
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
    pub fn new(workspace: Workspace, tracker: TaskTracker) -> Self {
        Self {
            workspace,
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
        let write_token = Bytes::copy_from_slice(uuid.as_bytes());
        Ok(Response::new(CreateResponse { write_token }))
    }

    async fn write(
        &self,
        request: Request<Streaming<WriteRequest>>,
    ) -> Result<Response<WriteResponse>> {
        let token = request
            .metadata()
            .get_bin(WRITE_TOKEN)
            .ok_or_else(|| Status::unauthenticated("write token is required"))?
            .to_bytes()
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let token = Uuid::from_slice(&token)
            .map_err(|_| Status::invalid_argument("invalid write token"))?;
        let metadata = self
            .creating
            .remove(&token)
            .ok_or_else(|| Status::invalid_argument("invalid write token"))?;
        let name = metadata.name;
        let description = metadata.description.unwrap_or_default();
        let tags = metadata.tags;
        let in_stream = request.into_inner();
        let workspace = self.workspace.clone();
        // TODO: Check error handling
        let writer_task = self.tracker.spawn_blocking(move || {
            let bytes_stream = in_stream
                .map_ok(|WriteRequest { chunk }| chunk)
                .map_err(|e| {
                    error!("Client connection error: {:?}", e);
                    std::io::Error::new(std::io::ErrorKind::Other, e)
                });
            let reader = SyncIoBridge::new(tokio_util::io::StreamReader::new(bytes_stream));
            let mut reader = StreamReader::try_new(reader, None)?;
            let Some(batch) = reader.next().transpose()? else {
                bail!("No data received.");
            };
            let handle = Handle::current();
            let mut writer = handle.block_on(workspace.create_dataset(
                name,
                description,
                tags,
                &batch.schema(),
            ))?;
            writer.write(batch)?;
            for batch in reader {
                let batch = match batch {
                    Ok(batch) => batch,
                    Err(e) => {
                        error!("Failed to read ipc stream from client: {:?}", e);
                        if let Err(e) = writer.finish() {
                            error!("Failed to finish writing ipc file: {:?}", e);
                        }
                        return Err(e.into());
                    }
                };
                writer.write(batch)?;
            }
            writer.finish()
        });
        let dataset = writer_task
            .await
            .map_err(|e| {
                error!("writer task panicked: {:?}", e);
                Status::internal(e.to_string())
            })?
            .map_err(|e| {
                error!("write failed: {:?}", e);
                Status::internal(e.to_string())
            })?;
        let uid = dataset.uid().to_string();
        Ok(Response::new(WriteResponse { uid }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>> {
        let uid = request.into_inner().uid;
        let uid = Uuid::parse_str(&uid).map_err(|_| Status::invalid_argument("invalid uid"))?;
        let dataset = self.workspace.open_dataset(uid).await.map_err(|e| {
            error!("get failed: {:?}", e);
            Status::internal(e.to_string())
        })?;
        todo!();
        // let dataset_record = fetch_by_uid(uid, &self.pool).await.map_err(|e| match e {
        //     db::Error::NotFound => Status::not_found("dataset not found"),
        //     db::Error::Other(e) => {
        //         error!("get failed: {:?}", e);
        //         Status::internal(e.to_string())
        //     }
        // })?;
        // let metadata = proto::Metadata {
        //     name: Some(dataset_record.name),
        //     description: Some(dataset_record.description),
        //     tags: dataset_record.tags,
        // };
        // let created_at = prost_types::Timestamp {
        //     seconds: dataset_record.created_at.and_utc().timestamp(),
        //     nanos: 0,
        // };
        // let response = GetResponse {
        //     metadata: Some(metadata),
        //     created_at: Some(created_at),
        //     path: dataset_record.path,
        // };
        // Ok(Response::new(response))
    }
}

mod fricon;
mod storage;

use std::path::Path;

use anyhow::Result;
use tokio::signal;
use tokio_util::task::TaskTracker;
use tonic::transport::Server;
use tracing::info;

use crate::{
    ipc,
    proto::{
        data_storage_service_server::DataStorageServiceServer,
        fricon_service_server::FriconServiceServer,
    },
    workspace,
};

use self::{fricon::Fricon, storage::Storage};

pub async fn run(path: &Path) -> Result<()> {
    let workspace = workspace::Workspace::open(path).await?;
    let ipc_file = workspace.root().ipc_file();
    let tracker = TaskTracker::new();
    let storage = Storage::new(workspace, tracker.clone());
    let service = DataStorageServiceServer::new(storage);
    let listener = ipc::listen(ipc_file.0)?;
    Server::builder()
        .add_service(service)
        .add_service(FriconServiceServer::new(Fricon))
        .serve_with_incoming_shutdown(listener, async {
            signal::ctrl_c()
                .await
                .expect("Failed to install ctrl-c handler.");
        })
        .await?;
    info!("Shutdown");
    tracker.close();
    tracker.wait().await;
    Ok(())
}

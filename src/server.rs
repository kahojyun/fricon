mod fricon;
mod storage;

use anyhow::Result;
use tokio::signal;
use tokio_util::task::TaskTracker;
use tonic::transport::Server;
use tracing::info;

use crate::{
    proto::{
        data_storage_service_server::DataStorageServiceServer,
        fricon_service_server::FriconServiceServer,
    },
    workspace,
};

use self::{fricon::Fricon, storage::Storage};

pub async fn run(path: std::path::PathBuf) -> Result<()> {
    let workspace = workspace::Workspace::open(path).await?;
    let port = workspace.config().port();
    let tracker = TaskTracker::new();
    let storage = Storage::new(workspace, tracker.clone());
    let service = DataStorageServiceServer::new(storage).max_decoding_message_size(usize::MAX);
    let addr = format!("[::1]:{port}").parse()?;
    info!("Listen on {}", addr);
    Server::builder()
        .add_service(service)
        .add_service(FriconServiceServer::new(Fricon))
        .serve_with_shutdown(addr, async {
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

mod dataset;
mod fricon;

use std::path::PathBuf;

use anyhow::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tonic::transport::Server;
use tracing::info;

use self::{dataset::Storage, fricon::Fricon};
pub use crate::dataset_manager::DatasetRecord;
use crate::{
    app::AppHandle,
    ipc,
    proto::{
        dataset_service_server::DatasetServiceServer, fricon_service_server::FriconServiceServer,
    },
};

pub fn start(
    ipc_file: PathBuf,
    app: &AppHandle,
    task_tracker: &TaskTracker,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let storage = Storage::new(app.dataset_manager(), cancellation_token.clone());
    let service = DatasetServiceServer::new(storage);
    let listener = ipc::listen(ipc_file)?;

    info!("Starting gRPC server");
    task_tracker.spawn(async move {
        Server::builder()
            .add_service(service)
            .add_service(FriconServiceServer::new(Fricon))
            .serve_with_incoming_shutdown(listener, async {
                cancellation_token.cancelled().await;
                info!("Received shutdown signal");
            })
            .await
            .expect("Server should run successfully until shutdown signal is received");
        info!("Server shutdown complete");
    });

    Ok(())
}

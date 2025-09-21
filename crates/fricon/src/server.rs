mod dataset;
mod fricon;

use anyhow::Result;
use tokio_util::sync::CancellationToken;
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

pub async fn run(app: AppHandle, cancellation_token: CancellationToken) -> Result<()> {
    let ipc_file = app.root().paths().ipc_file();
    let storage = Storage::new(app);
    let service = DatasetServiceServer::new(storage);
    let listener = ipc::listen(ipc_file)?;

    info!("Starting gRPC server");
    Server::builder()
        .add_service(service)
        .add_service(FriconServiceServer::new(Fricon))
        .serve_with_incoming_shutdown(listener, async {
            cancellation_token.cancelled().await;
            info!("Received shutdown signal");
        })
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

use std::path::PathBuf;

use anyhow::Result;
use grpc::{dataset_service::Storage, fricon_service::Fricon};
use tokio::runtime::Handle;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tonic::transport::Server;
use tracing::{error, info, instrument};

use crate::{
    app::AppHandle,
    proto::{
        dataset_service_server::DatasetServiceServer, fricon_service_server::FriconServiceServer,
    },
    transport::{grpc, ipc},
};

#[instrument(skip(app, task_tracker, cancellation_token), fields(ipc_file = %ipc_file.display()))]
pub(crate) fn start(
    ipc_file: PathBuf,
    app: &AppHandle,
    task_tracker: &TaskTracker,
    cancellation_token: CancellationToken,
    runtime: &Handle,
) -> Result<()> {
    let storage = Storage::new(
        app.dataset_catalog(),
        app.dataset_ingest(),
        cancellation_token.clone(),
    );
    let service = DatasetServiceServer::new(storage);
    let listener = ipc::listen(ipc_file, runtime)?;

    info!("Starting gRPC server");
    task_tracker.spawn_on(
        async move {
            let result = Server::builder()
                .add_service(service)
                .add_service(FriconServiceServer::new(Fricon))
                .serve_with_incoming_shutdown(listener, async {
                    cancellation_token.cancelled().await;
                    info!("Received shutdown signal");
                })
                .await;

            match result {
                Ok(()) => info!("Server shutdown complete"),
                Err(error) => error!(error = %error, "gRPC server exited with error"),
            }
        },
        runtime,
    );

    Ok(())
}

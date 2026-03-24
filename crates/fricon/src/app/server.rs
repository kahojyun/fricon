//! gRPC server startup for a workspace-backed app instance.

use std::path::PathBuf;

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

/// Start the workspace gRPC server on the IPC listener and register the task
/// with the shared tracker.
///
/// This composes the dataset and fricon RPC services, binds them to the
/// workspace-specific IPC transport, and arranges graceful shutdown through
/// `cancellation_token`.
#[instrument(skip(app, task_tracker, cancellation_token), fields(ipc_file = %ipc_file.display()))]
pub(crate) fn start(
    ipc_file: PathBuf,
    app: &AppHandle,
    task_tracker: &TaskTracker,
    cancellation_token: CancellationToken,
    runtime: &Handle,
) -> Result<(), crate::app::AppError> {
    let storage = Storage::new(app.clone(), cancellation_token.clone());
    let service = DatasetServiceServer::new(storage);
    let listener = ipc::listen(ipc_file, runtime)?;

    info!("Starting gRPC server");
    let app_for_fricon = app.clone();
    task_tracker.spawn_on(
        async move {
            let result = Server::builder()
                .add_service(service)
                .add_service(FriconServiceServer::new(Fricon {
                    app: app_for_fricon,
                }))
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

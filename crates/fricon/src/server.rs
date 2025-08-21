mod fricon;
mod storage;

pub use self::storage::DatasetRecord;

use std::path::PathBuf;

use anyhow::Result;
use tokio::signal;
use tokio_util::task::TaskTracker;
use tonic::transport::Server;
use tracing::info;

use crate::{
    app::App,
    ipc,
    proto::{
        data_storage_service_server::DataStorageServiceServer,
        fricon_service_server::FriconServiceServer,
    },
};

use self::{fricon::Fricon, storage::Storage};

pub async fn run(path: impl Into<PathBuf>) -> Result<()> {
    let app = App::open(path).await?;
    run_with_app(app).await
}

pub async fn run_with_app(app: App) -> Result<()> {
    let ipc_file = app.root().paths().ipc_file();
    let tracker = TaskTracker::new();
    let storage = Storage::new(app, tracker.clone());
    let service = DataStorageServiceServer::new(storage);
    let listener = ipc::listen(ipc_file)?;
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

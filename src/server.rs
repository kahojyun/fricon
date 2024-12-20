mod fricon;
mod storage;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::signal;
use tokio_util::task::TaskTracker;
use tonic::transport::Server;
use tracing::info;

use crate::{
    db::MIGRATOR,
    proto::{
        data_storage_service_server::DataStorageServiceServer,
        fricon_service_server::FriconServiceServer,
    },
    workspace,
};

use self::{fricon::Fricon, storage::Storage};

pub async fn run(path: std::path::PathBuf) -> Result<()> {
    let workspace = workspace::Workspace::open(path)?;
    let pool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::new().filename(workspace.root().database_file().0))
        .await?;
    MIGRATOR.run(&pool).await?;
    let port = workspace.config().port();
    let tracker = TaskTracker::new();
    let storage = Storage::new(workspace, pool, tracker.clone());
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

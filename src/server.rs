mod fricon;
mod storage;

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::signal;
use tonic::transport::Server;
use tracing::info;
use uuid::Uuid;

use crate::{
    dir,
    proto::{
        data_storage_service_server::DataStorageServiceServer,
        fricon_service_server::FriconServiceServer,
    },
};

use self::{fricon::Fricon, storage::Storage};

pub async fn run(path: std::path::PathBuf) -> Result<()> {
    let workspace = dir::Workspace::open(path);
    let pool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::new().filename(workspace.root().database_path()))
        .await?;
    dir::MIGRATOR.run(&pool).await?;
    let port = workspace.config().port();
    let storage = Storage::new(workspace, pool);
    let service = DataStorageServiceServer::new(storage);
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
    Ok(())
}

fn format_dataset_path(date: NaiveDate, uid: Uuid) -> String {
    format!("{date}/{uid}")
}

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_format_dataset_path() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let uid = uuid!("6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
        let path = format_dataset_path(date, uid);
        assert_eq!(path, "2021-01-01/6ecf30db-2e3f-4ef3-8aa1-1e035c6bddd0");
    }
}

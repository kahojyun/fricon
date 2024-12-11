pub mod cli;
mod config;
mod dataset;
mod db;
mod dir;
mod rpc;

use env_logger::Env;
use log::info;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::signal;
use tonic::transport::Server;

use crate::proto::fricon_service_server::FriconServiceServer;

use self::{
    cli::{Cli, Commands},
    rpc::{DataStorageServiceServer, Fricon, Storage},
};
pub use rpc::proto;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Main entry point for the application
///
/// # Errors
///
/// Returns a boxed error if server initialization or operation fails
pub async fn main(cli: Cli) -> Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));
    match cli.command {
        Commands::Init { path } => {
            dir::WorkDirectory::new(path).init().await;
        }
        Commands::Serve { path } => {
            run_server(path).await?;
        }
    }
    Ok(())
}

async fn run_server(path: std::path::PathBuf) -> Result<()> {
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

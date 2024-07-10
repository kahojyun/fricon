mod cli;
mod config;
mod dataset;
mod db;
mod dir;
mod rpc;

use clap::Parser as _;
use cli::Commands;
use log::info;
use rpc::{DataStorageServer, Storage};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::signal;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = cli::Cli::parse();
    match cli.command {
        Commands::Init { path } => {
            dir::WorkDirectory::new(path).init().await;
        }
        Commands::Serve { path } => {
            let workspace = dir::Workspace::open(path);
            let db_url = format!("sqlite://{}", workspace.root().database_path().display());
            let pool = SqlitePoolOptions::new().connect(&db_url).await?;
            dir::MIGRATOR.run(&pool).await?;
            let storage = Storage::new(pool);
            let service = DataStorageServer::new(storage);
            let port = workspace.config().port();
            let addr = format!("[::1]:{}", port).parse()?;
            info!("Listen on {}", addr);
            Server::builder()
                .add_service(service)
                .serve_with_shutdown(addr, async { signal::ctrl_c().await.unwrap() })
                .await?;
            info!("Shutdown");
        }
    }
    Ok(())
}

mod cli;
mod config;
mod dataset;
mod db;
mod dir;
mod rpc;

use clap::Parser as _;
use env_logger::Env;
use log::info;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::signal;
use tonic::transport::Server;

use self::{
    cli::Commands,
    rpc::{DataStorageServiceServer, Storage},
};

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));
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
            let port = workspace.config().port();
            let storage = Storage::new(workspace, pool);
            let service = DataStorageServiceServer::new(storage);
            let addr = format!("[::1]:{port}").parse()?;
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

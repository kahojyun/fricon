mod cli;
mod config;
mod storage;

use clap::Parser as _;
use cli::Commands;
use sqlx::sqlite::SqlitePoolOptions;
use storage::{DataStorageServer, Storage};
use tokio::signal;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = cli::Cli::parse();
    match cli.command {
        Some(Commands::Config) => {
            let path = config::default_config_path();
            if let Some(path) = path {
                println!("Config path: {}", path.display());
            } else {
                println!("Config path not found");
            }
        }
        None => {
            let addr = "[::1]:50051".parse()?;
            println!("Listen on {}", addr);
            let pool = SqlitePoolOptions::new()
                .connect("sqlite://test.sqlite3")
                .await?;
            sqlx::migrate!().run(&pool).await?;
            let storage = Storage { pool };
            let service = DataStorageServer::new(storage);
            Server::builder()
                .add_service(service)
                .serve_with_shutdown(addr, async { signal::ctrl_c().await.unwrap() })
                .await?;
            println!("Bye");
        }
    }
    Ok(())
}

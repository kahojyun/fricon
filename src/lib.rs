pub mod cli;
mod config;
mod dataset;
mod db;
mod dir;
pub mod proto;
mod server;

use anyhow::Result;

use self::{
    cli::{Cli, Commands},
    server::run,
};

/// Main entry point for the application
///
/// # Errors
///
/// Returns a boxed error if server initialization or operation fails
pub async fn main(cli: Cli) -> Result<()> {
    tracing_subscriber::fmt().init();
    match cli.command {
        Commands::Init { path } => {
            dir::WorkDirectory::new(path).init().await;
        }
        Commands::Serve { path } => {
            run(path).await?;
        }
    }
    Ok(())
}

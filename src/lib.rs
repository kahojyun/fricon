pub mod cli;
mod client;
mod config;
mod dataset;
mod db;
mod paths;
pub mod proto;
mod server;
mod workspace;

use anyhow::Result;

use self::{
    cli::{Cli, Commands},
    server::run,
    workspace::Workspace,
};

/// Version of fricon crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main entry point for the application
///
/// # Errors
///
/// Returns a boxed error if server initialization or operation fails
pub async fn main(cli: Cli) -> Result<()> {
    tracing_subscriber::fmt::init();
    match cli.command {
        Commands::Init { path } => {
            Workspace::init(path).await?;
        }
        Commands::Serve { path } => {
            run(path).await?;
        }
    }
    Ok(())
}

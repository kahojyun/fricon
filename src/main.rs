use anyhow::Result;
use clap::Parser as _;
use fricon::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    fricon::main(cli).await
}

use clap::Parser as _;
use fricon::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    fricon::main(cli).await
}

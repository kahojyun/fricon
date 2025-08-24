//! Binary entry point for fricon CLI

use anyhow::Result;
use clap::Parser;
use fricon_cli::Main;

fn main() -> Result<()> {
    let cli = fricon_cli::Cli::parse();

    cli.main()
}

//! Binary entry point for fricon CLI

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = fricon_cli::Cli::parse();

    fricon_cli::main(cli)
}

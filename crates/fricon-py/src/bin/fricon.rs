//! Binary entry point for fricon CLI

use anyhow::Result;
use clap::Parser;

use fricon_py::cli;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    cli::main(cli)
}

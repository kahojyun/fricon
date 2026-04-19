use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    fricon::cli::Cli::parse().run()
}

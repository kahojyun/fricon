// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    fricon_ui::cli::Gui::parse().run()
}

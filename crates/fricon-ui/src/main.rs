// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;

fn main() -> Result<()> {
    let _ = dotenv();
    let workspace_path = env::var("FRICON_WORKSPACE").ok().map(PathBuf::from);
    fricon_ui::cli::Gui::parse().run_standalone(workspace_path)
}

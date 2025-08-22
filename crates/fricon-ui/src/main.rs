// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fs};

use anyhow::Result;

fn main() -> Result<()> {
    let workspace_path = env::var("FRICON_WORKSPACE").expect("FRICON_WORKSPACE not set");
    let workspace_path =
        fs::canonicalize(workspace_path).expect("FRICON_WORKSPACE is not a valid path");
    fricon_ui::run_with_workspace(workspace_path)
}

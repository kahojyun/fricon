// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf};

use anyhow::Result;
use dotenvy::dotenv;
use fricon_ui::{InteractionMode, LaunchContext, LaunchSource};

fn main() -> Result<()> {
    let _ = dotenv();
    let workspace_path = env::var("FRICON_WORKSPACE").ok().map(PathBuf::from);
    fricon_ui::run_with_context(&LaunchContext {
        launch_source: LaunchSource::Standalone,
        workspace_path,
        interaction_mode: InteractionMode::Dialog,
    })
}

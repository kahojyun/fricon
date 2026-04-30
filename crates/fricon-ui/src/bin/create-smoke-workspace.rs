use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use fricon::{AppManager, Client, DatasetRow, DatasetScalar, WorkspaceRoot};
use indexmap::IndexMap;

#[derive(Debug, Parser)]
#[command(about = "Create a deterministic workspace fixture for desktop smoke tests")]
struct Args {
    /// Workspace path to create and seed.
    workspace: PathBuf,
    /// Remove the existing fixture workspace before recreating it.
    #[arg(long)]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    prepare_workspace_root(&args.workspace, args.force)?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(seed_workspace(args.workspace))
}

fn prepare_workspace_root(workspace: &PathBuf, force: bool) -> Result<()> {
    if workspace.exists() {
        if !force {
            bail!(
                "workspace fixture already exists at {}; rerun with --force to recreate it",
                workspace.display()
            );
        }
        fs::remove_dir_all(workspace).with_context(|| {
            format!(
                "failed to remove existing desktop smoke workspace at {}",
                workspace.display()
            )
        })?;
    }

    let Some(parent) = workspace.parent() else {
        bail!("workspace path must have a parent directory");
    };
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create parent directory for workspace fixture at {}",
            parent.display()
        )
    })?;
    Ok(())
}

async fn seed_workspace(workspace: PathBuf) -> Result<()> {
    WorkspaceRoot::create_new(&workspace).with_context(|| {
        format!(
            "failed to create desktop smoke workspace at {}",
            workspace.display()
        )
    })?;

    let app_manager = AppManager::new_with_path(&workspace)
        .context("failed to open workspace for fixture seeding")?
        .start(&tokio::runtime::Handle::current())
        .context("failed to start workspace server for fixture seeding")?;

    let client = Client::connect(&workspace)
        .await
        .context("failed to connect client for fixture seeding")?;

    let rows = create_smoke_rows();
    let schema = rows[0].to_schema();
    let mut writer = client
        .create_dataset(
            "desktop_smoke_signal".to_string(),
            "Small scalar dataset for Tauri desktop smoke coverage".to_string(),
            vec!["smoke".to_string(), "desktop".to_string()],
            schema,
            Vec::new(),
            None,
        )
        .await
        .context("failed to create smoke dataset writer")?;

    for row in rows {
        writer
            .write(row)
            .await
            .context("failed to write smoke dataset row")?;
    }

    let dataset = writer
        .finish()
        .await
        .context("failed to finalize smoke dataset")?;

    app_manager.shutdown().await;

    if dataset.id() <= 0 {
        bail!("fixture dataset id must be positive");
    }

    Ok(())
}

fn create_smoke_rows() -> Vec<DatasetRow> {
    [(0.0, 0.5), (1.0, 1.5), (2.0, 1.0), (3.0, 2.5), (4.0, 2.0)]
        .into_iter()
        .map(|(sweep, signal)| {
            let mut row = IndexMap::new();
            row.insert("sweep".to_string(), DatasetScalar::Numeric(sweep));
            row.insert("signal".to_string(), DatasetScalar::Numeric(signal));
            DatasetRow(row)
        })
        .collect()
}

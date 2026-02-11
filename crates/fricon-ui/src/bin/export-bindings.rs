use std::{env, path::PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let output = env::args().nth(1).map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("frontend")
                .join("src")
                .join("lib")
                .join("bindings.ts")
        },
        PathBuf::from,
    );

    fricon_ui::export_bindings(&output).with_context(|| {
        format!(
            "Failed to export Tauri bindings to {}",
            output.to_string_lossy()
        )
    })?;

    Ok(())
}

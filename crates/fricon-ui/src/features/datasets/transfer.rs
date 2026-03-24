use std::path::PathBuf;

use fricon::dataset::model::DatasetId;

use super::{
    error::UiDatasetError,
    types::{DatasetInfo, PreviewImportResult},
};
use crate::desktop_runtime::session::WorkspaceSession;

pub(crate) async fn export_datasets(
    session: &WorkspaceSession,
    ids: Vec<i32>,
    output_dir: PathBuf,
) -> Result<Vec<PathBuf>, UiDatasetError> {
    let app = session.app();
    let mut out_paths = Vec::with_capacity(ids.len());
    for id in ids {
        let out_path = app
            .export_dataset(DatasetId::Id(id), output_dir.clone())
            .await?;
        out_paths.push(out_path);
    }
    Ok(out_paths)
}

pub(crate) async fn preview_import_files(
    session: &WorkspaceSession,
    paths: Vec<PathBuf>,
) -> Result<Vec<PreviewImportResult>, UiDatasetError> {
    let app = session.app();
    let mut previews = Vec::with_capacity(paths.len());
    for path in paths {
        let preview = app.preview_import(path.clone()).await?;
        previews.push(PreviewImportResult {
            archive_path: path,
            preview,
        });
    }
    Ok(previews)
}

pub(crate) async fn import_dataset(
    session: &WorkspaceSession,
    archive_path: PathBuf,
    force: bool,
) -> Result<DatasetInfo, UiDatasetError> {
    Ok(session
        .app()
        .import_dataset(archive_path, force)
        .await?
        .into())
}

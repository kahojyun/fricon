use tracing::instrument;

use crate::{
    database::Pool,
    dataset_catalog::{DatasetCatalogError, DatasetId},
    dataset_ingest::WriteSessionRegistry,
    dataset_read::DatasetReader,
    workspace::WorkspaceRoot,
};

#[instrument(skip(database, root, write_sessions, id), fields(dataset.id = ?id))]
pub(crate) fn do_get_dataset_reader(
    database: &Pool,
    root: &WorkspaceRoot,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, DatasetCatalogError> {
    let mut conn = database.get()?;
    let dataset = super::do_get_dataset(&mut conn, id)?;
    if let Some(handle) = write_sessions.get(dataset.id) {
        Ok(DatasetReader::from_handle(handle)?)
    } else {
        let path = root.paths().dataset_path_from_uid(dataset.metadata.uid);
        Ok(DatasetReader::open_dir(path)?)
    }
}

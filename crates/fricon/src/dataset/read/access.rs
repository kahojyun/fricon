use tracing::instrument;

use crate::{
    dataset::{
        ingest::WriteSessionRegistry,
        model::DatasetId,
        read::{DatasetReader, ReadError},
        sqlite::{self, Pool},
    },
    workspace::WorkspacePaths,
};

#[instrument(skip(database, paths, write_sessions, id), fields(dataset.id = ?id))]
pub(crate) fn get_dataset_reader(
    database: &Pool,
    paths: &WorkspacePaths,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, ReadError> {
    let mut conn = database.get()?;
    let dataset = match id {
        DatasetId::Id(dataset_id) => sqlite::Dataset::find_by_id(&mut conn, dataset_id)?,
        DatasetId::Uid(uid) => sqlite::Dataset::find_by_uid(&mut conn, uid)?,
    }
    .ok_or_else(|| ReadError::NotFound {
        id: match id {
            DatasetId::Id(value) => value.to_string(),
            DatasetId::Uid(value) => value.to_string(),
        },
    })?;

    if let Some(handle) = write_sessions.get(dataset.id) {
        Ok(DatasetReader::from_handle(handle)?)
    } else {
        let path = paths.dataset_path_from_uid(dataset.uid.0);
        Ok(DatasetReader::open_dir(path)?)
    }
}

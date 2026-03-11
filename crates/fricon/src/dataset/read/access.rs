use tracing::instrument;

use crate::{
    dataset::{
        ingest::WriteSessionRegistry,
        model::DatasetId,
        read::{DatasetReadRepository, DatasetReader, ReadError},
    },
    workspace::WorkspacePaths,
};

#[instrument(skip(repository, paths, write_sessions, id), fields(dataset.id = ?id))]
pub(crate) fn get_dataset_reader(
    repository: &dyn DatasetReadRepository,
    paths: &WorkspacePaths,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, ReadError> {
    let dataset = repository.resolve_dataset(id)?;

    if let Some(handle) = write_sessions.get(dataset.id) {
        Ok(DatasetReader::from_handle(handle)?)
    } else {
        let path = paths.dataset_path_from_uid(dataset.uid);
        Ok(DatasetReader::open_dir(path)?)
    }
}

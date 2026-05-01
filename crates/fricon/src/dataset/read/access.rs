//! Read-access helper that chooses between active write sessions and
//! finalized on-disk dataset storage.

use tracing::instrument;

use crate::{
    dataset::{
        ingest::WriteSessionRegistry,
        model::DatasetId,
        read::{DatasetReadRepository, DatasetReader, ReadError},
        semantics::read_manifest_optional,
    },
    workspace::WorkspacePaths,
};

/// Resolve a dataset reader from either an active write session or the
/// finalized dataset directory.
///
/// Active sessions take precedence so reads stay consistent with the ingest
/// lifecycle while a dataset is still being written.
#[instrument(skip(repository, paths, write_sessions, id), fields(dataset.id = ?id))]
pub(crate) fn get_dataset_reader(
    repository: &dyn DatasetReadRepository,
    paths: &WorkspacePaths,
    write_sessions: &WriteSessionRegistry,
    id: DatasetId,
) -> Result<DatasetReader, ReadError> {
    let dataset = repository.resolve_dataset(id)?;

    let path = paths.dataset_path_from_uid(dataset.uid);

    // Prefer the active write session so reads observe in-progress data for a
    // dataset that has not yet been finalized to disk.
    if let Some(handle) = write_sessions.get(dataset.id) {
        Ok(DatasetReader::from_handle(
            handle,
            read_manifest_optional(&path)?,
        )?)
    } else {
        Ok(DatasetReader::open_dir(&path)?)
    }
}

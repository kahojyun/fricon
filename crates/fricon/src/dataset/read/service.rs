//! Dataset read service - resolves dataset readers across active write
//! sessions and on-disk dataset storage.

use std::sync::Arc;

use tracing::instrument;

use crate::{
    dataset::{
        ingest::WriteSessionRegistry,
        model::DatasetId,
        read::{DatasetReadRepository, DatasetReader, ReadError, access},
    },
    workspace::WorkspacePaths,
};

/// Stateless service coordinating dataset read access.
///
/// Holds the read repository boundary, workspace paths, and the shared
/// write-session registry. Active sessions take precedence so callers can read
/// freshly written data before the dataset is finalized on disk.
#[derive(Clone)]
pub(crate) struct DatasetReadService {
    repository: Arc<dyn DatasetReadRepository>,
    paths: WorkspacePaths,
    write_sessions: WriteSessionRegistry,
}

impl DatasetReadService {
    #[must_use]
    pub(crate) fn new(
        repository: Arc<dyn DatasetReadRepository>,
        paths: WorkspacePaths,
        write_sessions: WriteSessionRegistry,
    ) -> Self {
        Self {
            repository,
            paths,
            write_sessions,
        }
    }

    /// Resolve a dataset id into a [`DatasetReader`].
    ///
    /// The read path prefers an active write session over the on-disk payload
    /// so callers can observe in-progress ingest data.
    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub(crate) fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, ReadError> {
        access::get_dataset_reader(&*self.repository, &self.paths, &self.write_sessions, id)
    }
}

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

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub(crate) fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, ReadError> {
        access::get_dataset_reader(&*self.repository, &self.paths, &self.write_sessions, id)
    }
}

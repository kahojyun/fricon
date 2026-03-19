use std::sync::Arc;

use tracing::error;

use crate::{
    dataset::{
        events::DatasetEventPublisher,
        ingest::{
            CreateDatasetInputSource, CreateDatasetRequest, DatasetIngestRepository, IngestError,
            WriteSessionRegistry, create,
        },
        model::DatasetRecord,
    },
    workspace::WorkspacePaths,
};

#[derive(Clone)]
pub(crate) struct DatasetIngestService {
    repository: Arc<dyn DatasetIngestRepository>,
    paths: WorkspacePaths,
    write_sessions: WriteSessionRegistry,
}

impl DatasetIngestService {
    #[must_use]
    pub(crate) fn new(
        repository: Arc<dyn DatasetIngestRepository>,
        paths: WorkspacePaths,
        write_sessions: WriteSessionRegistry,
    ) -> Self {
        Self {
            repository,
            paths,
            write_sessions,
        }
    }

    pub(crate) fn create_dataset<S: CreateDatasetInputSource, P: DatasetEventPublisher>(
        &self,
        request: &CreateDatasetRequest,
        input_source: &mut S,
        events: &P,
    ) -> Result<DatasetRecord, IngestError> {
        let dataset_name = request.name.clone();

        create::create_dataset_with(
            &*self.repository,
            &self.paths,
            events,
            &self.write_sessions,
            request,
            input_source,
        )
        .inspect_err(|e| {
            error!(error = %e, dataset.name = %dataset_name, "Dataset creation failed");
        })
    }
}

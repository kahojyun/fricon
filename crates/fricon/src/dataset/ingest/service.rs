use std::sync::Arc;

use tracing::error;

use crate::{
    dataset::{
        events::DatasetEventPublisher,
        ingest::{
            CreateDatasetInput, CreateDatasetRequest, DatasetIngestRepository, IngestError,
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

    pub(crate) fn create_dataset<P, F>(
        &self,
        request: &CreateDatasetRequest,
        next_input: F,
        events: &P,
    ) -> Result<DatasetRecord, IngestError>
    where
        P: DatasetEventPublisher,
        F: FnMut() -> Option<CreateDatasetInput>,
    {
        let dataset_name = request.name.clone();

        create::create_dataset_with(
            &*self.repository,
            &self.paths,
            events,
            &self.write_sessions,
            request,
            next_input,
        )
        .inspect_err(|e| {
            error!(error = %e, dataset.name = %dataset_name, "Dataset creation failed");
        })
    }
}

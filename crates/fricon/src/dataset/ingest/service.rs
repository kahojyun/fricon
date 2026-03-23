//! Dataset ingest service - coordinates repository writes, dataset storage,
//! and write-session state for dataset creation.

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

/// Stateless service coordinating dataset ingest operations.
///
/// Holds the repository boundary, workspace paths, and shared write-session
/// registry. The actual multi-step create workflow lives in
/// [`create::create_dataset_with`].
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

    /// Create a dataset by delegating to the ingest workflow helper.
    ///
    /// This is the service entrypoint for streamed dataset creation. It
    /// passes the repository, workspace paths, event publisher, and shared
    /// session registry into the workflow and adds top-level failure logging.
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
        // Capture the name up front so failure logs still include it after the
        // request has been borrowed through the workflow helper.
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

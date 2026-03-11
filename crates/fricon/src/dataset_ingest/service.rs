use tracing::error;

use crate::{
    dataset_catalog::{DatasetCatalogError, DatasetRecord},
    dataset_ingest::{CreateDatasetRequest, CreateIngestEvent},
    dataset_manager::tasks,
    runtime::app::AppHandle,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct DatasetIngestService {
    app: AppHandle,
}

impl DatasetIngestService {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn create_dataset(
        &self,
        request: CreateDatasetRequest,
        events_rx: mpsc::Receiver<CreateIngestEvent>,
    ) -> Result<DatasetRecord, DatasetCatalogError> {
        let dataset_name = request.name.clone();
        self.app
            .spawn_blocking(move |state| {
                tasks::do_create_dataset(
                    &state.database,
                    &state.root,
                    &state.event_sender,
                    &state.write_sessions,
                    request,
                    events_rx,
                )
                .inspect_err(|e| {
                    error!(error = %e, dataset.name = %dataset_name, "Dataset creation failed");
                })
            })?
            .await?
    }
}

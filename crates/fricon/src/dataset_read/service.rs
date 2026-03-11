use tracing::instrument;

use crate::{
    dataset_catalog::{DatasetCatalogError, DatasetId, tasks},
    dataset_read::DatasetReader,
    runtime::app::AppHandle,
};

#[derive(Clone)]
pub struct DatasetReadService {
    app: AppHandle,
}

impl DatasetReadService {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    #[instrument(skip(self, id), fields(dataset.id = ?id))]
    pub async fn get_dataset_reader(
        &self,
        id: DatasetId,
    ) -> Result<DatasetReader, DatasetCatalogError> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_get_dataset_reader(
                    &state.database,
                    &state.root,
                    &state.write_sessions,
                    id,
                )
            })?
            .await?
    }
}

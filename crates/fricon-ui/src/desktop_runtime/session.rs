use std::sync::{Arc, Mutex};

use fricon::{DatasetReader, ReadAppError, app::AppHandle};

#[derive(Default)]
pub(crate) struct DatasetReaderCache {
    current_dataset: Mutex<Option<(i32, Arc<DatasetReader>)>>,
}

impl DatasetReaderCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn get_or_load(
        &self,
        app: &AppHandle,
        id: i32,
    ) -> Result<Arc<DatasetReader>, ReadAppError> {
        if let Some((current_id, current_dataset)) = self
            .current_dataset
            .lock()
            .expect("Should not be poisoned.")
            .clone()
            && current_id == id
        {
            return Ok(current_dataset);
        }

        let dataset = app.get_dataset_reader(id.into()).await?;
        let dataset = Arc::new(dataset);
        *self
            .current_dataset
            .lock()
            .expect("Should not be poisoned.") = Some((id, dataset.clone()));
        Ok(dataset)
    }
}

pub(crate) struct WorkspaceSession {
    app: AppHandle,
    dataset_readers: DatasetReaderCache,
}

impl WorkspaceSession {
    pub(crate) fn new(app: AppHandle) -> Self {
        Self {
            app,
            dataset_readers: DatasetReaderCache::new(),
        }
    }

    pub(crate) fn app(&self) -> &AppHandle {
        &self.app
    }

    pub(crate) async fn dataset(&self, id: i32) -> Result<Arc<DatasetReader>, ReadAppError> {
        self.dataset_readers.get_or_load(&self.app, id).await
    }
}

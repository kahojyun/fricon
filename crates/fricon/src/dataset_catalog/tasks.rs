use std::path::PathBuf;

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    database::{self, DatasetStatus, Pool},
    dataset_catalog::{DatasetCatalogError, DatasetId, DatasetRecord},
    dataset_ingest::{CreateDatasetRequest, WriteSessionGuard, WriteSessionRegistry},
    runtime::app::AppEvent,
    storage,
    workspace::WorkspaceRoot,
};

mod create;
mod mutate;
mod query;
mod reader;

pub(crate) use self::{
    create::do_create_dataset,
    mutate::{do_add_tags, do_delete_dataset, do_remove_tags, do_update_dataset},
    query::{do_get_dataset, do_list_dataset_tags, do_list_datasets},
    reader::do_get_dataset_reader,
};

#[cfg(test)]
mod tests;

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetRepo {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(database::Dataset, Vec<database::Tag>), DatasetCatalogError>;
    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), DatasetCatalogError>;
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetCatalogError>;
}

impl DatasetRepo for Pool {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<(database::Dataset, Vec<database::Tag>), DatasetCatalogError> {
        create::create_dataset_db_record(&mut *self.get()?, request, uid)
    }

    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), DatasetCatalogError> {
        let mut conn = self.get()?;
        database::Dataset::update_status(&mut conn, id, status)?;
        Ok(())
    }

    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, DatasetCatalogError> {
        let mut conn = self.get()?;
        query::do_get_dataset(&mut conn, id)
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetStore {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, DatasetCatalogError>;
}

impl DatasetStore for WorkspaceRoot {
    fn create_dataset_dir(&self, uid: Uuid) -> Result<PathBuf, DatasetCatalogError> {
        let path = self.paths().dataset_path_from_uid(uid);
        storage::create_dataset(&path)?;
        Ok(path)
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait DatasetEvents {
    fn send_dataset_created(&self, event: AppEvent);
}

impl DatasetEvents for broadcast::Sender<AppEvent> {
    fn send_dataset_created(&self, event: AppEvent) {
        let _ = self.send(event);
    }
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait WriteSessionGuardOps {
    fn write(&mut self, batch: RecordBatch) -> Result<(), DatasetCatalogError>;
    fn commit(self) -> Result<(), DatasetCatalogError>;
    fn abort(self) -> Result<(), DatasetCatalogError>;
}

impl WriteSessionGuardOps for WriteSessionGuard {
    fn write(&mut self, batch: RecordBatch) -> Result<(), DatasetCatalogError> {
        self.write_batch(batch)
    }

    fn commit(self) -> Result<(), DatasetCatalogError> {
        self.commit_session()
    }

    fn abort(self) -> Result<(), DatasetCatalogError> {
        self.abort_session()
    }
}

pub(super) trait WriteSessions {
    type Guard: WriteSessionGuardOps;

    fn start_session(&self, id: i32, path: PathBuf, schema: SchemaRef) -> Self::Guard;
}

impl WriteSessions for WriteSessionRegistry {
    type Guard = WriteSessionGuard;

    fn start_session(&self, id: i32, path: PathBuf, schema: SchemaRef) -> Self::Guard {
        WriteSessionRegistry::start_session(self, id, path, schema)
    }
}

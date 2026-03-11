use arrow_array::RecordBatch;
use uuid::Uuid;

mod create;
mod error;
mod registry;
mod service;
mod session;
mod storage;

pub use self::{error::IngestError, service::DatasetIngestService};
pub(crate) use self::{
    registry::{WriteSessionGuard, WriteSessionRegistry},
    session::WriteSessionHandle,
};
use crate::dataset::model::{DatasetId, DatasetRecord, DatasetStatus};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetIngestRepository: Send + Sync {
    fn create_dataset_record(
        &self,
        request: &CreateDatasetRequest,
        uid: Uuid,
    ) -> Result<DatasetRecord, IngestError>;
    fn update_status(&self, id: i32, status: DatasetStatus) -> Result<(), IngestError>;
    fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, IngestError>;
}

#[derive(Debug, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum CreateIngestEvent {
    Batch(RecordBatch),
    Terminal(CreateTerminal),
}

#[derive(Debug, Clone)]
pub enum CreateTerminal {
    Finish,
    Abort,
}

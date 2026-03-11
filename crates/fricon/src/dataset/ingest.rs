use arrow_array::RecordBatch;

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

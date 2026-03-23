//! Dataset ingest boundaries for creating and streaming dataset writes.
//!
//! The ingest flow creates a dataset record, streams zero or more Arrow
//! batches into a write session, and finishes in either `Completed` or
//! `Aborted` status. Repository implementations own database state; storage
//! and session lifetimes are coordinated in the ingest service/helpers.

mod create;
mod error;
mod registry;
mod service;
mod session;
mod storage;

use arrow_array::RecordBatch;
use uuid::Uuid;

pub use self::error::IngestError;
pub(crate) use self::{
    registry::WriteSessionRegistry, service::DatasetIngestService, session::WriteSessionHandle,
};
use crate::dataset::model::{DatasetId, DatasetRecord, DatasetStatus};

/// Persistence port for ingest-time dataset creation and status updates.
///
/// Implementations own the database record lifecycle for dataset creation.
/// The ingest workflow coordinates filesystem/session side effects around
/// these calls.
///
/// # Invariants
///
/// - `create_dataset_record` returns a record in `Writing` status for the
///   supplied `uid`.
/// - `update_status` persists the terminal ingest status chosen by the
///   workflow.
/// - `get_dataset` returns the current stored record after status updates.
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

/// User-supplied metadata for creating a new dataset before any rows are
/// written.
#[derive(Debug, Clone)]
pub(crate) struct CreateDatasetRequest {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
}

/// Stream input driving a dataset ingest session.
///
/// `Batch` appends rows, `Finish` commits the accumulated session, and
/// `Abort` discards any buffered data and leaves the dataset in `Aborted`
/// status.
#[derive(Debug)]
pub(crate) enum CreateDatasetInput {
    Batch(RecordBatch),
    Finish,
    Abort,
}

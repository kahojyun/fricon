//! Dataset read boundaries for resolving readers against live sessions or
//! stored dataset files.

mod access;
mod error;
mod reader;
mod service;

use std::ops::Bound;

use arrow_array::RecordBatch;
use uuid::Uuid;

pub(crate) use self::service::DatasetReadService;
pub use self::{error::ReadError, reader::DatasetReader};
use crate::dataset::model::DatasetId;

/// Repository-side location information needed to open a dataset payload.
///
/// The read service resolves ids through the repository, then chooses between
/// an active write session or the on-disk dataset directory using this data.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DatasetLocation {
    pub(crate) id: i32,
    pub(crate) uid: Uuid,
}

/// Persistence port for resolving a dataset id into a readable payload
/// location.
///
/// Implementations own any database lookup and not-found handling. The read
/// service uses this result to decide whether to read from an in-memory write
/// session or from the filesystem.
#[cfg_attr(test, mockall::automock)]
pub(crate) trait DatasetReadRepository: Send + Sync {
    fn resolve_dataset(&self, id: DatasetId) -> Result<DatasetLocation, ReadError>;
}

pub struct SelectOptions {
    pub start: Bound<usize>,
    pub end: Bound<usize>,
    pub index_filters: Option<RecordBatch>,
    pub selected_columns: Option<Vec<usize>>,
}

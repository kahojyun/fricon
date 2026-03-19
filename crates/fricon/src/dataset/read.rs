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

#[derive(Debug, Clone, Copy)]
pub(crate) struct DatasetLocation {
    pub(crate) id: i32,
    pub(crate) uid: Uuid,
}

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

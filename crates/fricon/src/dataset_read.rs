use std::ops::Bound;

use arrow_array::RecordBatch;

mod reader;
mod service;

pub use self::{reader::DatasetReader, service::DatasetReadService};

pub struct SelectOptions {
    pub start: Bound<usize>,
    pub end: Bound<usize>,
    pub index_filters: Option<RecordBatch>,
    pub selected_columns: Option<Vec<usize>>,
}

use std::ops::Bound;

use arrow_array::RecordBatch;

pub struct SelectOptions {
    pub start: Bound<usize>,
    pub end: Bound<usize>,
    pub index_filters: Option<RecordBatch>,
    pub selected_columns: Option<Vec<usize>>,
}

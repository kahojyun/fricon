mod arrays;
mod scalars;
mod table;
mod types;
mod utils;

use arrow_schema::ArrowError;

pub use self::{
    arrays::{DatasetArray, ScalarArray},
    scalars::{DatasetRow, DatasetScalar, FixedStepTrace, VariableStepTrace},
    table::ChunkedTable,
    types::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind},
    utils::downcast_array,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incompatible data type.")]
    IncompatibleType,
    #[error("X and Y length of trace mismatch.")]
    TraceLengthMismatch,
    #[error("Schema mismatch.")]
    SchemaMismatch,
    #[error("Invalid filter table.")]
    InvalidFilter,
    #[error(transparent)]
    Arrow(#[from] ArrowError),
}

mod arrays;
mod scalars;
mod table;
mod types;

use arrow_schema::ArrowError;

pub(crate) use self::table::ChunkedTable;
pub use self::{
    arrays::{DatasetArray, ScalarArray},
    scalars::{DatasetRow, DatasetScalar, FixedStepTrace, VariableStepTrace},
    types::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind},
};

#[derive(Debug, thiserror::Error)]
pub enum DatasetError {
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

mod arrays;
mod arrow_ext;
pub(crate) mod error;
mod model;
mod table;
mod values;

pub(crate) use self::table::ChunkedTable;
pub use self::{
    arrays::{DatasetArray, ScalarArray},
    error::DatasetError,
    model::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind},
    values::{DatasetRow, DatasetScalar, FixedStepTrace, VariableStepTrace},
};

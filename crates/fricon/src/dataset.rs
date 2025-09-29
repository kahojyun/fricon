mod arrays;
mod scalars;
mod types;
mod utils;

pub use arrays::{DatasetArray, ScalarArray};
pub use scalars::{DatasetRow, DatasetScalar, FixedStepTrace, VariableStepTrace};
pub use types::{DatasetDataType, DatasetSchema};
pub use utils::downcast_array;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incompatible data type")]
    IncompatibleType,
    #[error("X and Y length mismatch")]
    TraceLengthMismatch,
}

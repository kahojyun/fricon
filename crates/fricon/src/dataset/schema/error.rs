use arrow_schema::ArrowError;

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

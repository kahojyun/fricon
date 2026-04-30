use std::io;

use tempfile::PersistError;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Validation(#[from] ManifestValidationError),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ManifestValidationError {
    #[error("Unsupported dataset semantic manifest version: {found}")]
    UnsupportedVersion { found: u32 },
    #[error("Dataset semantic manifest must contain at least one column")]
    EmptyColumns,
    #[error("Dataset semantic manifest is missing the record id column")]
    MissingRecordIdColumn,
    #[error("Record id column must be named {expected}, found {found}")]
    InvalidRecordIdReference { expected: String, found: String },
    #[error("Record id column must be a uint64 system record id column")]
    InvalidRecordIdColumn,
    #[error("User column uses the reserved __ds_ prefix: {name}")]
    ReservedUserColumn { name: String },
    #[error("System column {name} is not valid in v1 dataset semantic manifests")]
    InvalidSystemColumn { name: String },
    #[error("System column {name} cannot carry user column metadata")]
    InvalidSystemColumnMetadata { name: String },
    #[error("Dataset semantic manifest v1 requires append_only=true")]
    AppendOnlyRequired,
    #[error("Dataset semantic manifest scan axis name must not be empty")]
    EmptyScanAxisName,
    #[error("Dataset semantic manifest scan axis {name} uses reserved system prefix")]
    ReservedScanAxisName { name: String },
    #[error("Dataset semantic manifest contains duplicate scan axis: {name}")]
    DuplicateScanAxis { name: String },
    #[error("Dataset semantic manifest static scan axis {name} must contain at least one value")]
    EmptyStaticScanAxis { name: String },
    #[error("Dataset semantic manifest v1 supports only one unknown-length scan axis")]
    MultipleUnknownScanAxes,
    #[error("Dataset semantic manifest v1 does not support mixed static and unknown scan axes")]
    MixedStaticAndUnknownScanAxes,
    #[error("Dataset semantic manifest scan plan requires implicit index realization")]
    ScanPlanRequiresImplicitRealization,
    #[error("Dataset semantic manifest implicit index realization requires a scan plan")]
    ImplicitRealizationRequiresScanPlan,
    #[error("Column metadata names a column that is not in the Arrow schema: {name}")]
    UnknownColumnMetadata { name: String },
    #[error("Column metadata was declared more than once: {name}")]
    DuplicateColumnMetadata { name: String },
    #[error("Arrow schema is missing manifest column: {name}")]
    MissingArrowColumn { name: String },
    #[error("Arrow schema has a column not declared in the manifest: {name}")]
    UnexpectedArrowColumn { name: String },
    #[error("Arrow type mismatch for {name}: expected {expected}, found {found}")]
    ArrowTypeMismatch {
        name: String,
        expected: String,
        found: String,
    },
    #[error("Unsupported Arrow type for dataset semantic manifest column {name}: {found}")]
    UnsupportedArrowType { name: String, found: String },
}

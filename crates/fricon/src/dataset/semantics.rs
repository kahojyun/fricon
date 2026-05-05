mod error;
mod io;
mod materialize;
mod model;

pub(crate) use self::materialize::{
    is_hidden_system_column, materialize_record_ids, materialized_schema,
};
pub use self::{
    error::{ManifestError, ManifestValidationError},
    io::{read_manifest, write_manifest},
    model::{
        ColumnMetadata, DatasetDType, DatasetSemanticManifest, DuplicateResolutionDefault,
        IndexRealization, Inference, MANIFEST_VERSION_V1, ManifestColumn, RECORD_ID_COLUMN,
        Realization, ScanAxis, ScanAxisMode, ScanAxisValue, ScanPlan, SemanticDescriptor,
        SemanticRole, SemanticShapeKind, SemanticValueKind, SystemColumn, TraceAxisDType,
        TraceDType, TraceLayout, TraceValueDType,
    },
};

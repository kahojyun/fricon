use std::collections::BTreeMap;

use arrow_schema::{DataType, Field, Schema, TimeUnit};
use serde::{Deserialize, Serialize};

use crate::dataset::semantics::ManifestValidationError;

pub const MANIFEST_VERSION_V1: u32 = 1;
pub const RECORD_ID_COLUMN: &str = "__ds_record_id";
const SYSTEM_COLUMN_PREFIX: &str = "__ds_";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetSemanticManifest {
    pub manifest_version: u32,
    pub columns: BTreeMap<String, ManifestColumn>,
    pub realization: Realization,
    pub compatibility: Compatibility,
}

impl DatasetSemanticManifest {
    #[must_use]
    pub fn minimal(columns: impl IntoIterator<Item = (String, ManifestColumn)>) -> Self {
        let mut columns: BTreeMap<_, _> = columns.into_iter().collect();
        columns.insert(RECORD_ID_COLUMN.to_string(), ManifestColumn::record_id());
        Self {
            manifest_version: MANIFEST_VERSION_V1,
            columns,
            realization: Realization::default(),
            compatibility: Compatibility::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.manifest_version != MANIFEST_VERSION_V1 {
            return Err(ManifestValidationError::UnsupportedVersion {
                found: self.manifest_version,
            });
        }
        if self.columns.is_empty() {
            return Err(ManifestValidationError::EmptyColumns);
        }
        if self.realization.record_id_column != RECORD_ID_COLUMN {
            return Err(ManifestValidationError::InvalidRecordIdReference {
                expected: RECORD_ID_COLUMN.to_string(),
                found: self.realization.record_id_column.clone(),
            });
        }
        if !self.realization.append_only {
            return Err(ManifestValidationError::AppendOnlyRequired);
        }
        self.validate_columns()
    }

    pub fn validate_against_arrow_schema(
        &self,
        schema: &Schema,
    ) -> Result<(), ManifestValidationError> {
        self.validate()?;

        for (name, column) in &self.columns {
            let field = schema
                .field_with_name(name)
                .map_err(|_| ManifestValidationError::MissingArrowColumn { name: name.clone() })?;
            let expected = column.dtype.physical_data_type();
            if field.data_type() != &expected {
                return Err(ManifestValidationError::ArrowTypeMismatch {
                    name: name.clone(),
                    expected: expected.to_string(),
                    found: field.data_type().to_string(),
                });
            }
            if field.is_nullable() {
                return Err(ManifestValidationError::ArrowTypeMismatch {
                    name: name.clone(),
                    expected: format!("non-null {expected}"),
                    found: format!("nullable {}", field.data_type()),
                });
            }
        }

        for field in schema.fields() {
            if !self.columns.contains_key(field.name()) {
                return Err(ManifestValidationError::UnexpectedArrowColumn {
                    name: field.name().clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_columns(&self) -> Result<(), ManifestValidationError> {
        let record_id_column = self
            .columns
            .get(RECORD_ID_COLUMN)
            .ok_or(ManifestValidationError::MissingRecordIdColumn)?;
        if !record_id_column.is_record_id() {
            return Err(ManifestValidationError::InvalidRecordIdColumn);
        }

        for (name, column) in &self.columns {
            match column.system {
                Some(SystemColumn::RecordId) if name != RECORD_ID_COLUMN => {
                    return Err(ManifestValidationError::InvalidSystemColumn {
                        name: name.clone(),
                    });
                }
                None if name.starts_with(SYSTEM_COLUMN_PREFIX) => {
                    return Err(ManifestValidationError::ReservedUserColumn { name: name.clone() });
                }
                Some(SystemColumn::RecordId) | None => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestColumn {
    pub dtype: DatasetDType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemColumn>,
}

impl ManifestColumn {
    #[must_use]
    pub fn new(dtype: DatasetDType) -> Self {
        Self {
            dtype,
            system: None,
        }
    }

    #[must_use]
    pub fn record_id() -> Self {
        Self {
            dtype: DatasetDType::UInt64,
            system: Some(SystemColumn::RecordId),
        }
    }

    #[must_use]
    pub fn is_record_id(&self) -> bool {
        self.dtype == DatasetDType::UInt64 && self.system == Some(SystemColumn::RecordId)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DatasetDType {
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "float32")]
    Float32,
    #[serde(rename = "int64")]
    Int64,
    #[serde(rename = "uint64")]
    UInt64,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "utf8")]
    Utf8,
    #[serde(rename = "timestamp_us")]
    TimestampUs,
    #[serde(rename = "complex128")]
    Complex128,
    #[serde(rename = "trace")]
    Trace {
        layout: TraceLayout,
        axis: TraceAxisDType,
        value: TraceValueDType,
    },
}

impl DatasetDType {
    #[must_use]
    pub fn trace(dtype: TraceDType) -> Self {
        Self::Trace {
            layout: dtype.layout,
            axis: dtype.axis,
            value: dtype.value,
        }
    }

    #[must_use]
    pub fn physical_data_type(&self) -> DataType {
        match self {
            Self::Float64 => DataType::Float64,
            Self::Float32 => DataType::Float32,
            Self::Int64 => DataType::Int64,
            Self::UInt64 => DataType::UInt64,
            Self::Bool => DataType::Boolean,
            Self::Utf8 => DataType::Utf8,
            Self::TimestampUs => DataType::Timestamp(TimeUnit::Microsecond, None),
            Self::Complex128 => complex128_data_type(),
            Self::Trace {
                layout,
                axis,
                value,
            } => trace_data_type(*layout, *axis, *value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceDType {
    pub layout: TraceLayout,
    pub axis: TraceAxisDType,
    pub value: TraceValueDType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TraceLayout {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "fixed_step")]
    FixedStep,
    #[serde(rename = "variable_step")]
    VariableStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TraceAxisDType {
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "float32")]
    Float32,
    #[serde(rename = "int64")]
    Int64,
    #[serde(rename = "uint64")]
    UInt64,
}

impl TraceAxisDType {
    #[must_use]
    pub fn physical_data_type(self) -> DataType {
        match self {
            Self::Float64 => DataType::Float64,
            Self::Float32 => DataType::Float32,
            Self::Int64 => DataType::Int64,
            Self::UInt64 => DataType::UInt64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TraceValueDType {
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "float32")]
    Float32,
    #[serde(rename = "int64")]
    Int64,
    #[serde(rename = "uint64")]
    UInt64,
    #[serde(rename = "complex128")]
    Complex128,
}

impl TraceValueDType {
    #[must_use]
    pub fn physical_data_type(self) -> DataType {
        match self {
            Self::Float64 => DataType::Float64,
            Self::Float32 => DataType::Float32,
            Self::Int64 => DataType::Int64,
            Self::UInt64 => DataType::UInt64,
            Self::Complex128 => complex128_data_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Realization {
    pub append_only: bool,
    pub record_id_column: String,
    pub index_realization: IndexRealization,
    pub duplicate_resolution_default: DuplicateResolutionDefault,
}

impl Default for Realization {
    fn default() -> Self {
        Self {
            append_only: true,
            record_id_column: RECORD_ID_COLUMN.to_string(),
            index_realization: IndexRealization::default(),
            duplicate_resolution_default: DuplicateResolutionDefault::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind")]
pub enum IndexRealization {
    #[default]
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind")]
pub enum DuplicateResolutionDefault {
    #[default]
    #[serde(rename = "latest_by_record_id")]
    LatestByRecordId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind")]
pub enum SystemColumn {
    #[default]
    #[serde(rename = "record_id")]
    RecordId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    pub allow_inference: bool,
}

impl Default for Compatibility {
    fn default() -> Self {
        Self {
            allow_inference: true,
        }
    }
}

fn trace_data_type(layout: TraceLayout, axis: TraceAxisDType, value: TraceValueDType) -> DataType {
    let axis = axis.physical_data_type();
    let value = value.physical_data_type();
    match layout {
        TraceLayout::Simple => DataType::new_list(value, false),
        TraceLayout::FixedStep => DataType::Struct(
            vec![
                Field::new("x0", axis.clone(), false),
                Field::new("step", axis, false),
                Field::new("y", DataType::new_list(value, false), false),
            ]
            .into(),
        ),
        TraceLayout::VariableStep => DataType::Struct(
            vec![
                Field::new("x", DataType::new_list(axis, false), false),
                Field::new("y", DataType::new_list(value, false), false),
            ]
            .into(),
        ),
    }
}

fn complex128_data_type() -> DataType {
    DataType::Struct(
        vec![
            Field::new("real", DataType::Float64, false),
            Field::new("imag", DataType::Float64, false),
        ]
        .into(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use arrow_schema::{DataType, Field, Schema};
    use serde_json::json;

    use super::{
        Compatibility, DatasetDType, DatasetSemanticManifest, DuplicateResolutionDefault,
        IndexRealization, ManifestColumn, ManifestValidationError, RECORD_ID_COLUMN, Realization,
        SystemColumn, TraceAxisDType, TraceDType, TraceLayout, TraceValueDType,
    };

    fn signal_columns() -> BTreeMap<String, ManifestColumn> {
        BTreeMap::from([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
    }

    #[test]
    fn minimal_manifest_serializes_to_adr_shape() {
        let manifest = DatasetSemanticManifest::minimal(signal_columns());
        let value = serde_json::to_value(&manifest).expect("serialize manifest");

        assert_eq!(
            value,
            json!({
                "manifest_version": 1,
                "columns": {
                    "__ds_record_id": {
                        "dtype": { "kind": "uint64" },
                        "system": { "kind": "record_id" }
                    },
                    "signal": {
                        "dtype": { "kind": "float64" }
                    }
                },
                "realization": {
                    "append_only": true,
                    "record_id_column": "__ds_record_id",
                    "index_realization": { "kind": "none" },
                    "duplicate_resolution_default": { "kind": "latest_by_record_id" }
                },
                "compatibility": {
                    "allow_inference": true
                }
            })
        );

        let parsed: DatasetSemanticManifest =
            serde_json::from_value(value).expect("deserialize manifest");
        assert_eq!(parsed, manifest);
        parsed.validate().expect("manifest should be valid");
    }

    #[test]
    fn minimal_manifest_injects_record_id_and_defaults() {
        let manifest = DatasetSemanticManifest::minimal(signal_columns());

        assert_eq!(
            manifest.columns.get(RECORD_ID_COLUMN),
            Some(&ManifestColumn::record_id())
        );
        assert_eq!(manifest.realization, Realization::default());
        assert_eq!(manifest.compatibility, Compatibility::default());
    }

    #[test]
    fn validate_rejects_invalid_version() {
        let mut manifest = DatasetSemanticManifest::minimal(signal_columns());
        manifest.manifest_version = 2;

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::UnsupportedVersion { found: 2 })
        );
    }

    #[test]
    fn validate_rejects_missing_record_id() {
        let manifest = DatasetSemanticManifest {
            manifest_version: 1,
            columns: signal_columns(),
            realization: Realization::default(),
            compatibility: Compatibility::default(),
        };

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::MissingRecordIdColumn)
        );
    }

    #[test]
    fn validate_rejects_wrong_record_id_dtype() {
        let mut manifest = DatasetSemanticManifest::minimal(signal_columns());
        manifest.columns.insert(
            RECORD_ID_COLUMN.to_string(),
            ManifestColumn {
                dtype: DatasetDType::Int64,
                system: Some(SystemColumn::RecordId),
            },
        );

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidRecordIdColumn)
        );
    }

    #[test]
    fn validate_rejects_bad_record_id_reference() {
        let mut manifest = DatasetSemanticManifest::minimal(signal_columns());
        manifest.realization.record_id_column = "other".to_string();

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidRecordIdReference {
                expected: RECORD_ID_COLUMN.to_string(),
                found: "other".to_string()
            })
        );
    }

    #[test]
    fn validate_rejects_reserved_user_prefix() {
        let manifest = DatasetSemanticManifest::minimal([(
            "__ds_user".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )]);

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::ReservedUserColumn {
                name: "__ds_user".to_string()
            })
        );
    }

    #[test]
    fn validate_rejects_non_append_only_manifest() {
        let mut manifest = DatasetSemanticManifest::minimal(signal_columns());
        manifest.realization.append_only = false;

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::AppendOnlyRequired)
        );
    }

    #[test]
    fn validate_against_arrow_schema_accepts_supported_physical_types() {
        let manifest = DatasetSemanticManifest::minimal([
            (
                "signal".to_string(),
                ManifestColumn::new(DatasetDType::Float64),
            ),
            (
                "complex".to_string(),
                ManifestColumn::new(DatasetDType::Complex128),
            ),
            (
                "trace".to_string(),
                ManifestColumn::new(DatasetDType::trace(TraceDType {
                    layout: TraceLayout::FixedStep,
                    axis: TraceAxisDType::Float64,
                    value: TraceValueDType::Complex128,
                })),
            ),
        ]);
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new(
                "complex",
                DatasetDType::Complex128.physical_data_type(),
                false,
            ),
            Field::new("signal", DataType::Float64, false),
            Field::new(
                "trace",
                DatasetDType::trace(TraceDType {
                    layout: TraceLayout::FixedStep,
                    axis: TraceAxisDType::Float64,
                    value: TraceValueDType::Complex128,
                })
                .physical_data_type(),
                false,
            ),
        ]);

        manifest
            .validate_against_arrow_schema(&schema)
            .expect("schema should match manifest");
    }

    #[test]
    fn validate_against_arrow_schema_rejects_type_mismatch() {
        let manifest = DatasetSemanticManifest::minimal(signal_columns());
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Utf8, false),
        ]);

        assert!(matches!(
            manifest.validate_against_arrow_schema(&schema),
            Err(ManifestValidationError::ArrowTypeMismatch { name, .. })
                if name == "signal"
        ));
    }

    #[test]
    fn validate_against_arrow_schema_rejects_nullable_record_id() {
        let manifest = DatasetSemanticManifest::minimal(signal_columns());
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, true),
            Field::new("signal", DataType::Float64, false),
        ]);

        assert!(matches!(
            manifest.validate_against_arrow_schema(&schema),
            Err(ManifestValidationError::ArrowTypeMismatch { name, .. })
                if name == RECORD_ID_COLUMN
        ));
    }

    #[test]
    fn default_manifest_values_are_adr_values() {
        assert_eq!(IndexRealization::default(), IndexRealization::None);
        assert_eq!(
            DuplicateResolutionDefault::default(),
            DuplicateResolutionDefault::LatestByRecordId
        );
    }
}

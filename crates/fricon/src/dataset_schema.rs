//! Dataset schema definition and conversion utilities.
//!
//! This module defines the core business logic types for dataset schemas,
//! providing a simplified type system that maps to Arrow types at boundaries.

use arrow::datatypes::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::datatypes::{ComplexType, FriconTypeExt, TraceType};

/// Scalar data types supported in MVP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScalarKind {
    Float64,
    Complex128,
}

/// Trace variants supported in MVP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceVariant {
    SimpleList,
    FixedStep,
    VariableStep,
}

/// Core dataset data types for business logic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatasetDataType {
    Scalar(ScalarKind),
    Trace {
        variant: TraceVariant,
        y: ScalarKind,
    },
}

/// A field in a dataset schema
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetField {
    pub name: String,
    pub dtype: DatasetDataType,
    pub nullable: bool,
}

/// Dataset schema containing field definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub fields: Vec<DatasetField>,
}

impl ScalarKind {
    /// Convert to Arrow Field for the scalar type
    #[must_use]
    pub fn to_arrow_field(&self, name: &str, nullable: bool) -> Field {
        match self {
            ScalarKind::Float64 => Field::new(name, DataType::Float64, nullable),
            ScalarKind::Complex128 => ComplexType::field(name, nullable),
        }
    }

    /// Try to infer `ScalarKind` from Arrow Field
    #[must_use]
    pub fn from_arrow_field(field: &Field) -> Option<ScalarKind> {
        match field.data_type() {
            DataType::Float64 => Some(ScalarKind::Float64),
            _ if field.is_complex() => Some(ScalarKind::Complex128),
            _ => None,
        }
    }
}

impl TraceVariant {
    /// Convert to `TraceType` for Arrow conversion
    #[must_use]
    pub fn to_trace_type(self) -> TraceType {
        match self {
            TraceVariant::SimpleList => TraceType::SimpleList,
            TraceVariant::FixedStep => TraceType::FixedStep,
            TraceVariant::VariableStep => TraceType::VariableStep,
        }
    }

    /// Convert from `TraceType`
    #[must_use]
    pub fn from_trace_type(trace_type: TraceType) -> TraceVariant {
        match trace_type {
            TraceType::SimpleList => TraceVariant::SimpleList,
            TraceType::FixedStep => TraceVariant::FixedStep,
            TraceType::VariableStep => TraceVariant::VariableStep,
        }
    }
}

impl DatasetDataType {
    /// Convert to Arrow Field with proper extension types
    #[must_use]
    pub fn to_arrow_field(&self, name: &str, nullable: bool) -> Field {
        match self {
            DatasetDataType::Scalar(scalar) => scalar.to_arrow_field(name, nullable),
            DatasetDataType::Trace { variant, y } => {
                let y_item_field = match y {
                    ScalarKind::Float64 => Arc::new(Field::new("item", DataType::Float64, false)),
                    ScalarKind::Complex128 => Arc::new(ComplexType::field("item", false)),
                };
                let trace_type = variant.to_trace_type();
                trace_type.field(name, y_item_field, nullable)
            }
        }
    }

    /// Try to infer `DatasetDataType` from Arrow Field
    #[must_use]
    pub fn from_arrow_field(field: &Field) -> Option<DatasetDataType> {
        // Try scalar types first
        if let Some(scalar) = ScalarKind::from_arrow_field(field) {
            return Some(DatasetDataType::Scalar(scalar));
        }

        // Try trace types
        if field.is_trace()
            && let Some(trace_type) = field.trace_type()
        {
            let variant = TraceVariant::from_trace_type(trace_type);
            // For MVP, assume y is Float64 for traces
            // TODO: In the future, we should infer the y type from the trace structure
            let y = ScalarKind::Float64;
            return Some(DatasetDataType::Trace { variant, y });
        }

        None
    }

    /// Get a human-readable description of this type
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            DatasetDataType::Scalar(ScalarKind::Float64) => "64-bit floating point".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex128) => "128-bit complex number".to_string(),
            DatasetDataType::Trace { variant, y } => {
                let y_desc = match y {
                    ScalarKind::Float64 => "float64",
                    ScalarKind::Complex128 => "complex128",
                };
                let variant_desc = match variant {
                    TraceVariant::SimpleList => "simple list",
                    TraceVariant::FixedStep => "fixed step",
                    TraceVariant::VariableStep => "variable step",
                };
                format!("Trace ({variant_desc}) of {y_desc} values")
            }
        }
    }
}

impl DatasetField {
    /// Create a new field
    pub fn new(name: impl Into<String>, dtype: DatasetDataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            dtype,
            nullable,
        }
    }

    /// Convert to Arrow Field
    #[must_use]
    pub fn to_arrow_field(&self) -> Field {
        self.dtype.to_arrow_field(&self.name, self.nullable)
    }

    /// Try to create from Arrow Field
    #[must_use]
    pub fn from_arrow_field(field: &Field) -> Option<DatasetField> {
        let dtype = DatasetDataType::from_arrow_field(field)?;
        Some(DatasetField {
            name: field.name().clone(),
            dtype,
            nullable: field.is_nullable(),
        })
    }
}

impl DatasetSchema {
    /// Create a new schema
    #[must_use]
    pub fn new(fields: Vec<DatasetField>) -> Self {
        Self { fields }
    }

    /// Convert to Arrow Schema
    #[must_use]
    pub fn to_arrow(&self) -> Arc<Schema> {
        let fields: Vec<Field> = self
            .fields
            .iter()
            .map(DatasetField::to_arrow_field)
            .collect();
        Arc::new(Schema::new(fields))
    }

    /// Try to create from Arrow Schema
    /// Returns None if any field cannot be converted to `DatasetDataType`
    #[must_use]
    pub fn try_from_arrow(schema: &Schema) -> Option<Self> {
        let mut fields = Vec::new();
        for arrow_field in schema.fields() {
            let dataset_field = DatasetField::from_arrow_field(arrow_field)?;
            fields.push(dataset_field);
        }
        Some(Self::new(fields))
    }

    /// Create from Arrow Schema, filtering out unsupported fields
    /// Returns the schema and a list of field names that were skipped
    #[must_use]
    pub fn from_arrow_with_filter(schema: &Schema) -> (Self, Vec<String>) {
        let mut fields = Vec::new();
        let mut skipped = Vec::new();

        for arrow_field in schema.fields() {
            if let Some(dataset_field) = DatasetField::from_arrow_field(arrow_field) {
                fields.push(dataset_field);
            } else {
                skipped.push(arrow_field.name().clone());
            }
        }

        (Self::new(fields), skipped)
    }

    /// Get field by name
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&DatasetField> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get field by index
    #[must_use]
    pub fn field_at(&self, index: usize) -> Option<&DatasetField> {
        self.fields.get(index)
    }

    /// Number of fields
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if schema is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;

    #[test]
    fn test_scalar_float64_roundtrip() {
        let field = DatasetField::new(
            "test_field",
            DatasetDataType::Scalar(ScalarKind::Float64),
            false,
        );
        let arrow_field = field.to_arrow_field();
        let converted = DatasetField::from_arrow_field(&arrow_field).unwrap();
        assert_eq!(field, converted);
    }

    #[test]
    fn test_scalar_complex_roundtrip() {
        let field = DatasetField::new(
            "test_complex",
            DatasetDataType::Scalar(ScalarKind::Complex128),
            true,
        );
        let arrow_field = field.to_arrow_field();
        let converted = DatasetField::from_arrow_field(&arrow_field).unwrap();
        assert_eq!(field, converted);
    }

    #[test]
    fn test_trace_simple_list_roundtrip() {
        let field = DatasetField::new(
            "test_trace",
            DatasetDataType::Trace {
                variant: TraceVariant::SimpleList,
                y: ScalarKind::Float64,
            },
            false,
        );
        let arrow_field = field.to_arrow_field();
        let converted = DatasetField::from_arrow_field(&arrow_field).unwrap();
        assert_eq!(field, converted);
    }

    #[test]
    fn test_schema_roundtrip() {
        let schema = DatasetSchema::new(vec![
            DatasetField::new(
                "float_col",
                DatasetDataType::Scalar(ScalarKind::Float64),
                false,
            ),
            DatasetField::new(
                "complex_col",
                DatasetDataType::Scalar(ScalarKind::Complex128),
                true,
            ),
            DatasetField::new(
                "trace_col",
                DatasetDataType::Trace {
                    variant: TraceVariant::FixedStep,
                    y: ScalarKind::Float64,
                },
                false,
            ),
        ]);

        let arrow_schema = schema.to_arrow();
        let converted = DatasetSchema::try_from_arrow(&arrow_schema).unwrap();
        assert_eq!(schema, converted);
    }

    #[test]
    fn test_json_serialization() {
        let schema = DatasetSchema::new(vec![
            DatasetField::new("x", DatasetDataType::Scalar(ScalarKind::Float64), false),
            DatasetField::new("y", DatasetDataType::Scalar(ScalarKind::Complex128), false),
        ]);

        let json = schema.to_json().unwrap();
        let deserialized = DatasetSchema::from_json(&json).unwrap();
        assert_eq!(schema, deserialized);
    }

    #[test]
    fn test_unsupported_arrow_type_filter() {
        // Create an Arrow schema with mixed supported and unsupported types
        let arrow_schema = Schema::new(vec![
            Field::new("float_col", DataType::Float64, false),
            Field::new("unsupported_string", DataType::Utf8, false),
            ComplexType::field("complex_col", true),
            Field::new("unsupported_int", DataType::Int64, false),
        ]);

        let (dataset_schema, skipped) = DatasetSchema::from_arrow_with_filter(&arrow_schema);

        assert_eq!(dataset_schema.len(), 2); // Only float and complex should be kept
        assert_eq!(skipped, vec!["unsupported_string", "unsupported_int"]);

        // Check that the supported fields are correct
        assert_eq!(
            dataset_schema.field("float_col").unwrap().dtype,
            DatasetDataType::Scalar(ScalarKind::Float64)
        );
        assert_eq!(
            dataset_schema.field("complex_col").unwrap().dtype,
            DatasetDataType::Scalar(ScalarKind::Complex128)
        );
    }
}

//! Dataset schema core types and Arrow conversion helpers.

use arrow::datatypes::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::datatypes::{ComplexType, FriconTypeExt, TraceType};

/// Scalar data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScalarKind {
    Float64,
    Complex128,
}

/// Dataset data types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatasetDataType {
    Scalar(ScalarKind),
    Trace { variant: TraceType, y: ScalarKind },
}

/// Field definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetField {
    pub name: String,
    pub dtype: DatasetDataType,
    pub nullable: bool,
}

/// Schema (ordered fields)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub fields: Vec<DatasetField>,
}

impl ScalarKind {
    /// Convert to Arrow field
    #[must_use]
    pub fn to_arrow_field(&self, name: &str, nullable: bool) -> Field {
        match self {
            ScalarKind::Float64 => Field::new(name, DataType::Float64, nullable),
            ScalarKind::Complex128 => ComplexType::field(name, nullable),
        }
    }

    /// Infer from Arrow field
    #[must_use]
    pub fn from_arrow_field(field: &Field) -> Option<ScalarKind> {
        match field.data_type() {
            DataType::Float64 => Some(ScalarKind::Float64),
            _ if field.is_complex() => Some(ScalarKind::Complex128),
            _ => None,
        }
    }
}

// Traces use extension type `TraceType` directly.

impl DatasetDataType {
    /// Convert to Arrow field (adds extension types where needed)
    #[must_use]
    pub fn to_arrow_field(&self, name: &str, nullable: bool) -> Field {
        match self {
            DatasetDataType::Scalar(scalar) => scalar.to_arrow_field(name, nullable),
            DatasetDataType::Trace { variant, y } => {
                let y_item_field = match y {
                    ScalarKind::Float64 => Arc::new(Field::new("item", DataType::Float64, false)),
                    ScalarKind::Complex128 => Arc::new(ComplexType::field("item", false)),
                };
                variant.field(name, y_item_field, nullable)
            }
        }
    }

    /// Try to infer `DatasetDataType` from Arrow Field
    #[must_use]
    pub fn from_arrow_field(field: &Field) -> Option<DatasetDataType> {
        // Scalar?
        if let Some(scalar) = ScalarKind::from_arrow_field(field) {
            return Some(DatasetDataType::Scalar(scalar));
        }

        // Trace?
        if let Some((variant, item_field)) = field.parse_trace_datatype() {
            if let Some(y) = ScalarKind::from_arrow_field(item_field) {
                return Some(DatasetDataType::Trace { variant, y });
            }
            return None; // Unsupported inner type (e.g., Int32) – we cannot represent it yet
        }

        None
    }

    /// Human-readable description
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
                    TraceType::SimpleList => "simple list",
                    TraceType::FixedStep => "fixed step",
                    TraceType::VariableStep => "variable step",
                };
                format!("Trace ({variant_desc}) of {y_desc} values")
            }
        }
    }
}

impl DatasetField {
    /// New field
    pub fn new(name: impl Into<String>, dtype: DatasetDataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            dtype,
            nullable,
        }
    }

    /// To Arrow field
    #[must_use]
    pub fn to_arrow_field(&self) -> Field {
        self.dtype.to_arrow_field(&self.name, self.nullable)
    }

    /// From Arrow field (lossy for unsupported types)
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
    /// New schema
    #[must_use]
    pub fn new(fields: Vec<DatasetField>) -> Self {
        Self { fields }
    }

    /// To Arrow schema
    #[must_use]
    pub fn to_arrow(&self) -> Arc<Schema> {
        let fields: Vec<Field> = self
            .fields
            .iter()
            .map(DatasetField::to_arrow_field)
            .collect();
        Arc::new(Schema::new(fields))
    }

    /// Try from Arrow (fails if any field unsupported)
    #[must_use]
    pub fn try_from_arrow(schema: &Schema) -> Option<Self> {
        let mut fields = Vec::new();
        for arrow_field in schema.fields() {
            let dataset_field = DatasetField::from_arrow_field(arrow_field)?;
            fields.push(dataset_field);
        }
        Some(Self::new(fields))
    }

    /// From Arrow filtering out unsupported fields (returns skipped names)
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

    /// Field by name
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&DatasetField> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Field by index
    #[must_use]
    pub fn field_at(&self, index: usize) -> Option<&DatasetField> {
        self.fields.get(index)
    }

    /// Number of fields
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
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
                variant: TraceType::SimpleList,
                y: ScalarKind::Float64,
            },
            false,
        );
        let arrow_field = field.to_arrow_field();
        let converted = DatasetField::from_arrow_field(&arrow_field).unwrap();
        assert_eq!(field, converted);
    }

    #[test]
    fn test_trace_simple_list_complex_roundtrip() {
        // Build a simple list trace whose y item is Complex128.
        let field = DatasetField::new(
            "complex_trace",
            DatasetDataType::Trace {
                variant: TraceType::SimpleList,
                y: ScalarKind::Complex128,
            },
            false,
        );
        let arrow_field = field.to_arrow_field();
        // Ensure Arrow representation actually has a complex item type
        if let DataType::List(item) = arrow_field.data_type() {
            assert!(item.is_complex());
        } else {
            panic!("expected list data type for complex trace");
        }
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
                    variant: TraceType::FixedStep,
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

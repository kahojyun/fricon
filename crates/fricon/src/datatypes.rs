//! Custom Arrow data type extensions for fricon
//!
//! This module defines custom Arrow data types specific to fricon using
//! Arrow's official extension type specification:
//! - Complex: Complex numbers with real and imaginary parts
//! - Trace: Different types of trace data with various x-axis representations
//!
//! Extension types use the ARROW:extension:name and ARROW:extension:metadata
//! keys in Field metadata to identify custom data types.

use arrow::datatypes::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Extension metadata keys for Arrow extension types
pub const ARROW_EXTENSION_NAME: &str = "ARROW:extension:name";

/// Complex number data type
///
/// Represents complex numbers with real and imaginary parts as f64 values
/// using Arrow's extension type specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComplexType {
    /// Extension type name
    extension_name: String,
}

impl ComplexType {
    /// Create a new complex type
    pub fn new() -> Self {
        Self {
            extension_name: "fricon.complex".to_string(),
        }
    }

    /// Get the extension name
    pub fn extension_name(&self) -> &str {
        &self.extension_name
    }

    /// Get the Arrow data type representation (storage type)
    pub fn storage_type() -> DataType {
        // Complex numbers are stored as a struct with real and imaginary fields
        DataType::Struct(
            vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ]
            .into(),
        )
    }

    /// Create a field for complex numbers with extension metadata
    pub fn field(name: &str, nullable: bool) -> Field {
        let mut field = Field::new(name, Self::storage_type(), nullable);

        // Add extension name (required)
        let mut metadata = HashMap::new();
        metadata.insert(
            ARROW_EXTENSION_NAME.to_string(),
            "fricon.complex".to_string(),
        );

        field.set_metadata(metadata);
        field
    }
}

/// Trace variants for different x-axis representations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceVariant {
    /// Simple list of y values with implicit x indices (0, 1, 2, ...)
    SimpleList,
    /// Fixed step size x values (x0, x0+step, x0+2*step, ...)
    FixedStep,
    /// Variable step size x values (explicit x array)
    VariableStep,
}

impl std::fmt::Display for TraceVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceVariant::SimpleList => write!(f, "simple_list"),
            TraceVariant::FixedStep => write!(f, "fixed_step"),
            TraceVariant::VariableStep => write!(f, "variable_step"),
        }
    }
}

/// Trace data type for storing time series or signal data
/// using Arrow's extension type specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceType {
    /// Extension type name
    extension_name: String,
    /// Variant of the trace
    variant: TraceVariant,
}

impl TraceType {
    /// Create a new simple list trace type
    pub fn simple_list() -> Self {
        Self {
            extension_name: "fricon.trace.simple_list".to_string(),
            variant: TraceVariant::SimpleList,
        }
    }

    /// Create a new fixed step trace type
    pub fn fixed_step() -> Self {
        Self {
            extension_name: "fricon.trace.fixed_step".to_string(),
            variant: TraceVariant::FixedStep,
        }
    }

    /// Create a new variable step trace type
    pub fn variable_step() -> Self {
        Self {
            extension_name: "fricon.trace.variable_step".to_string(),
            variant: TraceVariant::VariableStep,
        }
    }

    /// Get the extension name
    pub fn extension_name(&self) -> &str {
        &self.extension_name
    }

    /// Get the trace variant
    pub fn variant(&self) -> TraceVariant {
        self.variant
    }

    /// Get the Arrow data type representation (storage type)
    pub fn storage_type(&self) -> DataType {
        match self.variant {
            TraceVariant::SimpleList => {
                // Simple list: just y values as f64 array
                DataType::List(Arc::new(Field::new("y", DataType::Float64, false)))
            }
            TraceVariant::FixedStep => {
                // Fixed step: struct with x0, step, and y values
                DataType::Struct(
                    vec![
                        Field::new("x0", DataType::Float64, false),
                        Field::new("step", DataType::Float64, false),
                        Field::new(
                            "y",
                            DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                            false,
                        ),
                    ]
                    .into(),
                )
            }
            TraceVariant::VariableStep => {
                // Variable step: struct with x and y arrays
                DataType::Struct(
                    vec![
                        Field::new(
                            "x",
                            DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                            false,
                        ),
                        Field::new(
                            "y",
                            DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                            false,
                        ),
                    ]
                    .into(),
                )
            }
        }
    }

    /// Create a field for trace data with extension metadata
    pub fn field(&self, name: &str, nullable: bool) -> Field {
        let mut field = Field::new(name, self.storage_type(), nullable);

        // Add extension name (required)
        let mut metadata = HashMap::new();
        metadata.insert(
            ARROW_EXTENSION_NAME.to_string(),
            self.extension_name.clone(),
        );

        field.set_metadata(metadata);
        field
    }
}

/// Extension trait for DataType to easily check for fricon custom types
pub trait FriconDataTypeExt {
    /// Check if the data type is a complex number
    fn is_complex(&self) -> bool;

    /// Check if the data type is a trace
    fn is_trace(&self) -> bool;

    /// Get the trace variant if it's a trace type
    fn trace_variant(&self) -> Option<TraceVariant>;
}

/// Extension trait for Field to easily check for fricon custom types via metadata
pub trait FriconFieldExt {
    /// Check if the field is a complex number extension type
    fn is_complex_extension(&self) -> bool;

    /// Check if the field is a trace extension type
    fn is_trace_extension(&self) -> bool;

    /// Get the trace variant if it's a trace extension type
    fn trace_variant_extension(&self) -> Option<TraceVariant>;
}

impl FriconDataTypeExt for DataType {
    fn is_complex(&self) -> bool {
        match self {
            DataType::Struct(fields) => {
                fields.len() == 2
                    && fields[0].name() == "real"
                    && fields[0].data_type() == &DataType::Float64
                    && fields[1].name() == "imag"
                    && fields[1].data_type() == &DataType::Float64
            }
            _ => false,
        }
    }

    fn is_trace(&self) -> bool {
        self.trace_variant().is_some()
    }

    fn trace_variant(&self) -> Option<TraceVariant> {
        match self {
            DataType::List(field) => {
                // Check if it's a simple list trace (list of f64)
                if field.data_type() == &DataType::Float64 {
                    return Some(TraceVariant::SimpleList);
                }
            }
            DataType::Struct(fields) => {
                if fields.len() == 3 {
                    // Check for fixed step: x0, step, y
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == &["x0", "step", "y"] {
                        return Some(TraceVariant::FixedStep);
                    }
                } else if fields.len() == 2 {
                    // Check for variable step: x, y
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == &["x", "y"] {
                        return Some(TraceVariant::VariableStep);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl FriconFieldExt for Field {
    fn is_complex_extension(&self) -> bool {
        self.metadata()
            .get(ARROW_EXTENSION_NAME)
            .map(|name| name == "fricon.complex")
            .unwrap_or(false)
    }

    fn is_trace_extension(&self) -> bool {
        self.metadata()
            .get(ARROW_EXTENSION_NAME)
            .map(|name| name.starts_with("fricon.trace."))
            .unwrap_or(false)
    }

    fn trace_variant_extension(&self) -> Option<TraceVariant> {
        let extension_name = self.metadata().get(ARROW_EXTENSION_NAME)?;

        match extension_name.as_str() {
            "fricon.trace.simple_list" => Some(TraceVariant::SimpleList),
            "fricon.trace.fixed_step" => Some(TraceVariant::FixedStep),
            "fricon.trace.variable_step" => Some(TraceVariant::VariableStep),
            _ => None,
        }
    }
}

/// Utility functions for creating fricon-specific schemas
pub struct FriconSchemaBuilder {
    fields: Vec<Field>,
}

impl FriconSchemaBuilder {
    /// Create a new schema builder
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add a complex field
    pub fn add_complex(mut self, name: &str, nullable: bool) -> Self {
        self.fields.push(ComplexType::field(name, nullable));
        self
    }

    /// Add a simple list trace field
    pub fn add_simple_list_trace(mut self, name: &str, nullable: bool) -> Self {
        self.fields
            .push(TraceType::simple_list().field(name, nullable));
        self
    }

    /// Add a fixed step trace field
    pub fn add_fixed_step_trace(mut self, name: &str, nullable: bool) -> Self {
        self.fields
            .push(TraceType::fixed_step().field(name, nullable));
        self
    }

    /// Add a variable step trace field
    pub fn add_variable_step_trace(mut self, name: &str, nullable: bool) -> Self {
        self.fields
            .push(TraceType::variable_step().field(name, nullable));
        self
    }

    /// Add a standard field
    pub fn add_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Build the schema
    pub fn build(self) -> Schema {
        Schema::new(self.fields)
    }
}

impl Default for FriconSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_type() {
        let _complex_type = ComplexType::new();
        let data_type = ComplexType::storage_type();

        assert!(data_type.is_complex());
        assert!(!data_type.is_trace());

        let field = ComplexType::field("complex_field", false);
        assert_eq!(field.name(), "complex_field");
        assert_eq!(field.data_type(), &data_type);

        // Check extension metadata
        assert_eq!(
            field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.complex".to_string())
        );
    }

    #[test]
    fn test_trace_types() {
        // Test simple list trace
        let simple_trace = TraceType::simple_list();
        let simple_data_type = simple_trace.storage_type();
        assert!(simple_data_type.is_trace());
        assert_eq!(
            simple_data_type.trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        // Test fixed step trace
        let fixed_trace = TraceType::fixed_step();
        let fixed_data_type = fixed_trace.storage_type();
        assert!(fixed_data_type.is_trace());
        assert_eq!(
            fixed_data_type.trace_variant(),
            Some(TraceVariant::FixedStep)
        );

        // Test variable step trace
        let variable_trace = TraceType::variable_step();
        let variable_data_type = variable_trace.storage_type();
        assert!(variable_data_type.is_trace());
        assert_eq!(
            variable_data_type.trace_variant(),
            Some(TraceVariant::VariableStep)
        );

        // Test extension metadata
        let simple_field = TraceType::simple_list().field("simple", false);
        assert_eq!(
            simple_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.simple_list".to_string())
        );

        let fixed_field = TraceType::fixed_step().field("fixed", false);
        assert_eq!(
            fixed_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.fixed_step".to_string())
        );

        let variable_field = TraceType::variable_step().field("variable", false);
        assert_eq!(
            variable_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.variable_step".to_string())
        );
    }

    #[test]
    fn test_schema_builder() {
        let schema = FriconSchemaBuilder::new()
            .add_complex("complex_data", false)
            .add_simple_list_trace("simple_trace", true)
            .add_fixed_step_trace("fixed_trace", false)
            .add_variable_step_trace("variable_trace", true)
            .add_field(Field::new("regular_field", DataType::Int32, false))
            .build();

        assert_eq!(schema.fields().len(), 5);

        // Check complex field
        let complex_field = schema.field_with_name("complex_data").unwrap();
        assert!(complex_field.data_type().is_complex());

        // Check trace fields
        let simple_field = schema.field_with_name("simple_trace").unwrap();
        assert_eq!(
            simple_field.data_type().trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        let fixed_field = schema.field_with_name("fixed_trace").unwrap();
        assert_eq!(
            fixed_field.data_type().trace_variant(),
            Some(TraceVariant::FixedStep)
        );

        let variable_field = schema.field_with_name("variable_trace").unwrap();
        assert_eq!(
            variable_field.data_type().trace_variant(),
            Some(TraceVariant::VariableStep)
        );
    }

    #[test]
    fn test_trace_variant_display() {
        assert_eq!(TraceVariant::SimpleList.to_string(), "simple_list");
        assert_eq!(TraceVariant::FixedStep.to_string(), "fixed_step");
        assert_eq!(TraceVariant::VariableStep.to_string(), "variable_step");
    }
}

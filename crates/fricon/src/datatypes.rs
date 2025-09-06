//! Custom Arrow data type extensions for fricon
//!
//! This module defines custom Arrow data types specific to fricon using
//! Arrow's official extension type specification:
//! - Complex: Complex numbers with real and imaginary parts
//! - Trace: Different types of trace data with various x-axis representations
//!
//! Extension types use the ARROW:extension:name and ARROW:extension:metadata
//! keys in Field metadata to identify custom data types.

use arrow::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Extension metadata keys for Arrow extension types
pub const ARROW_EXTENSION_NAME: &str = "ARROW:extension:name";

/// Complex number data type
///
/// Represents complex numbers with real and imaginary parts as f64 values
/// using Arrow's extension type specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ComplexType {
    // ComplexType is a unit struct since the storage type is always the same
}

impl ComplexType {
    /// Create a new complex type
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the extension name
    #[must_use]
    pub fn extension_name() -> &'static str {
        "fricon.complex"
    }

    /// Get the Arrow data type representation (storage type)
    #[must_use]
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
    #[must_use]
    pub fn field(name: &str, nullable: bool) -> Field {
        let mut field = Field::new(name, Self::storage_type(), nullable);

        // Add extension name (required)
        let mut metadata = HashMap::new();
        metadata.insert(
            ARROW_EXTENSION_NAME.to_string(),
            Self::extension_name().to_string(),
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

impl TraceVariant {
    /// Create a field for this trace variant with extension metadata
    #[must_use]
    pub fn field(self, name: &str, y_item_type: DataType, nullable: bool) -> Field {
        let mut field = Field::new(name, self.storage_type(y_item_type), nullable);

        // Add extension name (required)
        let mut metadata = HashMap::new();
        metadata.insert(
            ARROW_EXTENSION_NAME.to_string(),
            self.extension_name().to_string(),
        );

        field.set_metadata(metadata);
        field
    }

    /// Get the extension name for this variant
    #[must_use]
    pub fn extension_name(self) -> &'static str {
        match self {
            TraceVariant::SimpleList => "fricon.trace.simple_list",
            TraceVariant::FixedStep => "fricon.trace.fixed_step",
            TraceVariant::VariableStep => "fricon.trace.variable_step",
        }
    }

    /// Get the Arrow data type representation (storage type)
    fn storage_type(self, y_item_type: DataType) -> DataType {
        match self {
            TraceVariant::SimpleList => {
                // Simple list: just y values as list of specified data type
                DataType::List(Arc::new(Field::new_list_field(y_item_type.clone(), false)))
            }
            TraceVariant::FixedStep => {
                // Fixed step: struct with x0, step, and y values
                DataType::Struct(
                    vec![
                        Field::new("x0", DataType::Float64, false),
                        Field::new("step", DataType::Float64, false),
                        Field::new(
                            "y",
                            DataType::List(Arc::new(Field::new_list_field(
                                y_item_type.clone(),
                                false,
                            ))),
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
                            DataType::List(Arc::new(Field::new_list_field(
                                DataType::Float64,
                                false,
                            ))),
                            false,
                        ),
                        Field::new(
                            "y",
                            DataType::List(Arc::new(Field::new_list_field(
                                y_item_type.clone(),
                                false,
                            ))),
                            false,
                        ),
                    ]
                    .into(),
                )
            }
        }
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceType {
    /// Variant of the trace
    variant: TraceVariant,
    /// Data type for y list items
    y_item_type: DataType,
}

impl TraceType {
    /// Create a new simple list trace type
    #[must_use]
    pub fn simple_list(y_item_type: DataType) -> Self {
        Self {
            variant: TraceVariant::SimpleList,
            y_item_type,
        }
    }

    /// Create a new fixed step trace type
    #[must_use]
    pub fn fixed_step(y_item_type: DataType) -> Self {
        Self {
            variant: TraceVariant::FixedStep,
            y_item_type,
        }
    }

    /// Create a new variable step trace type
    #[must_use]
    pub fn variable_step(y_item_type: DataType) -> Self {
        Self {
            variant: TraceVariant::VariableStep,
            y_item_type,
        }
    }

    /// Get the extension name
    #[must_use]
    pub fn extension_name(&self) -> &str {
        self.variant.extension_name()
    }

    /// Get the trace variant
    #[must_use]
    pub fn variant(&self) -> TraceVariant {
        self.variant
    }

    /// Get the Arrow data type representation (storage type)
    #[must_use]
    pub fn storage_type(&self) -> DataType {
        self.variant.storage_type(self.y_item_type.clone())
    }

    /// Create a field for trace data with extension metadata
    #[must_use]
    pub fn field(&self, name: &str, nullable: bool) -> Field {
        self.variant.field(name, self.y_item_type.clone(), nullable)
    }

    /// Convenience method to create a simple list trace field
    #[must_use]
    pub fn simple_list_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        TraceVariant::SimpleList.field(name, y_item_type, nullable)
    }

    /// Convenience method to create a fixed step trace field
    #[must_use]
    pub fn fixed_step_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        TraceVariant::FixedStep.field(name, y_item_type, nullable)
    }

    /// Convenience method to create a variable step trace field
    #[must_use]
    pub fn variable_step_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        TraceVariant::VariableStep.field(name, y_item_type, nullable)
    }
}

/// Unified extension trait for checking fricon custom types
pub trait FriconTypeExt {
    /// Check if the data type is a complex number
    fn is_complex(&self) -> bool;

    /// Check if the data type is a trace
    fn is_trace(&self) -> bool;

    /// Get the trace variant if it's a trace type
    fn trace_variant(&self) -> Option<TraceVariant>;
}

impl FriconTypeExt for DataType {
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
            DataType::List(_field) => {
                // Check if it's a simple list trace (any list can be a simple list trace)
                return Some(TraceVariant::SimpleList);
            }
            DataType::Struct(fields) => {
                if fields.len() == 3 {
                    // Check for fixed step: x0, step, y (with y being a list)
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == ["x0", "step", "y"]
                        && let DataType::List(_) = fields[2].data_type()
                    {
                        return Some(TraceVariant::FixedStep);
                    }
                } else if fields.len() == 2 {
                    // Check for variable step: x, y (both being lists)
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == ["x", "y"]
                        && let (DataType::List(_), DataType::List(_)) =
                            (&fields[0].data_type(), &fields[1].data_type())
                    {
                        return Some(TraceVariant::VariableStep);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl FriconTypeExt for Field {
    fn is_complex(&self) -> bool {
        // Check extension metadata first, then fall back to structural check
        self.metadata().get(ARROW_EXTENSION_NAME).map_or_else(
            || self.data_type().is_complex(),
            |name| name == "fricon.complex",
        )
    }

    fn is_trace(&self) -> bool {
        // Check extension metadata first, then fall back to structural check
        self.metadata().get(ARROW_EXTENSION_NAME).map_or_else(
            || self.data_type().is_trace(),
            |name| name.starts_with("fricon.trace."),
        )
    }

    fn trace_variant(&self) -> Option<TraceVariant> {
        // Check extension metadata first
        if let Some(extension_name) = self.metadata().get(ARROW_EXTENSION_NAME) {
            match extension_name.as_str() {
                "fricon.trace.simple_list" => return Some(TraceVariant::SimpleList),
                "fricon.trace.fixed_step" => return Some(TraceVariant::FixedStep),
                "fricon.trace.variable_step" => return Some(TraceVariant::VariableStep),
                _ => {}
            }
        }

        // Fall back to structural check
        self.data_type().trace_variant()
    }
}

// helper removed — construct Schema::new(...) directly where needed

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::Schema;

    #[test]
    fn test_complex_type() {
        let data_type = ComplexType::storage_type();

        assert!(data_type.is_complex());
        assert!(!data_type.is_trace());

        let field = ComplexType::field("complex_field", false);
        assert_eq!(field.name(), "complex_field");
        assert_eq!(field.data_type(), &data_type);
        assert!(field.is_complex());
        assert!(!field.is_trace());

        // Check extension metadata
        assert_eq!(
            field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.complex".to_string())
        );

        // Test new() method and extension_name()
        let _complex_type = ComplexType::new();
        assert_eq!(ComplexType::extension_name(), "fricon.complex");
    }

    #[test]
    fn test_trace_types() {
        // Test simple list trace
        let simple_trace = TraceType::simple_list(DataType::Float64);
        let simple_data_type = simple_trace.storage_type();
        assert!(simple_data_type.is_trace());
        assert_eq!(
            simple_data_type.trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        // Test fixed step trace
        let fixed_trace = TraceType::fixed_step(DataType::Float64);
        let fixed_data_type = fixed_trace.storage_type();
        assert!(fixed_data_type.is_trace());
        assert_eq!(
            fixed_data_type.trace_variant(),
            Some(TraceVariant::FixedStep)
        );

        // Test variable step trace
        let variable_trace = TraceType::variable_step(DataType::Float64);
        let variable_data_type = variable_trace.storage_type();
        assert!(variable_data_type.is_trace());
        assert_eq!(
            variable_data_type.trace_variant(),
            Some(TraceVariant::VariableStep)
        );

        // Test extension metadata
        let simple_field = TraceType::simple_list(DataType::Float64).field("simple", false);
        assert_eq!(
            simple_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.simple_list".to_string())
        );
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_variant(), Some(TraceVariant::SimpleList));

        let fixed_field = TraceType::fixed_step(DataType::Float64).field("fixed", false);
        assert_eq!(
            fixed_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.fixed_step".to_string())
        );
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_variant(), Some(TraceVariant::FixedStep));

        let variable_field = TraceType::variable_step(DataType::Float64).field("variable", false);
        assert_eq!(
            variable_field.metadata().get(ARROW_EXTENSION_NAME),
            Some(&"fricon.trace.variable_step".to_string())
        );
        assert!(variable_field.is_trace());
        assert_eq!(
            variable_field.trace_variant(),
            Some(TraceVariant::VariableStep)
        );
    }

    #[test]
    fn test_trace_variant_field_creation() {
        // Test direct field creation from TraceVariant
        let simple_field = TraceVariant::SimpleList.field("simple", DataType::Float64, false);
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_variant(), Some(TraceVariant::SimpleList));

        let fixed_field = TraceVariant::FixedStep.field("fixed", DataType::Float64, false);
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_variant(), Some(TraceVariant::FixedStep));

        let variable_field = TraceVariant::VariableStep.field("variable", DataType::Float64, false);
        assert!(variable_field.is_trace());
        assert_eq!(
            variable_field.trace_variant(),
            Some(TraceVariant::VariableStep)
        );
    }

    #[test]
    fn test_trace_type_convenience_methods() {
        // Test convenience methods
        let simple_field = TraceType::simple_list_field("simple", DataType::Float64, false);
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_variant(), Some(TraceVariant::SimpleList));

        let fixed_field = TraceType::fixed_step_field("fixed", DataType::Float64, false);
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_variant(), Some(TraceVariant::FixedStep));

        let variable_field = TraceType::variable_step_field("variable", DataType::Float64, false);
        assert!(variable_field.is_trace());
        assert_eq!(
            variable_field.trace_variant(),
            Some(TraceVariant::VariableStep)
        );
    }

    #[test]
    fn test_schema_creation() {
        // Test creating schemas directly without builder
        let schema = Schema::new(vec![
            ComplexType::field("complex_data", false),
            TraceType::simple_list_field("simple_trace", DataType::Float64, true),
            TraceType::fixed_step_field("fixed_trace", DataType::Float64, false),
            TraceType::variable_step_field("variable_trace", DataType::Float64, true),
            Field::new("regular_field", DataType::Int32, false),
        ]);

        assert_eq!(schema.fields().len(), 5);

        // Check complex field
        let complex_field = schema.field_with_name("complex_data").unwrap();
        assert!(complex_field.is_complex());

        // Check trace fields
        let simple_field = schema.field_with_name("simple_trace").unwrap();
        assert_eq!(simple_field.trace_variant(), Some(TraceVariant::SimpleList));

        let fixed_field = schema.field_with_name("fixed_trace").unwrap();
        assert_eq!(fixed_field.trace_variant(), Some(TraceVariant::FixedStep));

        let variable_field = schema.field_with_name("variable_trace").unwrap();
        assert_eq!(
            variable_field.trace_variant(),
            Some(TraceVariant::VariableStep)
        );
    }

    #[test]
    fn test_trace_variant_display() {
        assert_eq!(TraceVariant::SimpleList.to_string(), "simple_list");
        assert_eq!(TraceVariant::FixedStep.to_string(), "fixed_step");
        assert_eq!(TraceVariant::VariableStep.to_string(), "variable_step");
    }

    #[test]
    fn test_trace_with_custom_y_datatype() {
        // Test creating traces with different y data types
        let simple_int = TraceType::simple_list(DataType::Int32);
        let simple_int_type = simple_int.storage_type();
        assert!(simple_int_type.is_trace());
        assert_eq!(
            simple_int_type.trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        let simple_string = TraceType::simple_list(DataType::Utf8);
        let simple_string_type = simple_string.storage_type();
        assert!(simple_string_type.is_trace());
        assert_eq!(
            simple_string_type.trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        let simple_bool = TraceType::simple_list(DataType::Boolean);
        let simple_bool_type = simple_bool.storage_type();
        assert!(simple_bool_type.is_trace());
        assert_eq!(
            simple_bool_type.trace_variant(),
            Some(TraceVariant::SimpleList)
        );

        // Test fixed step with different y data types
        let fixed_int = TraceType::fixed_step(DataType::Int32);
        let fixed_int_type = fixed_int.storage_type();
        assert!(fixed_int_type.is_trace());
        assert_eq!(
            fixed_int_type.trace_variant(),
            Some(TraceVariant::FixedStep)
        );

        // Test variable step with different y data types
        let variable_string = TraceType::variable_step(DataType::Utf8);
        let variable_string_type = variable_string.storage_type();
        assert!(variable_string_type.is_trace());
        assert_eq!(
            variable_string_type.trace_variant(),
            Some(TraceVariant::VariableStep)
        );

        // Test field creation with custom y data types
        let int_field = TraceType::simple_list_field("int_trace", DataType::Int32, false);
        assert!(int_field.is_trace());
        assert_eq!(int_field.trace_variant(), Some(TraceVariant::SimpleList));

        let string_field = TraceVariant::FixedStep.field("string_trace", DataType::Utf8, false);
        assert!(string_field.is_trace());
        assert_eq!(string_field.trace_variant(), Some(TraceVariant::FixedStep));
    }
}

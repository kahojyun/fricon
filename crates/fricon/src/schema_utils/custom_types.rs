//! Custom Arrow data types for fricon
//!
//! This module centralizes all custom Arrow data type definitions including:
//! - Complex numbers (complex128)
//! - Time series traces (variable and fixed step)

use std::sync::LazyLock;

use arrow::datatypes::{DataType, Field, Fields};
use serde::{Deserialize, Serialize};

/// Information about custom data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTypeInfo {
    pub type_name: String,
    pub description: String,
    pub visualization_support: CustomTypeVisualization,
    pub conversion_hints: Vec<String>,
}

/// Visualization capabilities for custom types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTypeVisualization {
    pub can_plot: bool,
    pub can_group: bool,
    pub can_filter: bool,
    pub recommended_representation: String,
}

/// Get complex128 data type (struct with real and imaginary float64 fields)
pub fn complex128() -> DataType {
    static COMPLEX128: LazyLock<DataType> = LazyLock::new(|| {
        let fields = vec![
            Field::new("real", DataType::Float64, false),
            Field::new("imag", DataType::Float64, false),
        ];
        DataType::Struct(Fields::from(fields))
    });
    COMPLEX128.clone()
}

/// Get trace data type with variable x steps
///
/// Structure: { xs: [f64], ys: [`item_type`] }
#[must_use]
pub fn trace_variable_step(item_type: DataType) -> DataType {
    let fields = vec![
        Field::new("xs", DataType::new_list(DataType::Float64, false), false),
        Field::new("ys", DataType::new_list(item_type, false), false),
    ];
    DataType::Struct(Fields::from(fields))
}

/// Get trace data type with fixed x steps
///
/// Structure: { x0: f64, dx: f64, ys: [`item_type`] }
#[must_use]
pub fn trace_fixed_step(item_type: DataType) -> DataType {
    let fields = vec![
        Field::new("x0", DataType::Float64, false),
        Field::new("dx", DataType::Float64, false),
        Field::new("ys", DataType::new_list(item_type, false), false),
    ];
    DataType::Struct(Fields::from(fields))
}

/// Get trace data type based on whether it has fixed steps
#[must_use]
pub fn trace(item_type: DataType, fixed_step: bool) -> DataType {
    if fixed_step {
        trace_fixed_step(item_type)
    } else {
        trace_variable_step(item_type)
    }
}

/// Check if a `DataType` is the complex128 custom type
#[must_use]
pub fn is_complex_type(data_type: &DataType) -> bool {
    if let DataType::Struct(fields) = data_type {
        fields.len() == 2
            && fields
                .iter()
                .any(|f| f.name() == "real" && *f.data_type() == DataType::Float64)
            && fields
                .iter()
                .any(|f| f.name() == "imag" && *f.data_type() == DataType::Float64)
    } else {
        false
    }
}

/// Check if a `DataType` is a trace custom type (either variant)
#[must_use]
pub fn is_trace_type(data_type: &DataType) -> bool {
    is_trace_variable_step(data_type) || is_trace_fixed_step(data_type)
}

/// Check if a `DataType` is a variable-step trace
#[must_use]
pub fn is_trace_variable_step(data_type: &DataType) -> bool {
    if let DataType::Struct(fields) = data_type {
        fields.len() == 2
            && fields.iter().any(|f| {
                f.name() == "xs" && matches!(f.data_type(), DataType::List(field) if *field.data_type() == DataType::Float64)
            })
            && fields.iter().any(|f| {
                f.name() == "ys" && matches!(f.data_type(), DataType::List(_))
            })
    } else {
        false
    }
}

/// Check if a `DataType` is a fixed-step trace
#[must_use]
pub fn is_trace_fixed_step(data_type: &DataType) -> bool {
    if let DataType::Struct(fields) = data_type {
        fields.len() == 3
            && fields
                .iter()
                .any(|f| f.name() == "x0" && *f.data_type() == DataType::Float64)
            && fields
                .iter()
                .any(|f| f.name() == "dx" && *f.data_type() == DataType::Float64)
            && fields
                .iter()
                .any(|f| f.name() == "ys" && matches!(f.data_type(), DataType::List(_)))
    } else {
        false
    }
}

/// Get the y-axis data type from a trace type
#[must_use]
pub fn get_trace_y_type(trace_type: &DataType) -> Option<DataType> {
    if let DataType::Struct(fields) = trace_type {
        for field in fields {
            if field.name() == "ys"
                && let DataType::List(list_field) = field.data_type()
            {
                return Some(list_field.data_type().clone());
            }
        }
    }
    None
}

/// Get custom type information for a given `DataType`
#[must_use]
pub fn get_custom_type_info(data_type: &DataType) -> Option<CustomTypeInfo> {
    if is_complex_type(data_type) {
        Some(CustomTypeInfo {
            type_name: "complex128".to_string(),
            description: "128-bit complex number with real and imaginary parts".to_string(),
            visualization_support: CustomTypeVisualization {
                can_plot: true,
                can_group: false,
                can_filter: false,
                recommended_representation: "magnitude".to_string(),
            },
            conversion_hints: vec![
                "Visualized as magnitude: sqrt(real² + imag²)".to_string(),
                "Real and imaginary parts accessible separately".to_string(),
            ],
        })
    } else if is_trace_type(data_type) {
        let trace_variant = if is_trace_fixed_step(data_type) {
            "fixed-step"
        } else {
            "variable-step"
        };

        Some(CustomTypeInfo {
            type_name: format!("trace_{trace_variant}"),
            description: format!("Time series trace with {trace_variant} x-axis values"),
            visualization_support: CustomTypeVisualization {
                can_plot: true,
                can_group: false,
                can_filter: false,
                recommended_representation: "time_series".to_string(),
            },
            conversion_hints: vec![
                "Contains x-axis and y-axis data for time series plots".to_string(),
                "Can be expanded into multiple data points for visualization".to_string(),
            ],
        })
    } else {
        None
    }
}

/// Validate that a schema contains only supported custom types
pub fn validate_schema_custom_types(schema: &arrow::datatypes::Schema) -> Result<(), String> {
    for field in schema.fields() {
        if let Err(e) = validate_field_type(field) {
            return Err(format!("Invalid type in field '{}': {}", field.name(), e));
        }
    }
    Ok(())
}

/// Validate that a field type is supported
pub fn validate_field_type(field: &Field) -> Result<(), String> {
    validate_data_type(field.data_type())
}

/// Recursively validate a data type
pub fn validate_data_type(data_type: &DataType) -> Result<(), String> {
    match data_type {
        DataType::Struct(_fields) => {
            // Check if it's a known custom type
            if is_complex_type(data_type) || is_trace_type(data_type) {
                Ok(())
            } else {
                // Validate nested fields for unknown struct types
                if let DataType::Struct(fields) = data_type {
                    for field in fields {
                        validate_data_type(field.data_type())?;
                    }
                }
                Ok(())
            }
        }
        DataType::List(field) | DataType::LargeList(field) => validate_data_type(field.data_type()),
        DataType::FixedSizeList(field, _) => validate_data_type(field.data_type()),
        // All primitive types are valid
        DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float16
        | DataType::Float32
        | DataType::Float64
        | DataType::Utf8
        | DataType::LargeUtf8 => Ok(()),

        // Other types might need case-by-case validation
        _ => Ok(()), // For now, allow other types
    }
}

/// Get all custom type definitions for documentation/introspection
#[must_use]
pub fn get_all_custom_types() -> Vec<(String, DataType, CustomTypeInfo)> {
    vec![
        (
            "complex128".to_string(),
            complex128(),
            get_custom_type_info(&complex128()).unwrap(),
        ),
        (
            "trace_variable_step_f64".to_string(),
            trace_variable_step(DataType::Float64),
            get_custom_type_info(&trace_variable_step(DataType::Float64)).unwrap(),
        ),
        (
            "trace_fixed_step_f64".to_string(),
            trace_fixed_step(DataType::Float64),
            get_custom_type_info(&trace_fixed_step(DataType::Float64)).unwrap(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex128_type() {
        let complex_type = complex128();
        assert!(is_complex_type(&complex_type));
        assert!(!is_trace_type(&complex_type));
    }

    #[test]
    fn test_trace_types() {
        let var_trace = trace_variable_step(DataType::Float64);
        let fixed_trace = trace_fixed_step(DataType::Float64);

        assert!(is_trace_type(&var_trace));
        assert!(is_trace_type(&fixed_trace));
        assert!(is_trace_variable_step(&var_trace));
        assert!(is_trace_fixed_step(&fixed_trace));

        assert!(!is_trace_fixed_step(&var_trace));
        assert!(!is_trace_variable_step(&fixed_trace));

        assert!(!is_complex_type(&var_trace));
        assert!(!is_complex_type(&fixed_trace));
    }

    #[test]
    fn test_trace_y_type_extraction() {
        let trace = trace_variable_step(DataType::Int32);
        let y_type = get_trace_y_type(&trace);
        assert_eq!(y_type, Some(DataType::Int32));
    }

    #[test]
    fn test_custom_type_info() {
        let complex_info = get_custom_type_info(&complex128());
        assert!(complex_info.is_some());

        let info = complex_info.unwrap();
        assert_eq!(info.type_name, "complex128");
        assert!(info.visualization_support.can_plot);
    }
}

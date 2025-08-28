//! Integration tests for `schema_utils` module

use arrow::datatypes::DataType;
use fricon::{
    ColumnDataType,
    schema_utils::{
        classify_data_type,
        custom_types::{complex128, is_complex_type, is_trace_type, trace_variable_step},
    },
};

#[test]
fn test_custom_data_types_work() {
    // Test that custom data types can be created and identified
    let complex_type = complex128();
    assert!(is_complex_type(&complex_type));
    assert!(!is_trace_type(&complex_type));

    let trace_var = trace_variable_step(DataType::Float64);
    assert!(is_trace_type(&trace_var));
    assert!(!is_complex_type(&trace_var));

    println!("✓ Custom data types work correctly");
}

#[test]
fn test_schema_classification() {
    // Test data type classification
    assert_eq!(
        classify_data_type(&DataType::Int64),
        ColumnDataType::Numeric
    );
    assert_eq!(classify_data_type(&DataType::Utf8), ColumnDataType::Text);
    assert_eq!(
        classify_data_type(&DataType::Boolean),
        ColumnDataType::Boolean
    );

    let complex_type = complex128();
    assert_eq!(classify_data_type(&complex_type), ColumnDataType::Complex);

    let trace_type = trace_variable_step(DataType::Float64);
    assert_eq!(classify_data_type(&trace_type), ColumnDataType::Trace);

    println!("✓ Schema classification works correctly");
}

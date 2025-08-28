//! Arrow Schema Utilities
//!
//! This module provides centralized functionality for Arrow schema operations including:
//! - Custom data type definitions (complex numbers, traces)
//! - Schema inspection and analysis
//! - Dataset shape information
//! - Visualization compatibility checking
//! - Sample data extraction

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use arrow::array::Array;
use arrow::datatypes::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};

pub mod compatibility;
pub mod custom_types;
pub mod inspector;

// Re-export commonly used types
pub use compatibility::VisualizationCompatibility;
pub use custom_types::CustomTypeInfo;
pub use inspector::{DatasetShape, SchemaInspector};

// Re-export custom type functions for convenience
pub use custom_types::{is_complex_type, is_trace_type};

/// Represents different types of column values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ColumnValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Simplified data types for UI consumption
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnDataType {
    Numeric,
    Text,
    Boolean,
    Complex,
    Trace,
    List,
    Other,
}

/// Detailed column information
#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnDataType,
    #[serde(skip)] // Skip serialization of Arrow DataType
    pub arrow_type: DataType,
    pub is_index_column: bool,
    pub nullable: bool,
    pub unique_value_count: Option<usize>,
    pub sample_values: Option<Vec<ColumnValue>>,
}

// Manual implementation of Deserialize to avoid Default requirement on DataType
impl<'de> serde::Deserialize<'de> for ColumnInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ColumnInfoHelper {
            name: String,
            data_type: ColumnDataType,
            is_index_column: bool,
            nullable: bool,
            unique_value_count: Option<usize>,
            sample_values: Option<Vec<ColumnValue>>,
        }

        let helper = ColumnInfoHelper::deserialize(deserializer)?;
        Ok(ColumnInfo {
            name: helper.name,
            data_type: helper.data_type,
            arrow_type: DataType::Utf8, // Default fallback
            is_index_column: helper.is_index_column,
            nullable: helper.nullable,
            unique_value_count: helper.unique_value_count,
            sample_values: helper.sample_values,
        })
    }
}

impl Default for ColumnInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            data_type: ColumnDataType::Other,
            arrow_type: DataType::Utf8, // Use a simple default
            is_index_column: false,
            nullable: true,
            unique_value_count: None,
            sample_values: None,
        }
    }
}

/// Complete dataset schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSchemaInfo {
    pub columns: Vec<ColumnInfo>,
    pub shape: DatasetShape,
    pub index_columns: Vec<String>,
    pub custom_types: HashMap<String, CustomTypeInfo>,
    pub visualization_compatibility: VisualizationCompatibility,
}

/// Inspect dataset schema from Arrow IPC file
pub fn inspect_dataset_schema(path: &Path, index_columns: &[String]) -> Result<DatasetSchemaInfo> {
    let inspector = SchemaInspector::new();
    inspector.inspect_schema(path, index_columns)
}

/// Get column information for a specific field
#[must_use]
pub fn get_column_info(
    field: &Field,
    is_index_column: bool,
    sample_values: Option<Vec<ColumnValue>>,
) -> ColumnInfo {
    let data_type = classify_data_type(field.data_type());

    ColumnInfo {
        name: field.name().clone(),
        data_type,
        arrow_type: field.data_type().clone(),
        is_index_column,
        nullable: field.is_nullable(),
        unique_value_count: sample_values.as_ref().map(std::vec::Vec::len),
        sample_values,
    }
}

/// Classify Arrow data type into simplified categories
#[must_use]
pub fn classify_data_type(arrow_type: &DataType) -> ColumnDataType {
    match arrow_type {
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float16
        | DataType::Float32
        | DataType::Float64 => ColumnDataType::Numeric,

        DataType::Utf8 | DataType::LargeUtf8 => ColumnDataType::Text,

        DataType::Boolean => ColumnDataType::Boolean,

        DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(_, _) => {
            ColumnDataType::List
        }

        DataType::Struct(_fields) => {
            if is_complex_type(arrow_type) {
                ColumnDataType::Complex
            } else if is_trace_type(arrow_type) {
                ColumnDataType::Trace
            } else {
                ColumnDataType::Other
            }
        }

        _ => ColumnDataType::Other,
    }
}

/// Extract unique values from a column with limit
pub fn extract_unique_values(
    path: &Path,
    column_name: &str,
    limit: usize,
) -> Result<Vec<ColumnValue>> {
    let inspector = SchemaInspector::new();
    inspector.extract_unique_values(path, column_name, limit)
}

/// Check if data type is supported for visualization
#[must_use]
pub fn is_visualization_supported(data_type: &ColumnDataType) -> bool {
    matches!(
        data_type,
        ColumnDataType::Numeric
            | ColumnDataType::Text
            | ColumnDataType::Boolean
            | ColumnDataType::Complex // complex numbers can be visualized as magnitude/phase
    )
}

/// Get schema summary information
#[must_use]
pub fn get_schema_summary(schema: &Schema) -> SchemaSummary {
    let mut summary = SchemaSummary {
        total_columns: schema.fields().len(),
        numeric_columns: 0,
        text_columns: 0,
        boolean_columns: 0,
        complex_columns: 0,
        trace_columns: 0,
        list_columns: 0,
        other_columns: 0,
    };

    for field in schema.fields() {
        match classify_data_type(field.data_type()) {
            ColumnDataType::Numeric => summary.numeric_columns += 1,
            ColumnDataType::Text => summary.text_columns += 1,
            ColumnDataType::Boolean => summary.boolean_columns += 1,
            ColumnDataType::Complex => summary.complex_columns += 1,
            ColumnDataType::Trace => summary.trace_columns += 1,
            ColumnDataType::List => summary.list_columns += 1,
            ColumnDataType::Other => summary.other_columns += 1,
        }
    }

    summary
}

/// Schema summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSummary {
    pub total_columns: usize,
    pub numeric_columns: usize,
    pub text_columns: usize,
    pub boolean_columns: usize,
    pub complex_columns: usize,
    pub trace_columns: usize,
    pub list_columns: usize,
    pub other_columns: usize,
}

/// Helper function to extract column value at specific row index
pub fn extract_column_value_at(column: &dyn Array, row_idx: usize) -> Result<Option<ColumnValue>> {
    use arrow::array::{
        Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array,
        Int64Array, LargeStringArray, StringArray, StructArray, UInt8Array, UInt16Array,
        UInt32Array, UInt64Array,
    };

    if column.is_null(row_idx) {
        return Ok(None);
    }

    let value = match column.data_type() {
        DataType::Int8 => {
            let array = column.as_any().downcast_ref::<Int8Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::Int16 => {
            let array = column.as_any().downcast_ref::<Int16Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::Int32 => {
            let array = column.as_any().downcast_ref::<Int32Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::Int64 => {
            let array = column.as_any().downcast_ref::<Int64Array>().unwrap();
            #[allow(clippy::cast_precision_loss)]
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::UInt8 => {
            let array = column.as_any().downcast_ref::<UInt8Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::UInt16 => {
            let array = column.as_any().downcast_ref::<UInt16Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::UInt32 => {
            let array = column.as_any().downcast_ref::<UInt32Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::UInt64 => {
            let array = column.as_any().downcast_ref::<UInt64Array>().unwrap();
            #[allow(clippy::cast_precision_loss)]
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::Float32 => {
            let array = column.as_any().downcast_ref::<Float32Array>().unwrap();
            ColumnValue::Number(f64::from(array.value(row_idx)))
        }
        DataType::Float64 => {
            let array = column.as_any().downcast_ref::<Float64Array>().unwrap();
            ColumnValue::Number(array.value(row_idx))
        }
        DataType::Utf8 => {
            let array = column.as_any().downcast_ref::<StringArray>().unwrap();
            ColumnValue::String(array.value(row_idx).to_string())
        }
        DataType::LargeUtf8 => {
            let array = column.as_any().downcast_ref::<LargeStringArray>().unwrap();
            ColumnValue::String(array.value(row_idx).to_string())
        }
        DataType::Boolean => {
            let array = column.as_any().downcast_ref::<BooleanArray>().unwrap();
            ColumnValue::Boolean(array.value(row_idx))
        }
        DataType::Struct(_fields) => {
            // Handle complex numbers
            if is_complex_type(column.data_type()) {
                let array = column.as_any().downcast_ref::<StructArray>().unwrap();
                let real_array = array
                    .column_by_name("real")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .unwrap();
                let imag_array = array
                    .column_by_name("imag")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .unwrap();

                let real = real_array.value(row_idx);
                let imag = imag_array.value(row_idx);
                let magnitude = (real * real + imag * imag).sqrt();

                ColumnValue::Number(magnitude) // Use magnitude for visualization
            } else {
                return Ok(None); // Other struct types not supported
            }
        }
        _ => return Ok(None), // Unsupported type
    };

    Ok(Some(value))
}

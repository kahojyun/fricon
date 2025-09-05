//! Plot configuration generation from dataset schemas
//!
//! This module provides functionality to generate plot configurations
//! from Arrow dataset schemas, analyzing column types and suggesting
//! appropriate visualization settings.

use crate::TraceVariant;
use crate::datatypes::FriconTypeExt;
use arrow::datatypes::{DataType, Field, SchemaRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plot configuration for a dataset column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnPlotConfig {
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: String,
    /// Whether this column can be used for X-axis
    pub can_be_x_axis: bool,
    /// Whether this column can be used for Y-axis
    pub can_be_y_axis: bool,
    /// Suggested plot types for this column
    pub suggested_plot_types: Vec<PlotType>,
    /// Additional column-specific settings
    pub settings: HashMap<String, String>,
}

/// Plot configuration for an entire dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetPlotConfig {
    /// Dataset name
    pub dataset_name: String,
    /// Configuration for each column
    pub columns: Vec<ColumnPlotConfig>,
    /// Overall dataset settings
    pub settings: HashMap<String, String>,
}

/// Supported plot types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotType {
    /// Line plot
    Line,
    /// Scatter plot
    Scatter,
    /// Histogram
    Histogram,
    /// Heatmap
    Heatmap,
}

impl std::fmt::Display for PlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotType::Line => write!(f, "line"),
            PlotType::Scatter => write!(f, "scatter"),
            PlotType::Histogram => write!(f, "histogram"),
            PlotType::Heatmap => write!(f, "heatmap"),
        }
    }
}

/// Error type for plot configuration generation
#[derive(Debug, thiserror::Error)]
pub enum PlotConfigError {
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
}

/// Generate plot configuration from an Arrow schema
pub fn generate_plot_config(dataset_name: &str, schema: &SchemaRef) -> DatasetPlotConfig {
    let mut columns = Vec::new();

    for field in schema.fields() {
        let column_config = generate_column_config(field);
        columns.push(column_config);
    }

    DatasetPlotConfig {
        dataset_name: dataset_name.to_string(),
        columns,
        settings: HashMap::new(),
    }
}

/// Generate plot configuration for a single column
fn generate_column_config(field: &Field) -> ColumnPlotConfig {
    let data_type = field.data_type();
    let type_name = format!("{data_type:?}");

    let (can_be_x_axis, can_be_y_axis, suggested_plot_types) = if field.is_complex() {
        // Complex numbers can be used for both axes (magnitude/phase or real/imaginary)
        (true, true, vec![PlotType::Scatter, PlotType::Heatmap])
    } else if field.is_trace() {
        // Trace data is typically plotted as line or scatter
        match field.trace_variant() {
            Some(TraceVariant::SimpleList) => {
                (false, true, vec![PlotType::Line, PlotType::Scatter])
            }
            Some(TraceVariant::FixedStep) => (true, true, vec![PlotType::Line, PlotType::Scatter]),
            Some(TraceVariant::VariableStep) => {
                (true, true, vec![PlotType::Line, PlotType::Scatter])
            }
            None => (false, false, Vec::new()),
        }
    } else {
        match data_type {
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
            | DataType::Float64 => {
                // Numeric types can be used for both axes
                (
                    true,
                    true,
                    vec![PlotType::Line, PlotType::Scatter, PlotType::Histogram],
                )
            }
            DataType::Boolean => {
                // Boolean types are typically categorical
                (true, false, vec![PlotType::Histogram])
            }
            DataType::Utf8 | DataType::LargeUtf8 => {
                // String types are categorical
                (true, false, vec![PlotType::Histogram])
            }
            DataType::Date32 | DataType::Date64 | DataType::Timestamp(_, _) => {
                // Date/time types can be X-axis
                (true, false, vec![PlotType::Line, PlotType::Scatter])
            }
            _ => {
                // Other types are not directly plottable
                (false, false, Vec::new())
            }
        }
    };

    let mut settings = HashMap::new();
    settings.insert("nullable".to_string(), field.is_nullable().to_string());

    // Add specific settings for fricon data types
    if data_type.is_complex() {
        settings.insert("complex".to_string(), "true".to_string());
    } else if data_type.is_trace() {
        if let Some(variant) = data_type.trace_variant() {
            settings.insert("trace".to_string(), "true".to_string());
            settings.insert("trace_variant".to_string(), variant.to_string());
        }
    }

    ColumnPlotConfig {
        name: field.name().clone(),
        data_type: type_name,
        can_be_x_axis,
        can_be_y_axis,
        suggested_plot_types,
        settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::{ComplexType, FriconTypeExt, TraceType, create_fricon_schema};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_schema() -> SchemaRef {
        let fields = vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("value", DataType::Float64, true),
            Field::new(
                "timestamp",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None),
                false,
            ),
            Field::new("category", DataType::Utf8, true),
        ];

        Arc::new(Schema::new(fields))
    }

    #[test]
    fn test_generate_plot_config() {
        let schema = create_test_schema();
        let config = generate_plot_config("test_dataset", &schema);

        assert_eq!(config.dataset_name, "test_dataset");
        assert_eq!(config.columns.len(), 5);

        // Check ID column (Int32)
        let id_col = &config.columns[0];
        assert_eq!(id_col.name, "id");
        assert_eq!(id_col.data_type, "Int32");
        assert!(id_col.can_be_x_axis);
        assert!(id_col.can_be_y_axis);
        assert!(id_col.suggested_plot_types.contains(&PlotType::Line));
        assert!(id_col.suggested_plot_types.contains(&PlotType::Scatter));

        // Check name column (Utf8)
        let name_col = &config.columns[1];
        assert_eq!(name_col.name, "name");
        assert_eq!(name_col.data_type, "Utf8");
        assert!(name_col.can_be_x_axis);
        assert!(!name_col.can_be_y_axis);
        assert!(name_col.suggested_plot_types.contains(&PlotType::Histogram));

        // Check value column (Float64)
        let value_col = &config.columns[2];
        assert_eq!(value_col.name, "value");
        assert_eq!(value_col.data_type, "Float64");
        assert!(value_col.can_be_x_axis);
        assert!(value_col.can_be_y_axis);
        assert_eq!(
            value_col.settings.get("nullable"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_plot_type_display() {
        assert_eq!(PlotType::Line.to_string(), "line");
        assert_eq!(PlotType::Scatter.to_string(), "scatter");
        assert_eq!(PlotType::Histogram.to_string(), "histogram");
        assert_eq!(PlotType::Heatmap.to_string(), "heatmap");
    }

    #[test]
    fn test_column_config_generation() {
        let field = Field::new("test_int", DataType::Int64, false);
        let config = generate_column_config(&field);

        assert_eq!(config.name, "test_int");
        assert_eq!(config.data_type, "Int64");
        assert!(config.can_be_x_axis);
        assert!(config.can_be_y_axis);
        assert!(config.suggested_plot_types.contains(&PlotType::Line));
        assert!(config.suggested_plot_types.contains(&PlotType::Scatter));
        assert_eq!(config.settings.get("nullable"), Some(&"false".to_string()));
    }

    #[test]
    fn test_complex_type_plot_config() {
        let field = ComplexType::field("complex_data", false);
        let config = generate_column_config(&field);

        assert_eq!(config.name, "complex_data");
        assert!(config.can_be_x_axis);
        assert!(config.can_be_y_axis);
        assert!(config.suggested_plot_types.contains(&PlotType::Scatter));
        assert!(config.suggested_plot_types.contains(&PlotType::Heatmap));
        assert_eq!(config.settings.get("complex"), Some(&"true".to_string()));
        assert_eq!(config.settings.get("nullable"), Some(&"false".to_string()));
    }

    #[test]
    fn test_simple_list_trace_plot_config() {
        let field = TraceType::simple_list_field("simple_trace", false);
        let config = generate_column_config(&field);

        assert_eq!(config.name, "simple_trace");
        assert!(!config.can_be_x_axis);
        assert!(config.can_be_y_axis);
        assert!(config.suggested_plot_types.contains(&PlotType::Line));
        assert!(config.suggested_plot_types.contains(&PlotType::Scatter));
        assert_eq!(config.settings.get("trace"), Some(&"true".to_string()));
        assert_eq!(
            config.settings.get("trace_variant"),
            Some(&"simple_list".to_string())
        );
    }

    #[test]
    fn test_fixed_step_trace_plot_config() {
        let field = TraceType::fixed_step_field("fixed_trace", true);
        let config = generate_column_config(&field);

        assert_eq!(config.name, "fixed_trace");
        assert!(config.can_be_x_axis);
        assert!(config.can_be_y_axis);
        assert!(config.suggested_plot_types.contains(&PlotType::Line));
        assert!(config.suggested_plot_types.contains(&PlotType::Scatter));
        assert_eq!(config.settings.get("trace"), Some(&"true".to_string()));
        assert_eq!(
            config.settings.get("trace_variant"),
            Some(&"fixed_step".to_string())
        );
        assert_eq!(config.settings.get("nullable"), Some(&"true".to_string()));
    }

    #[test]
    fn test_variable_step_trace_plot_config() {
        let field = TraceType::variable_step_field("variable_trace", false);
        let config = generate_column_config(&field);

        assert_eq!(config.name, "variable_trace");
        assert!(config.can_be_x_axis);
        assert!(config.can_be_y_axis);
        assert!(config.suggested_plot_types.contains(&PlotType::Line));
        assert!(config.suggested_plot_types.contains(&PlotType::Scatter));
        assert_eq!(config.settings.get("trace"), Some(&"true".to_string()));
        assert_eq!(
            config.settings.get("trace_variant"),
            Some(&"variable_step".to_string())
        );
    }

    #[test]
    fn test_mixed_schema_plot_config() {
        let schema = create_fricon_schema(vec![
            ComplexType::field("complex_col", false),
            TraceType::simple_list_field("simple_trace", true),
            TraceType::fixed_step_field("fixed_trace", false),
            TraceType::variable_step_field("variable_trace", true),
            Field::new("regular_int", DataType::Int32, false),
        ]);

        let schema_ref = Arc::new(schema);
        let config = generate_plot_config("mixed_dataset", &schema_ref);

        assert_eq!(config.columns.len(), 5);

        // Check complex column
        let complex_col = &config.columns[0];
        assert_eq!(complex_col.name, "complex_col");
        assert!(complex_col.can_be_x_axis);
        assert!(complex_col.can_be_y_axis);
        assert_eq!(
            complex_col.settings.get("complex"),
            Some(&"true".to_string())
        );

        // Check trace columns
        let simple_trace = &config.columns[1];
        assert_eq!(simple_trace.name, "simple_trace");
        assert!(!simple_trace.can_be_x_axis);
        assert!(simple_trace.can_be_y_axis);
        assert_eq!(
            simple_trace.settings.get("trace_variant"),
            Some(&"simple_list".to_string())
        );

        let fixed_trace = &config.columns[2];
        assert_eq!(fixed_trace.name, "fixed_trace");
        assert!(fixed_trace.can_be_x_axis);
        assert!(fixed_trace.can_be_y_axis);
        assert_eq!(
            fixed_trace.settings.get("trace_variant"),
            Some(&"fixed_step".to_string())
        );

        let variable_trace = &config.columns[3];
        assert_eq!(variable_trace.name, "variable_trace");
        assert!(variable_trace.can_be_x_axis);
        assert!(variable_trace.can_be_y_axis);
        assert_eq!(
            variable_trace.settings.get("trace_variant"),
            Some(&"variable_step".to_string())
        );

        // Check regular column
        let regular_col = &config.columns[4];
        assert_eq!(regular_col.name, "regular_int");
        assert!(regular_col.can_be_x_axis);
        assert!(regular_col.can_be_y_axis);
        assert!(regular_col.suggested_plot_types.contains(&PlotType::Line));
    }
}

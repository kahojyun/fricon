//! Visualization compatibility analysis
//!
//! This module analyzes dataset schemas to determine which columns
//! can be used for different types of visualization operations.

use serde::{Deserialize, Serialize};

use super::{ColumnDataType, ColumnInfo};

/// Visualization compatibility information for the entire dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationCompatibility {
    pub plottable_columns: Vec<String>,
    pub numeric_columns: Vec<String>,
    pub categorical_columns: Vec<String>,
    pub indexable_columns: Vec<String>,
    pub unsupported_columns: Vec<UnsupportedColumn>,
    pub chart_recommendations: ChartRecommendations,
}

/// Information about columns that cannot be visualized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedColumn {
    pub name: String,
    pub data_type: ColumnDataType,
    pub reason: String,
    pub suggestions: Vec<String>,
}

/// Chart type recommendations based on available data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRecommendations {
    pub recommended_chart_types: Vec<ChartType>,
    pub can_create_scatter_plot: bool,
    pub can_create_line_chart: bool,
    pub can_create_bar_chart: bool,
    pub can_create_histogram: bool,
    pub parameter_sweep_capable: bool,
}

/// Supported chart types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Scatter,
    Bar,
    Histogram,
    Heatmap,
    TimeSeries,
    Complex,
}

impl VisualizationCompatibility {
    /// Analyze columns for visualization compatibility
    #[must_use]
    pub fn analyze(columns: &[ColumnInfo]) -> Self {
        let mut plottable_columns = Vec::new();
        let mut numeric_columns = Vec::new();
        let mut categorical_columns = Vec::new();
        let mut indexable_columns = Vec::new();
        let mut unsupported_columns = Vec::new();

        for column in columns {
            match column.data_type {
                ColumnDataType::Numeric => {
                    numeric_columns.push(column.name.clone());
                    plottable_columns.push(column.name.clone());
                    if column.is_index_column {
                        indexable_columns.push(column.name.clone());
                    }
                }
                ColumnDataType::Text => {
                    categorical_columns.push(column.name.clone());
                    if column.is_index_column {
                        indexable_columns.push(column.name.clone());
                    }
                    // Text can be used for grouping but not as continuous values
                    if Self::has_reasonable_cardinality(column) {
                        plottable_columns.push(column.name.clone());
                    } else {
                        unsupported_columns.push(UnsupportedColumn {
                            name: column.name.clone(),
                            data_type: column.data_type.clone(),
                            reason: "Too many unique values for categorical visualization"
                                .to_string(),
                            suggestions: vec![
                                "Consider grouping similar values".to_string(),
                                "Use text filtering instead of visualization".to_string(),
                            ],
                        });
                    }
                }
                ColumnDataType::Boolean => {
                    categorical_columns.push(column.name.clone());
                    plottable_columns.push(column.name.clone());
                    if column.is_index_column {
                        indexable_columns.push(column.name.clone());
                    }
                }
                ColumnDataType::Complex => {
                    // Complex numbers can be visualized as magnitude, phase, etc.
                    numeric_columns.push(column.name.clone());
                    plottable_columns.push(column.name.clone());
                }
                ColumnDataType::Trace => {
                    // Traces can be expanded for time series visualization
                    plottable_columns.push(column.name.clone());
                }
                ColumnDataType::List => {
                    unsupported_columns.push(UnsupportedColumn {
                        name: column.name.clone(),
                        data_type: column.data_type.clone(),
                        reason: "List types require expansion for visualization".to_string(),
                        suggestions: vec![
                            "Extract individual elements from list".to_string(),
                            "Use list length as a numeric feature".to_string(),
                            "Aggregate list values (sum, mean, etc.)".to_string(),
                        ],
                    });
                }
                ColumnDataType::Other => {
                    unsupported_columns.push(UnsupportedColumn {
                        name: column.name.clone(),
                        data_type: column.data_type.clone(),
                        reason: "Unsupported data type for visualization".to_string(),
                        suggestions: vec![
                            "Convert to a supported data type".to_string(),
                            "Extract features from the complex type".to_string(),
                        ],
                    });
                }
            }
        }

        let chart_recommendations = Self::generate_chart_recommendations(
            &numeric_columns,
            &categorical_columns,
            &indexable_columns,
            columns,
        );

        Self {
            plottable_columns,
            numeric_columns,
            categorical_columns,
            indexable_columns,
            unsupported_columns,
            chart_recommendations,
        }
    }

    /// Check if a column has reasonable cardinality for categorical visualization
    fn has_reasonable_cardinality(column: &ColumnInfo) -> bool {
        const MAX_CATEGORICAL_VALUES: usize = 50;

        if let Some(ref sample_values) = column.sample_values {
            sample_values.len() <= MAX_CATEGORICAL_VALUES
        } else {
            // If we don't have sample values, assume it's reasonable
            true
        }
    }

    /// Generate chart type recommendations
    fn generate_chart_recommendations(
        numeric_columns: &[String],
        categorical_columns: &[String],
        indexable_columns: &[String],
        all_columns: &[ColumnInfo],
    ) -> ChartRecommendations {
        let mut recommended_chart_types = Vec::new();

        let has_numeric = !numeric_columns.is_empty();
        let has_categorical = !categorical_columns.is_empty();
        let has_index = !indexable_columns.is_empty();
        let has_multiple_numeric = numeric_columns.len() >= 2;

        // Time series charts
        let has_traces = all_columns
            .iter()
            .any(|c| c.data_type == ColumnDataType::Trace);
        if has_traces {
            recommended_chart_types.push(ChartType::TimeSeries);
        }

        // Complex number visualization
        let has_complex = all_columns
            .iter()
            .any(|c| c.data_type == ColumnDataType::Complex);
        if has_complex {
            recommended_chart_types.push(ChartType::Complex);
        }

        // Line charts - good for indexed numeric data
        let can_create_line_chart = has_numeric && has_index;
        if can_create_line_chart {
            recommended_chart_types.push(ChartType::Line);
        }

        // Scatter plots - good for multiple numeric columns
        let can_create_scatter_plot = has_multiple_numeric;
        if can_create_scatter_plot {
            recommended_chart_types.push(ChartType::Scatter);
        }

        // Bar charts - good for categorical data
        let can_create_bar_chart = has_categorical && has_numeric;
        if can_create_bar_chart {
            recommended_chart_types.push(ChartType::Bar);
        }

        // Histograms - good for single numeric column
        let can_create_histogram = has_numeric;
        if can_create_histogram {
            recommended_chart_types.push(ChartType::Histogram);
        }

        // Heatmaps - good for multiple indices and numeric values
        if indexable_columns.len() >= 2 && has_numeric {
            recommended_chart_types.push(ChartType::Heatmap);
        }

        // Parameter sweep capability - multiple index columns + numeric data
        let parameter_sweep_capable = indexable_columns.len() >= 2 && has_numeric;

        ChartRecommendations {
            recommended_chart_types,
            can_create_scatter_plot,
            can_create_line_chart,
            can_create_bar_chart,
            can_create_histogram,
            parameter_sweep_capable,
        }
    }

    /// Get columns suitable for X-axis
    #[must_use]
    pub fn get_x_axis_candidates(&self) -> Vec<String> {
        let mut candidates = self.indexable_columns.clone();
        candidates.extend(self.numeric_columns.iter().cloned());
        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Get columns suitable for Y-axis
    #[must_use]
    pub fn get_y_axis_candidates(&self) -> Vec<String> {
        self.numeric_columns.clone()
    }

    /// Get columns suitable for grouping/filtering
    #[must_use]
    pub fn get_grouping_candidates(&self) -> Vec<String> {
        let mut candidates = self.categorical_columns.clone();
        candidates.extend(self.indexable_columns.iter().cloned());
        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Check if dataset supports parameter sweep visualization
    #[must_use]
    pub fn supports_parameter_sweep(&self) -> bool {
        self.chart_recommendations.parameter_sweep_capable
    }

    /// Get recommended chart types for given X and Y columns
    #[must_use]
    pub fn get_chart_types_for_columns(
        &self,
        x_column: &str,
        y_columns: &[String],
    ) -> Vec<ChartType> {
        let mut chart_types = Vec::new();

        if y_columns.is_empty() {
            return chart_types;
        }

        let x_is_index = self.indexable_columns.contains(&x_column.to_string());
        let x_is_numeric = self.numeric_columns.contains(&x_column.to_string());
        let all_y_numeric = y_columns.iter().all(|y| self.numeric_columns.contains(y));

        if !all_y_numeric {
            return chart_types;
        }

        // Line charts for indexed X-axis
        if x_is_index {
            chart_types.push(ChartType::Line);
        }

        // Scatter plots for numeric X-axis
        if x_is_numeric {
            chart_types.push(ChartType::Scatter);
        }

        // Bar charts if X-axis is categorical
        if self.categorical_columns.contains(&x_column.to_string()) {
            chart_types.push(ChartType::Bar);
        }

        chart_types
    }

    /// Get visualization summary as human-readable text
    #[must_use]
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&"Dataset Visualization Summary:\n".to_string());
        summary.push_str(&format!(
            "- Plottable columns: {}\n",
            self.plottable_columns.len()
        ));
        summary.push_str(&format!(
            "- Numeric columns: {}\n",
            self.numeric_columns.len()
        ));
        summary.push_str(&format!(
            "- Categorical columns: {}\n",
            self.categorical_columns.len()
        ));
        summary.push_str(&format!(
            "- Index columns: {}\n",
            self.indexable_columns.len()
        ));
        summary.push_str(&format!(
            "- Unsupported columns: {}\n",
            self.unsupported_columns.len()
        ));

        if !self
            .chart_recommendations
            .recommended_chart_types
            .is_empty()
        {
            summary.push_str("\nRecommended chart types:\n");
            for chart_type in &self.chart_recommendations.recommended_chart_types {
                summary.push_str(&format!("- {chart_type:?}\n"));
            }
        }

        if self.chart_recommendations.parameter_sweep_capable {
            summary.push_str("\n✓ Parameter sweep visualization supported\n");
        }

        if !self.unsupported_columns.is_empty() {
            summary.push_str("\nUnsupported columns:\n");
            for unsupported in &self.unsupported_columns {
                summary.push_str(&format!("- {}: {}\n", unsupported.name, unsupported.reason));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;

    fn create_test_column(name: &str, data_type: ColumnDataType, is_index: bool) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type,
            arrow_type: DataType::Float64, // placeholder
            is_index_column: is_index,
            nullable: false,
            unique_value_count: None,
            sample_values: None,
        }
    }

    #[test]
    fn test_visualization_compatibility_analysis() {
        let columns = vec![
            create_test_column("id", ColumnDataType::Numeric, true),
            create_test_column("value", ColumnDataType::Numeric, false),
            create_test_column("category", ColumnDataType::Text, true),
            create_test_column("flag", ColumnDataType::Boolean, false),
            create_test_column("data", ColumnDataType::List, false),
        ];

        let compatibility = VisualizationCompatibility::analyze(&columns);

        assert_eq!(compatibility.numeric_columns.len(), 2); // id, value
        assert_eq!(compatibility.categorical_columns.len(), 2); // category, flag
        assert_eq!(compatibility.indexable_columns.len(), 2); // id, category
        assert_eq!(compatibility.unsupported_columns.len(), 1); // data (list)

        assert!(compatibility.chart_recommendations.can_create_line_chart);
        assert!(compatibility.chart_recommendations.can_create_scatter_plot);
        assert!(compatibility.chart_recommendations.can_create_bar_chart);
        assert!(compatibility.chart_recommendations.parameter_sweep_capable);
    }

    #[test]
    fn test_chart_type_recommendations() {
        let columns = vec![
            create_test_column("x", ColumnDataType::Numeric, true),
            create_test_column("y", ColumnDataType::Numeric, false),
        ];

        let compatibility = VisualizationCompatibility::analyze(&columns);
        let chart_types = compatibility.get_chart_types_for_columns("x", &vec!["y".to_string()]);

        assert!(chart_types.contains(&ChartType::Line));
        assert!(chart_types.contains(&ChartType::Scatter));
    }

    #[test]
    fn test_parameter_sweep_capability() {
        let columns = vec![
            create_test_column("param1", ColumnDataType::Text, true),
            create_test_column("param2", ColumnDataType::Numeric, true),
            create_test_column("result", ColumnDataType::Numeric, false),
        ];

        let compatibility = VisualizationCompatibility::analyze(&columns);
        assert!(compatibility.supports_parameter_sweep());
    }
}

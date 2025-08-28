//! Chart data processing for GUI visualization
//!
//! This module provides functionality to read Arrow schema information and
//! extract chart data for parameter sweep visualization in the GUI.
//!
//! Note: This module now uses the centralized `schema_utils` for schema operations.

use std::fs::File;

use anyhow::{Context, Result};
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

use crate::app::AppHandle;
use crate::dataset_manager::DatasetId;
use crate::schema_utils::{ColumnValue, extract_column_value_at, inspect_dataset_schema};

// Re-export commonly used types for backward compatibility
pub use crate::schema_utils::{ColumnDataType, ColumnInfo as SchemaColumnInfo};

/// Column information for chart configuration (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnDataType,
    pub is_index_column: bool,
    pub unique_values: Option<Vec<ColumnValue>>,
}

impl From<SchemaColumnInfo> for ColumnInfo {
    fn from(info: SchemaColumnInfo) -> Self {
        Self {
            name: info.name,
            data_type: info.data_type,
            is_index_column: info.is_index_column,
            unique_values: info.sample_values, // Use sample_values as unique_values
        }
    }
}

/// Schema response for chart configuration UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSchemaResponse {
    pub columns: Vec<ColumnInfo>,
    pub index_columns: Vec<String>,
}

/// Filter for index column values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexColumnFilter {
    pub column: String,
    pub value: ColumnValue,
}

/// Chart data request from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataRequest {
    pub dataset_id: i32,
    pub x_column: String,
    pub y_columns: Vec<String>,
    pub index_column_filters: Vec<IndexColumnFilter>,
}

/// `ECharts` optimized dataset format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataset {
    pub dimensions: Vec<String>,
    pub source: Vec<Vec<ColumnValue>>,
}

/// `ECharts` series configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsSeries {
    pub name: String,
    #[serde(rename = "type")]
    pub chart_type: String,
    pub data_group_id: usize,
}

/// Response with `ECharts` formatted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataResponse {
    pub dataset: EChartsDataset,
    pub series: Vec<EChartsSeries>,
}

/// Reads schema information from Arrow files for chart configuration
pub struct ChartSchemaReader {
    app: AppHandle,
}

impl ChartSchemaReader {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Read chart schema information for a dataset
    pub async fn read_chart_schema(&self, dataset_id: i32) -> Result<ChartSchemaResponse> {
        // Get dataset metadata for index columns
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await?;
        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(dataset.metadata.uuid);

        // Read Arrow schema from IPC file using schema_utils
        let arrow_file = dataset_path.join("dataset.arrow");
        let schema_info = inspect_dataset_schema(&arrow_file, &dataset.metadata.index_columns)?;

        // Convert to chart-specific format for backward compatibility
        let columns: Vec<ColumnInfo> = schema_info
            .columns
            .into_iter()
            .map(std::convert::Into::into)
            .collect();

        Ok(ChartSchemaResponse {
            columns,
            index_columns: dataset.metadata.index_columns,
        })
    }
}

/// Reads chart data from Arrow files with filtering capabilities
pub struct ChartDataReader {
    app: AppHandle,
}

impl ChartDataReader {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Read chart data for visualization
    pub async fn read_chart_data(&self, request: ChartDataRequest) -> Result<EChartsDataResponse> {
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(request.dataset_id))
            .await?;
        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(dataset.metadata.uuid);

        let arrow_file = dataset_path.join("dataset.arrow");
        let file = File::open(&arrow_file)
            .with_context(|| format!("Failed to open Arrow file: {}", arrow_file.display()))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut chart_data = Vec::new();

        for batch_result in reader {
            let batch = batch_result?;
            let filtered_indices =
                Self::apply_index_filters(&batch, &request.index_column_filters)?;

            for &row_idx in &filtered_indices {
                let mut row_values = Vec::new();

                // X-axis value
                if let Some(x_value) = Self::extract_value_at(&batch, &request.x_column, row_idx)? {
                    row_values.push(x_value);

                    // Y-axis values (numeric only)
                    let mut all_y_valid = true;
                    let mut y_values = Vec::new();

                    for y_column in &request.y_columns {
                        if let Some(y_value) =
                            Self::extract_numeric_value_at(&batch, y_column, row_idx)?
                        {
                            y_values.push(ColumnValue::Number(y_value));
                        } else {
                            all_y_valid = false;
                            break;
                        }
                    }

                    if all_y_valid {
                        row_values.extend(y_values);
                        chart_data.push(row_values);
                    }
                }
            }
        }

        Ok(Self::format_for_echarts(request, chart_data))
    }

    /// Process chart data from a single `RecordBatch` (used by `ChartService`)
    pub fn process_batch_data(
        request: ChartDataRequest,
        batch: &RecordBatch,
    ) -> Result<EChartsDataResponse> {
        let filtered_indices = Self::apply_index_filters(batch, &request.index_column_filters)?;
        let mut chart_data = Vec::new();

        for &row_idx in &filtered_indices {
            let mut row_values = Vec::new();

            // X-axis value
            if let Some(x_value) = Self::extract_value_at(batch, &request.x_column, row_idx)? {
                row_values.push(x_value);

                // Y-axis values (numeric only)
                let mut all_y_valid = true;
                let mut y_values = Vec::new();

                for y_column in &request.y_columns {
                    if let Some(y_value) = Self::extract_numeric_value_at(batch, y_column, row_idx)?
                    {
                        y_values.push(ColumnValue::Number(y_value));
                    } else {
                        all_y_valid = false;
                        break;
                    }
                }

                if all_y_valid {
                    row_values.extend(y_values);
                    chart_data.push(row_values);
                }
            }
        }

        Ok(Self::format_for_echarts(request, chart_data))
    }

    fn apply_index_filters(
        batch: &RecordBatch,
        filters: &[IndexColumnFilter],
    ) -> Result<Vec<usize>> {
        if filters.is_empty() {
            return Ok((0..batch.num_rows()).collect());
        }

        let mut matching_indices = Vec::new();

        'row_loop: for row_idx in 0..batch.num_rows() {
            for filter in filters {
                let column_index = batch
                    .schema()
                    .column_with_name(&filter.column)
                    .map(|(idx, _)| idx)
                    .with_context(|| format!("Filter column '{}' not found", filter.column))?;

                let column = batch.column(column_index);

                if let Some(row_value) = extract_column_value_at(column, row_idx)? {
                    if row_value != filter.value {
                        continue 'row_loop;
                    }
                } else {
                    // Null values don't match any filter
                    continue 'row_loop;
                }
            }
            matching_indices.push(row_idx);
        }

        Ok(matching_indices)
    }

    fn extract_value_at(
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<ColumnValue>> {
        let column_index = batch
            .schema()
            .column_with_name(column_name)
            .map(|(idx, _)| idx)
            .with_context(|| format!("Column '{column_name}' not found"))?;

        let column = batch.column(column_index);
        extract_column_value_at(column, row_idx)
    }

    fn extract_numeric_value_at(
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<f64>> {
        if let Some(value) = Self::extract_value_at(batch, column_name, row_idx)? {
            match value {
                ColumnValue::Number(n) => Ok(Some(n)),
                _ => Ok(None), // Non-numeric values are filtered out
            }
        } else {
            Ok(None)
        }
    }

    fn format_for_echarts(
        request: ChartDataRequest,
        rows: Vec<Vec<ColumnValue>>,
    ) -> EChartsDataResponse {
        let mut dimensions = vec![request.x_column];
        dimensions.extend(request.y_columns.iter().cloned());

        let dataset = EChartsDataset {
            dimensions: dimensions.clone(),
            source: rows,
        };

        let series = request
            .y_columns
            .into_iter()
            .enumerate()
            .map(|(idx, name)| EChartsSeries {
                name,
                chart_type: "line".to_string(),
                data_group_id: idx + 1, // +1 because 0 is X-axis
            })
            .collect();

        EChartsDataResponse { dataset, series }
    }
}

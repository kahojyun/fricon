//! Chart data processing for GUI visualization
//!
//! This module provides functionality to read Arrow schema information and
//! extract chart data for parameter sweep visualization in the GUI.

use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use arrow::array::*;
use arrow::datatypes::DataType;
use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

use crate::app::AppHandle;
use crate::dataset_manager::{DatasetId, DatasetRecord};

/// Represents different types of column values for charting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ColumnValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Simplified data types for chart UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnDataType {
    Numeric,
    Text,
    Boolean,
    Other,
}

/// Column information for chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnDataType,
    pub is_index_column: bool,
    pub unique_values: Option<Vec<ColumnValue>>,
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

/// ECharts optimized dataset format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataset {
    pub dimensions: Vec<String>,
    pub source: Vec<Vec<ColumnValue>>,
}

/// ECharts series configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsSeries {
    pub name: String,
    #[serde(rename = "type")]
    pub chart_type: String,
    pub data_group_id: usize,
}

/// Response with ECharts formatted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataResponse {
    pub dataset: EChartsDataset,
    pub series: Vec<EChartsSeries>,
}

/// Helper function to extract column values at a specific row index
fn extract_column_value_at(column: &dyn Array, row_idx: usize) -> Result<Option<ColumnValue>> {
    if column.is_null(row_idx) {
        return Ok(None);
    }

    let value = match column.data_type() {
        DataType::Int8 => {
            let array = column.as_any().downcast_ref::<Int8Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::Int16 => {
            let array = column.as_any().downcast_ref::<Int16Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::Int32 => {
            let array = column.as_any().downcast_ref::<Int32Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::Int64 => {
            let array = column.as_any().downcast_ref::<Int64Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::UInt8 => {
            let array = column.as_any().downcast_ref::<UInt8Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::UInt16 => {
            let array = column.as_any().downcast_ref::<UInt16Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::UInt32 => {
            let array = column.as_any().downcast_ref::<UInt32Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::UInt64 => {
            let array = column.as_any().downcast_ref::<UInt64Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
        }
        DataType::Float32 => {
            let array = column.as_any().downcast_ref::<Float32Array>().unwrap();
            ColumnValue::Number(array.value(row_idx) as f64)
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
        _ => return Ok(None), // Unsupported type
    };

    Ok(Some(value))
}

/// Reads schema information from Arrow files for chart configuration
pub struct ChartSchemaReader {
    app: AppHandle,
}

impl ChartSchemaReader {
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

        // Read Arrow schema from IPC file
        let arrow_file = dataset_path.join("dataset.arrow");
        let schema_info = self.read_arrow_schema(&arrow_file, &dataset).await?;

        Ok(schema_info)
    }

    async fn read_arrow_schema(
        &self,
        arrow_file: &Path,
        dataset: &DatasetRecord,
    ) -> Result<ChartSchemaResponse> {
        let file = File::open(arrow_file)
            .with_context(|| format!("Failed to open Arrow file: {:?}", arrow_file))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;
        let schema = reader.schema();

        let mut columns = Vec::new();

        for field in schema.fields() {
            let data_type = self.classify_data_type(field.data_type());
            let is_index_column = dataset.metadata.index_columns.contains(field.name());

            // For index columns, read unique values in a separate file read
            let unique_values = if is_index_column {
                Some(
                    self.extract_unique_values_for_column(arrow_file, field.name())
                        .await?,
                )
            } else {
                None
            };

            columns.push(ColumnInfo {
                name: field.name().clone(),
                data_type,
                is_index_column,
                unique_values,
            });
        }

        Ok(ChartSchemaResponse {
            columns,
            index_columns: dataset.metadata.index_columns.clone(),
        })
    }

    fn classify_data_type(&self, arrow_type: &DataType) -> ColumnDataType {
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
            _ => ColumnDataType::Other,
        }
    }

    async fn extract_unique_values_for_column(
        &self,
        arrow_file: &Path,
        column_name: &str,
    ) -> Result<Vec<ColumnValue>> {
        let file = File::open(arrow_file)
            .with_context(|| format!("Failed to open Arrow file: {:?}", arrow_file))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        self.extract_unique_values(reader, column_name).await
    }

    async fn extract_unique_values(
        &self,
        reader: FileReader<File>,
        column_name: &str,
    ) -> Result<Vec<ColumnValue>> {
        let mut unique_values = HashSet::new();
        const MAX_UNIQUE_VALUES: usize = 100; // Limit to avoid excessive memory usage

        for batch_result in reader {
            let batch = batch_result?;
            let column_index = batch
                .schema()
                .column_with_name(column_name)
                .map(|(idx, _)| idx)
                .with_context(|| format!("Column '{}' not found in batch", column_name))?;

            let column = batch.column(column_index);

            for row_idx in 0..column.len() {
                if unique_values.len() >= MAX_UNIQUE_VALUES {
                    break;
                }

                if let Some(value) = extract_column_value_at(column, row_idx)? {
                    unique_values.insert(format!("{:?}", value)); // Use debug format as key
                    if unique_values.len() >= MAX_UNIQUE_VALUES {
                        break;
                    }
                }
            }

            if unique_values.len() >= MAX_UNIQUE_VALUES {
                break;
            }
        }

        // Convert back to ColumnValue (this is a simplification)
        // In a real implementation, we'd need more sophisticated deduplication
        let result: Vec<ColumnValue> = unique_values
            .into_iter()
            .take(MAX_UNIQUE_VALUES)
            .filter_map(|s| {
                // This is a placeholder - we'd need proper parsing based on the column type
                if let Ok(num) = s.parse::<f64>() {
                    Some(ColumnValue::Number(num))
                } else if s == "true" || s == "false" {
                    Some(ColumnValue::Boolean(s.parse().unwrap()))
                } else {
                    Some(ColumnValue::String(s))
                }
            })
            .collect();

        Ok(result)
    }
}

/// Reads chart data from Arrow files with filtering capabilities
pub struct ChartDataReader {
    app: AppHandle,
}

impl ChartDataReader {
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
            .with_context(|| format!("Failed to open Arrow file: {:?}", arrow_file))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut chart_data = Vec::new();

        for batch_result in reader {
            let batch = batch_result?;
            let filtered_indices =
                self.apply_index_filters(&batch, &request.index_column_filters)?;

            for &row_idx in &filtered_indices {
                let mut row_values = Vec::new();

                // X-axis value
                if let Some(x_value) = self.extract_value_at(&batch, &request.x_column, row_idx)? {
                    row_values.push(x_value);

                    // Y-axis values (numeric only)
                    let mut all_y_valid = true;
                    let mut y_values = Vec::new();

                    for y_column in &request.y_columns {
                        if let Some(y_value) =
                            self.extract_numeric_value_at(&batch, y_column, row_idx)?
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

        self.format_for_echarts(request, chart_data)
    }

    fn apply_index_filters(
        &self,
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
        &self,
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<ColumnValue>> {
        let column_index = batch
            .schema()
            .column_with_name(column_name)
            .map(|(idx, _)| idx)
            .with_context(|| format!("Column '{}' not found", column_name))?;

        let column = batch.column(column_index);
        extract_column_value_at(column, row_idx)
    }

    fn extract_numeric_value_at(
        &self,
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<f64>> {
        if let Some(value) = self.extract_value_at(batch, column_name, row_idx)? {
            match value {
                ColumnValue::Number(n) => Ok(Some(n)),
                _ => Ok(None), // Non-numeric values are filtered out
            }
        } else {
            Ok(None)
        }
    }

    fn format_for_echarts(
        &self,
        request: ChartDataRequest,
        rows: Vec<Vec<ColumnValue>>,
    ) -> Result<EChartsDataResponse> {
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

        Ok(EChartsDataResponse { dataset, series })
    }
}

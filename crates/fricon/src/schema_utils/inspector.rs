//! Schema inspection and analysis functionality
//!
//! This module provides tools to read Arrow schema information from IPC files,
//! extract dataset shape information, and sample column values.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use arrow::ipc::reader::FileReader;
use serde::{Deserialize, Serialize};

use super::{
    ColumnValue, DatasetSchemaInfo, SchemaSummary, compatibility::VisualizationCompatibility,
    custom_types, extract_column_value_at, get_column_info, get_schema_summary,
};

/// Dataset shape and size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetShape {
    pub row_count: Option<usize>,
    pub column_count: usize,
    pub memory_size_bytes: Option<usize>,
    pub batch_count: Option<usize>,
}

/// Schema inspection engine
pub struct SchemaInspector;

impl SchemaInspector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Inspect complete schema information from an Arrow IPC file
    pub fn inspect_schema(
        &self,
        path: &Path,
        index_columns: &[String],
    ) -> Result<DatasetSchemaInfo> {
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let schema = reader.schema();
        let shape = self.extract_shape_info(path)?;

        let mut columns = Vec::new();
        let mut custom_types = HashMap::new();

        // Process each field in the schema
        for field in schema.fields() {
            let is_index_column = index_columns.contains(field.name());

            // Extract sample values for index columns
            let sample_values = if is_index_column {
                Some(self.extract_unique_values(path, field.name(), 100)?)
            } else {
                None
            };

            let column_info = get_column_info(field, is_index_column, sample_values);

            // Collect custom type information
            if let Some(custom_info) = custom_types::get_custom_type_info(field.data_type()) {
                custom_types.insert(field.name().clone(), custom_info);
            }

            columns.push(column_info);
        }

        // Analyze visualization compatibility
        let compatibility = VisualizationCompatibility::analyze(&columns);

        Ok(DatasetSchemaInfo {
            columns,
            shape,
            index_columns: index_columns.to_vec(),
            custom_types,
            visualization_compatibility: compatibility,
        })
    }

    /// Extract dataset shape information
    pub fn extract_shape_info(&self, path: &Path) -> Result<DatasetShape> {
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let schema = reader.schema();
        let column_count = schema.fields().len();

        let mut total_rows = 0;
        let mut batch_count = 0;
        let mut memory_size_bytes = 0;

        // Iterate through batches to count rows and estimate memory usage
        for batch_result in reader {
            let batch = batch_result?;
            total_rows += batch.num_rows();
            batch_count += 1;

            // Estimate memory usage (this is approximate)
            memory_size_bytes += batch
                .columns()
                .iter()
                .map(|col| col.get_array_memory_size())
                .sum::<usize>();
        }

        Ok(DatasetShape {
            row_count: Some(total_rows),
            column_count,
            memory_size_bytes: Some(memory_size_bytes),
            batch_count: Some(batch_count),
        })
    }

    /// Extract unique values from a specific column with limit
    pub fn extract_unique_values(
        &self,
        path: &Path,
        column_name: &str,
        limit: usize,
    ) -> Result<Vec<ColumnValue>> {
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        self.extract_unique_values_from_reader(reader, column_name, limit, path)
    }

    /// Extract unique values from a `FileReader`
    fn extract_unique_values_from_reader(
        &self,
        reader: FileReader<File>,
        column_name: &str,
        limit: usize,
        arrow_file: &Path,
    ) -> Result<Vec<ColumnValue>> {
        let mut unique_values = HashSet::new();

        for batch_result in reader {
            let batch = batch_result?;
            let column_index = batch
                .schema()
                .column_with_name(column_name)
                .map(|(idx, _)| idx)
                .with_context(|| format!("Column '{column_name}' not found in batch"))?;

            let column = batch.column(column_index);

            for row_idx in 0..column.len() {
                if unique_values.len() >= limit {
                    break;
                }

                if let Some(value) = extract_column_value_at(column, row_idx)? {
                    // Use a string representation as key for deduplication
                    let key = match &value {
                        ColumnValue::Number(n) => format!("n:{n}"),
                        ColumnValue::String(s) => format!("s:{s}"),
                        ColumnValue::Boolean(b) => format!("b:{b}"),
                    };

                    if unique_values.insert(key) {
                        // Successfully added unique value
                    }
                }
            }

            if unique_values.len() >= limit {
                break;
            }
        }

        // Convert back to ColumnValue (re-read to maintain order and types)
        self.collect_unique_values(arrow_file, column_name, unique_values, limit)
    }

    /// Re-read the file to collect actual unique values maintaining proper types
    fn collect_unique_values(
        &self,
        arrow_file: &Path,
        column_name: &str,
        seen_keys: HashSet<String>,
        limit: usize,
    ) -> Result<Vec<ColumnValue>> {
        let file = File::open(arrow_file)
            .with_context(|| format!("Failed to open Arrow file: {arrow_file:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut result = Vec::new();
        let mut collected_keys = HashSet::new();

        for batch_result in reader {
            let batch = batch_result?;
            let column_index = batch
                .schema()
                .column_with_name(column_name)
                .map(|(idx, _)| idx)
                .with_context(|| format!("Column '{column_name}' not found in batch"))?;

            let column = batch.column(column_index);

            for row_idx in 0..column.len() {
                if result.len() >= limit {
                    break;
                }

                if let Some(value) = extract_column_value_at(column, row_idx)? {
                    let key = match &value {
                        ColumnValue::Number(n) => format!("n:{n}"),
                        ColumnValue::String(s) => format!("s:{s}"),
                        ColumnValue::Boolean(b) => format!("b:{b}"),
                    };

                    if seen_keys.contains(&key) && collected_keys.insert(key) {
                        result.push(value);
                    }
                }
            }

            if result.len() >= limit {
                break;
            }
        }

        Ok(result)
    }

    /// Sample values from a column (not necessarily unique)
    pub fn sample_column_values(
        &self,
        path: &Path,
        column_name: &str,
        sample_size: usize,
    ) -> Result<Vec<ColumnValue>> {
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut samples = Vec::new();
        let mut total_rows = 0;

        // First pass: count total rows
        for batch_result in reader {
            let batch = batch_result?;
            total_rows += batch.num_rows();
        }

        if total_rows == 0 {
            return Ok(samples);
        }

        // Calculate sampling interval
        let interval = if total_rows <= sample_size {
            1
        } else {
            total_rows / sample_size
        };

        // Second pass: collect samples
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut current_row = 0;
        let mut next_sample_row = 0;

        for batch_result in reader {
            let batch = batch_result?;
            let column_index = batch
                .schema()
                .column_with_name(column_name)
                .map(|(idx, _)| idx)
                .with_context(|| format!("Column '{column_name}' not found in batch"))?;

            let column = batch.column(column_index);

            for row_idx in 0..column.len() {
                if current_row == next_sample_row
                    && samples.len() < sample_size
                    && let Some(value) = extract_column_value_at(column, row_idx)?
                {
                    samples.push(value);
                    next_sample_row += interval;
                }
                current_row += 1;
            }

            if samples.len() >= sample_size {
                break;
            }
        }

        Ok(samples)
    }

    /// Get schema summary statistics
    pub fn get_schema_summary(&self, path: &Path) -> Result<SchemaSummary> {
        let file =
            File::open(path).with_context(|| format!("Failed to open Arrow file: {path:?}"))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let schema = reader.schema();
        Ok(get_schema_summary(&schema))
    }

    /// Check if file exists and is a valid Arrow IPC file
    pub fn validate_arrow_file(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let file = File::open(path).with_context(|| format!("Failed to open file: {path:?}"))?;

        match FileReader::try_new(file, None) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for SchemaInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::ipc::writer::FileWriter;
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_test_arrow_file() -> Result<NamedTempFile> {
        let temp_file = NamedTempFile::new()?;

        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false),
        ]);

        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5])),
                Arc::new(StringArray::from(vec!["a", "b", "c", "a", "b"])),
                Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0])),
            ],
        )?;

        let file = File::create(temp_file.path())?;
        let mut writer = FileWriter::try_new(file, &schema)?;
        writer.write(&batch)?;
        writer.finish()?;

        Ok(temp_file)
    }

    #[test]
    fn test_extract_shape_info() -> Result<()> {
        let temp_file = create_test_arrow_file()?;
        let inspector = SchemaInspector::new();

        let shape = inspector.extract_shape_info(temp_file.path())?;

        assert_eq!(shape.row_count, Some(5));
        assert_eq!(shape.column_count, 3);
        assert_eq!(shape.batch_count, Some(1));
        assert!(shape.memory_size_bytes.unwrap() > 0);

        Ok(())
    }

    #[test]
    fn test_extract_unique_values() -> Result<()> {
        let temp_file = create_test_arrow_file()?;
        let inspector = SchemaInspector::new();

        let unique_values = inspector.extract_unique_values(temp_file.path(), "name", 10)?;

        // Should contain "a", "b", "c"
        assert_eq!(unique_values.len(), 3);

        Ok(())
    }

    #[test]
    fn test_validate_arrow_file() -> Result<()> {
        let temp_file = create_test_arrow_file()?;
        let inspector = SchemaInspector::new();

        assert!(inspector.validate_arrow_file(temp_file.path())?);

        // Test with non-existent file
        assert!(!inspector.validate_arrow_file(Path::new("/nonexistent/file.arrow"))?);

        Ok(())
    }
}

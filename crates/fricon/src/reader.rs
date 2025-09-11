use crate::dataset_manager::DatasetManagerError;
use crate::live::{LiveDataset, LiveDatasetWriter, SelectError as LiveSelectError};
use crate::utils::{chunk_path, read_ipc_file_mmap};
use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CompletedDataset {
    schema: SchemaRef,
    batches: Arc<Vec<RecordBatch>>,
}
impl CompletedDataset {
    pub fn open(dir_path: &Path) -> Result<Self, DatasetManagerError> {
        let mut batches = Vec::new();
        let mut schema = None;

        // Try to read chunked files starting from data_chunk_0.arrow
        let mut chunk_index = 0;

        loop {
            let chunk_path = chunk_path(dir_path, chunk_index);

            // If chunk file doesn't exist, break
            if !chunk_path.exists() {
                break;
            }

            // Use shared mmap reading function
            let chunk_batches = read_ipc_file_mmap(&chunk_path)
                .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;

            if !chunk_batches.is_empty() {
                if schema.is_none() {
                    schema = Some(chunk_batches[0].schema());
                }
                batches.extend(chunk_batches);
            }

            chunk_index += 1;
        }

        let schema = schema.ok_or_else(|| {
            DatasetManagerError::io_invalid_data("no chunk files found in dataset directory")
        })?;
        Ok(Self {
            schema,
            batches: Arc::new(batches),
        })
    }
    pub fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    pub fn select_by_indices(
        &self,
        indices: &[usize],
        column_indices: Option<&[usize]>,
    ) -> Result<RecordBatch, DatasetManagerError> {
        use arrow::compute::concat_batches;
        if self.batches.is_empty() {
            return Err(DatasetManagerError::io_invalid_data("empty dataset"));
        }
        let full = concat_batches(&self.schema, &self.batches[..])
            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;
        let writer = LiveDatasetWriter::new(self.schema.clone());
        let live = writer.reader();
        writer.append(full);
        live.select_by_indices(indices, column_indices)
            .map_err(map_live_select_err)
    }
    pub fn batches_slice(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// Infer a multi-index over the dataset based on column change patterns.
    #[must_use]
    pub fn infer_multi_index(&self) -> crate::multi_index::MultiIndex {
        crate::multi_index::infer_multi_index_from_batches(self.batches_slice())
    }
}
#[allow(clippy::needless_pass_by_value)]
fn map_live_select_err(err: LiveSelectError) -> DatasetManagerError {
    DatasetManagerError::io_invalid_data(format!("selection error: {err}"))
}
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum DatasetReader {
    Completed(CompletedDataset),
    Live(LiveDataset),
}
impl DatasetReader {
    #[must_use]
    pub fn schema(&self) -> SchemaRef {
        match self {
            Self::Completed(c) => c.schema.clone(),
            Self::Live(l) => l.schema(),
        }
    }

    pub fn select_by_indices(
        &self,
        indices: &[usize],
        column_indices: Option<&[usize]>,
    ) -> Result<RecordBatch, DatasetManagerError> {
        match self {
            Self::Completed(c) => c.select_by_indices(indices, column_indices),
            Self::Live(l) => l
                .select_by_indices(indices, column_indices)
                .map_err(map_live_select_err),
        }
    }

    #[must_use]
    pub fn batches(&self) -> Option<&[RecordBatch]> {
        match self {
            Self::Completed(c) => Some(c.batches_slice()),
            Self::Live(_) => None,
        }
    }

    /// Infer multi-index if possible. For Completed datasets this analyzes persisted batches.
    /// For Live datasets, infer using the first two logical rows when available.
    #[must_use]
    pub fn infer_multi_index(&self) -> crate::multi_index::MultiIndex {
        match self {
            Self::Completed(c) => c.infer_multi_index(),
            Self::Live(l) => {
                if l.total_rows() < 2 {
                    return crate::multi_index::MultiIndex {
                        level_indices: Vec::new(),
                        level_names: Vec::new(),
                        deepest_level_col: None,
                    };
                }
                match l.select_by_indices(&[0, 1], None) {
                    Ok(b) => crate::multi_index::infer_multi_index_from_batches(&[b]),
                    Err(_) => crate::multi_index::MultiIndex {
                        level_indices: Vec::new(),
                        level_names: Vec::new(),
                        deepest_level_col: None,
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::write_session::background_writer::BackgroundWriter;
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio_util::task::TaskTracker;

    fn make_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![Field::new("v", DataType::Int32, false)]))
    }

    fn make_batch(schema: &SchemaRef, start: i32, n: i32) -> RecordBatch {
        let arr = Int32Array::from_iter_values(start..start + n);
        RecordBatch::try_new(schema.clone(), vec![Arc::new(arr)]).unwrap()
    }

    #[tokio::test]
    async fn test_completed_dataset_mmap() {
        let dir = tempdir().unwrap();
        let dataset_dir = dir.path();
        let schema = make_schema();
        let tracker = TaskTracker::new();

        // Create chunked files using BackgroundWriter
        let (chunk_completed_sender, _chunk_completed_receiver) = tokio::sync::mpsc::channel(16);
        let writer = BackgroundWriter::new(
            &tracker,
            dataset_dir,
            schema.clone(),
            chunk_completed_sender,
        );

        // Write some data
        for i in 0..10 {
            let batch = make_batch(&schema, i * 10, 10);
            writer.write(batch).await.unwrap();
        }

        // Shutdown and wait for completion
        writer.shutdown().await.unwrap();
        tracker.close();
        tracker.wait().await;

        // Now try to read using mmap
        let dataset = CompletedDataset::open(dataset_dir).unwrap();

        assert_eq!(dataset.schema(), schema);
        assert_eq!(
            dataset
                .batches_slice()
                .iter()
                .map(arrow::array::RecordBatch::num_rows)
                .sum::<usize>(),
            100
        );

        // Test selection
        let result = dataset.select_by_indices(&[0, 1, 2], None).unwrap();
        assert_eq!(result.num_rows(), 3);
    }
}

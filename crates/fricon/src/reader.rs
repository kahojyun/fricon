use crate::dataset_manager::DatasetManagerError;
use crate::live::{LiveDataset, LiveDatasetWriter, SelectError as LiveSelectError};
use arrow::{array::RecordBatch, datatypes::SchemaRef, ipc::reader::FileReader};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct CompletedDataset {
    schema: SchemaRef,
    batches: Arc<Vec<RecordBatch>>,
}
impl CompletedDataset {
    pub fn from_arrow_file(path: &Path) -> Result<Self, DatasetManagerError> {
        let file = File::open(path)?;
        let reader = FileReader::try_new(file, None)?;
        let mut batches = Vec::new();
        let mut schema = None;
        for b in reader {
            let b = b?;
            if schema.is_none() {
                schema = Some(b.schema());
            }
            batches.push(b);
        }
        let schema =
            schema.ok_or_else(|| DatasetManagerError::io_invalid_data("empty dataset file"))?;
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
        let writer = LiveDatasetWriter::new(self.schema.clone(), PathBuf::from("/dev/null"));
        let live = writer.reader();
        writer.append(full);
        live.select_by_indices(indices, column_indices)
            .map_err(map_live_select_err)
    }
    pub fn batches_slice(&self) -> &[RecordBatch] {
        &self.batches
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
}

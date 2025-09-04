use crate::dataset_manager::DatasetManagerError;
use crate::live::{LiveDataset, LiveDatasetWriter, SelectError as LiveSelectError};
use crate::utils::chunk_path;
use arrow::{
    array::RecordBatch,
    buffer::Buffer,
    datatypes::SchemaRef,
    ipc::{
        Block,
        convert::fb_to_schema,
        reader::{FileDecoder, read_footer_length},
        root_as_footer,
    },
};
use std::{fs::File, path::Path, sync::Arc};

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

            // Open the file and memory map it
            let ipc_file = File::open(&chunk_path)
                .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;
            let mmap = unsafe { memmap2::Mmap::map(&ipc_file) }
                .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;

            // Convert the mmap region to an Arrow `Buffer`
            let bytes = bytes::Bytes::from_owner(mmap);
            let buffer = Buffer::from(bytes);

            // Use the IPCBufferDecoder to read batches
            let decoder = IPCBufferDecoder::new(buffer)?;

            for i in 0..decoder.num_batches() {
                let batch = decoder.get_batch(i)?.ok_or_else(|| {
                    DatasetManagerError::io_invalid_data("failed to read batch: batch was None")
                })?;
                if schema.is_none() {
                    schema = Some(batch.schema());
                }
                batches.push(batch);
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
}
#[allow(clippy::needless_pass_by_value)]
fn map_live_select_err(err: LiveSelectError) -> DatasetManagerError {
    DatasetManagerError::io_invalid_data(format!("selection error: {err}"))
}

/// Incrementally decodes [`RecordBatch`]es from an IPC file stored in a Arrow
/// [`Buffer`] using the [`FileDecoder`] API.
///
/// This is a wrapper around the example in the `FileDecoder` which handles the
/// low level interaction with the Arrow IPC format.
struct IPCBufferDecoder {
    /// Memory (or memory mapped) Buffer with the data
    buffer: Buffer,
    /// Decoder that reads Arrays that refers to the underlying buffers
    decoder: FileDecoder,
    /// Location of the batches within the buffer
    batches: Vec<Block>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl IPCBufferDecoder {
    fn new(buffer: Buffer) -> Result<Self, DatasetManagerError> {
        let trailer_start = buffer.len() - 10;
        let footer_len = read_footer_length(buffer[trailer_start..].try_into().unwrap())
            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;
        let footer = root_as_footer(&buffer[trailer_start - footer_len..trailer_start])
            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;

        let schema = fb_to_schema(footer.schema().unwrap());

        let mut decoder = FileDecoder::new(Arc::new(schema), footer.version());

        // Read dictionaries
        for block in footer.dictionaries().iter().flatten() {
            let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
            let data = buffer.slice_with_length(block.offset() as _, block_len);
            decoder
                .read_dictionary(block, &data)
                .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))?;
        }

        // convert to Vec from the flatbuffers Vector to avoid having a direct dependency on flatbuffers
        let batches = footer
            .recordBatches()
            .map(|b| b.iter().copied().collect())
            .unwrap_or_default();

        Ok(Self {
            buffer,
            decoder,
            batches,
        })
    }

    /// Return the number of [`RecordBatch`]es in this buffer
    fn num_batches(&self) -> usize {
        self.batches.len()
    }

    /// Return the [`RecordBatch`] at message index `i`.
    ///
    /// This may return `None` if the IPC message was None
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn get_batch(&self, i: usize) -> Result<Option<RecordBatch>, DatasetManagerError> {
        let block = &self.batches[i];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let data = self
            .buffer
            .slice_with_length(block.offset() as _, block_len);
        self.decoder
            .read_record_batch(block, &data)
            .map_err(|e| DatasetManagerError::io_invalid_data(e.to_string()))
    }
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

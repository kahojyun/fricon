pub(crate) mod background_writer;

use crate::utils::read_ipc_file_mmap;
use arrow::{array::RecordBatch, datatypes::SchemaRef};
use std::path::PathBuf;
use tokio_util::task::TaskTracker;

use self::background_writer::{BackgroundWriter, Result};
use crate::live::{LiveDataset, LiveDatasetWriter};

/// High-level write session combining a `BackgroundWriter` with a `LiveDataset`.
///
/// Separation of concerns:
/// - `BackgroundWriter` handles persistence & events
/// - `WriteSession` enriches by updating the in-memory `LiveDataset`
pub struct WriteSession {
    writer: BackgroundWriter,
    live_writer: LiveDatasetWriter,
}

impl WriteSession {
    pub fn new(tracker: &TaskTracker, dir_path: impl Into<PathBuf>, schema: SchemaRef) -> Self {
        let live_writer = LiveDatasetWriter::new(schema.clone());

        // Channel for chunk completed notifications
        let (chunk_completed_sender, mut chunk_completed_receiver) = tokio::sync::mpsc::channel(16);

        let writer = BackgroundWriter::new(tracker, dir_path, schema, chunk_completed_sender);

        // Listen for chunk completion events and trigger replace_sequential_front
        let live_writer_for_task = live_writer.clone();
        tracker.spawn(async move {
            // Single chunk completion expected in current design; loop if future needs multiple.
            while let Some(path) = chunk_completed_receiver.recv().await {
                if let Err(e) = Self::handle_chunk_completion(&live_writer_for_task, &path) {
                    tracing::error!("Failed to handle chunk completion: {}", e);
                }
            }
        });

        Self {
            writer,
            live_writer,
        }
    }

    fn handle_chunk_completion(
        live_writer: &LiveDatasetWriter,
        chunk_path: &std::path::Path,
    ) -> Result<()> {
        // Read the completed chunk using memory-mapped reading and replace the sequential front in live dataset
        let batches = read_ipc_file_mmap(chunk_path)
            .map_err(|e| background_writer::Error::Io(std::io::Error::other(e)))?;

        if !batches.is_empty()
            && let Err(e) = live_writer.replace_sequential_front(&batches)
        {
            tracing::warn!("Failed to replace sequential front: {}", e);
        }

        Ok(())
    }

    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        self.live_writer.append(batch.clone());
        self.writer.write(batch).await
    }

    pub fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle {
            live: self.live_writer.reader(),
        }
    }

    /// Flush and finalize by shutting down the background writer.
    pub async fn shutdown(self) -> Result<()> {
        self.writer.shutdown().await
    }
}

/// A shareable handle to a `WriteSession`, allowing concurrent access.
#[derive(Clone)]
pub struct WriteSessionHandle {
    live: LiveDataset,
}

impl WriteSessionHandle {
    pub fn live(&self) -> &LiveDataset {
        &self.live
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::chunk_path;

    use super::*;
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::tempdir;

    fn make_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![Field::new("v", DataType::Int32, false)]))
    }

    fn make_batch(schema: &SchemaRef, start: i32, n: i32) -> RecordBatch {
        let arr = Int32Array::from_iter_values(start..start + n);
        RecordBatch::try_new(schema.clone(), vec![Arc::new(arr)]).unwrap()
    }

    #[tokio::test]
    async fn write_session_basic() {
        let dir = tempdir().unwrap();
        let dataset_dir = dir.path();
        let schema = make_schema();
        let tracker = TaskTracker::new();

        let session = WriteSession::new(&tracker, dataset_dir, schema.clone());
        let handle = session.handle();

        // Write some data
        for i in 0..5 {
            let batch = make_batch(&schema, i * 10, 10);
            session.write(batch).await.unwrap();
        }

        // Check that live dataset has the data
        let live = handle.live();
        assert_eq!(live.total_rows(), 50);

        // Explicit shutdown instead of relying on tracker close.
        session.shutdown().await.unwrap();

        // Check that chunk file exists
        let chunk_path = chunk_path(dir.path(), 0);
        assert!(chunk_path.exists(), "data_chunk_0.arrow should exist");
    }
}

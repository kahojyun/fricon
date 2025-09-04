pub(crate) mod background_writer;

use arrow::{array::RecordBatch, datatypes::SchemaRef, ipc::reader::FileReader};
use std::{fs::File, path::PathBuf};
use tokio_util::task::TaskTracker;

use self::background_writer::{BackgroundWriter, Event, Result};
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
        let writer = BackgroundWriter::new(tracker, dir_path, schema);

        // Listen for chunk completion events and trigger replace_sequential_front
        let mut event_rx = writer.subscribe();
        let live_writer_for_task = live_writer.clone();
        tracker.spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                if let Event::ChunkCompleted { path, .. } = event {
                    if let Err(e) = Self::handle_chunk_completion(&live_writer_for_task, &path) {
                        tracing::error!("Failed to handle chunk completion: {}", e);
                    }
                } else if matches!(event, Event::Closed) {
                    break;
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
        // Read the completed chunk and replace the sequential front in live dataset
        let file = File::open(chunk_path)?;
        let reader = FileReader::try_new(file, None)?;
        let mut batches = Vec::new();

        for batch_result in reader {
            let batch = batch_result?;
            batches.push(batch);
        }

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
    pub(crate) fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.writer.subscribe()
    }
    pub fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle {
            live: self.live_writer.reader(),
        }
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

        // Drop session to trigger flush
        drop(session);

        // Wait for completion
        tracker.close();
        tracker.wait().await;

        // Check that chunk file exists
        let chunk_path = chunk_path(dir.path(), 0);
        assert!(chunk_path.exists(), "data_chunk_0.arrow should exist");
    }
}

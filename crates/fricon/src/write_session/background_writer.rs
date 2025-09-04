use crate::utils::chunk_path;
use arrow::{
    array::RecordBatch, compute::BatchCoalescer, datatypes::SchemaRef, ipc::writer::FileWriter,
};
use std::{
    fs::File,
    io::{BufWriter, Seek},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio_util::task::TaskTracker;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Arrow error: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Send error: {0}")]
    Send(String),
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Event {
    #[allow(dead_code)]
    ChunkCompleted {
        chunk_index: usize,
        path: PathBuf,
    },
    Closed,
}

/// Background writer that persists incoming batches to chunked Arrow IPC files.
///
/// Responsibilities extracted from `WriteSession`:
/// - Own the mpsc sender for `RecordBatch`
/// - Own the broadcast event channel (ChunkCompleted / Closed)
/// - Spawn blocking write / coalesce task
pub struct BackgroundWriter {
    sender: mpsc::Sender<RecordBatch>,
    event_sender: broadcast::Sender<Event>,
}

impl BackgroundWriter {
    pub fn new(tracker: &TaskTracker, dir_path: impl Into<PathBuf>, schema: SchemaRef) -> Self {
        let dir_path = dir_path.into();
        let (sender, receiver) = mpsc::channel(32);
        let (event_sender, _) = broadcast::channel(16);
        let event_sender_for_task = event_sender.clone();
        tracker.spawn_blocking(move || {
            if let Err(e) =
                blocking_write_task(&dir_path, &schema, receiver, &event_sender_for_task)
            {
                error!("BackgroundWriter task failed: {e}");
            }
        });
        Self {
            sender,
            event_sender,
        }
    }

    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        self.sender
            .send(batch)
            .await
            .map_err(|e| Error::Send(e.to_string()))?;
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_sender.subscribe()
    }
}

struct ChunkWriter {
    writer: FileWriter<BufWriter<File>>,
    chunk_index: usize,
    current_chunk_size: u64,
    total_rows: usize,
    dir_path: PathBuf,
}

impl ChunkWriter {
    fn new(dir_path: &Path, chunk_index: usize, schema: &SchemaRef) -> Result<Self> {
        let chunk_path = chunk_path(dir_path, chunk_index);
        let file = File::create_new(&chunk_path)?;
        let buf_writer = BufWriter::new(file);
        let writer = FileWriter::try_new(buf_writer, schema)?;

        Ok(Self {
            writer,
            chunk_index,
            current_chunk_size: 0,
            total_rows: 0,
            dir_path: dir_path.to_path_buf(),
        })
    }

    fn write_batch(&mut self, batch: &RecordBatch) -> Result<()> {
        self.writer.write(batch)?;
        self.current_chunk_size = self.writer.get_mut().stream_position()?;
        self.total_rows += batch.num_rows();
        Ok(())
    }

    fn finish_chunk(mut self, event_sender: &broadcast::Sender<Event>) -> Result<()> {
        self.writer.finish()?;
        let chunk_path = chunk_path(&self.dir_path, self.chunk_index);
        info!(
            "Chunk {} completed: {} rows written to {:?}",
            self.chunk_index, self.total_rows, chunk_path
        );
        let _ = event_sender.send(Event::ChunkCompleted {
            chunk_index: self.chunk_index,
            path: chunk_path,
        });
        Ok(())
    }

    fn should_rotate(&self, max_chunk_size: u64) -> bool {
        self.current_chunk_size >= max_chunk_size
    }
}

fn blocking_write_task(
    dir_path: &Path,
    schema: &SchemaRef,
    mut receiver: mpsc::Receiver<RecordBatch>,
    event_sender: &broadcast::Sender<Event>,
) -> Result<()> {
    const TARGET_BATCH_SIZE: usize = 4096;
    const BIGGEST_COALESCE_BATCH_SIZE: usize = 64 * 1024 * 1024;
    const MAX_CHUNK_SIZE: u64 = 256 * 1024 * 1024; // 256MB

    let mut chunk_writer = ChunkWriter::new(dir_path, 0, schema)?;
    let mut coalescer = BatchCoalescer::new(schema.clone(), TARGET_BATCH_SIZE)
        .with_biggest_coalesce_batch_size(Some(BIGGEST_COALESCE_BATCH_SIZE));
    let mut chunk_index = 0;

    while let Some(batch) = receiver.blocking_recv() {
        coalescer.push_batch(batch)?;

        while let Some(coalesced_batch) = coalescer.next_completed_batch() {
            chunk_writer.write_batch(&coalesced_batch)?;

            if chunk_writer.should_rotate(MAX_CHUNK_SIZE) {
                chunk_writer.finish_chunk(event_sender)?;
                chunk_index += 1;
                chunk_writer = ChunkWriter::new(dir_path, chunk_index, schema)?;
            }
        }
    }

    // Finish any buffered batches
    coalescer.finish_buffered_batch()?;
    while let Some(coalesced_batch) = coalescer.next_completed_batch() {
        chunk_writer.write_batch(&coalesced_batch)?;
    }

    // Finish the final chunk
    chunk_writer.finish_chunk(event_sender)?;
    let _ = event_sender.send(Event::Closed);
    Ok(())
}

#[cfg(test)]
mod tests {
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
    async fn background_writer_basic() {
        let dir = tempdir().unwrap();
        let dataset_dir = dir.path();
        let schema = make_schema();
        let tracker = TaskTracker::new();
        let writer = BackgroundWriter::new(&tracker, dataset_dir, schema.clone());
        let mut rx = writer.subscribe();
        // write a couple batches
        writer.write(make_batch(&schema, 0, 5)).await.unwrap();
        writer.write(make_batch(&schema, 5, 5)).await.unwrap();
        // Drop sender side to close channel -> triggers flush & Closed event after task drains
        drop(writer);
        // Wait for task completion
        tracker.close();
        tracker.wait().await;
        // Collect events (order: some ChunkCompleted .., finally Closed)
        let mut saw_closed = false;
        for _ in 0..10 {
            // bounded to avoid hanging
            match rx.try_recv() {
                Ok(Event::Closed) => {
                    saw_closed = true;
                    break;
                }
                Ok(Event::ChunkCompleted { .. }) => {}
                Err(broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(_) => break,
            }
        }
        assert!(saw_closed, "expected Closed event");
        // File should exist and be non-empty (data_chunk_0.arrow)
        let chunk_path = chunk_path(dir.path(), 0);
        let meta = std::fs::metadata(&chunk_path).expect("file metadata");
        assert!(meta.len() > 0, "arrow file should have content");
    }

    #[tokio::test]
    async fn background_writer_chunking() {
        let dir = tempdir().unwrap();
        let dataset_dir = dir.path();
        let schema = make_schema();
        let tracker = TaskTracker::new();
        let writer = BackgroundWriter::new(&tracker, dataset_dir, schema.clone());

        // Write a few batches
        for i in 0..10 {
            let batch = make_batch(&schema, i * 10, 10);
            writer.write(batch).await.unwrap();
        }

        // Drop sender side to close channel
        drop(writer);

        // Wait for task completion
        tracker.close();
        tracker.wait().await;

        // Check that at least the first chunk file exists
        let chunk0_path = chunk_path(dir.path(), 0);
        assert!(chunk0_path.exists(), "data_chunk_0.arrow should exist");
        let meta = std::fs::metadata(&chunk0_path).expect("chunk 0 metadata");
        assert!(meta.len() > 0, "chunk 0 should have content");
    }
}

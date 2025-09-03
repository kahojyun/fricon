use arrow::{
    array::RecordBatch, compute::BatchCoalescer, datatypes::SchemaRef, ipc::writer::FileWriter,
};
use std::{fs::File, io::BufWriter, path::Path};
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
    Received,
    BatchWritten,
    Closed,
}

/// Background writer that persists incoming batches to an Arrow IPC file.
///
/// Responsibilities extracted from `WriteSession`:
/// - Own the mpsc sender for `RecordBatch`
/// - Own the broadcast event channel (Received / `BatchWritten` / Closed)
/// - Spawn blocking write / coalesce task
pub struct BackgroundWriter {
    sender: mpsc::Sender<RecordBatch>,
    event_sender: broadcast::Sender<Event>,
}

impl BackgroundWriter {
    pub fn new(tracker: &TaskTracker, path: impl AsRef<Path>, schema: SchemaRef) -> Self {
        let path = path.as_ref().to_path_buf();
        let (sender, receiver) = mpsc::channel(32);
        let (event_sender, _) = broadcast::channel(16);
        let event_sender_for_task = event_sender.clone();
        tracker.spawn_blocking(move || {
            if let Err(e) = blocking_write_task(&path, &schema, receiver, &event_sender_for_task) {
                error!("BackgroundWriter task failed: {e}");
            }
        });
        Self {
            sender,
            event_sender,
        }
    }

    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        // Fire Received event before enqueueing so listeners can react immediately
        let _ = self.event_sender.send(Event::Received);
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

fn blocking_write_task(
    path: &Path,
    schema: &SchemaRef,
    mut receiver: mpsc::Receiver<RecordBatch>,
    event_sender: &broadcast::Sender<Event>,
) -> Result<()> {
    const TARGET_BATCH_SIZE: usize = 4096;
    const BIGGEST_COALESCE_BATCH_SIZE: usize = 64 * 1024 * 1024;
    let file = File::create_new(path)?;
    let buf_writer = BufWriter::new(file);
    let mut writer = FileWriter::try_new(buf_writer, schema)?;
    let mut total_rows = 0usize;
    let mut coalescer = BatchCoalescer::new(schema.clone(), TARGET_BATCH_SIZE)
        .with_biggest_coalesce_batch_size(Some(BIGGEST_COALESCE_BATCH_SIZE));
    while let Some(batch) = receiver.blocking_recv() {
        coalescer.push_batch(batch)?;
        while let Some(coalesced_batch) = coalescer.next_completed_batch() {
            let rows = coalesced_batch.num_rows();
            writer.write(&coalesced_batch)?;
            total_rows += rows;
            let _ = event_sender.send(Event::BatchWritten);
        }
    }
    coalescer.finish_buffered_batch()?;
    while let Some(coalesced_batch) = coalescer.next_completed_batch() {
        let rows = coalesced_batch.num_rows();
        writer.write(&coalesced_batch)?;
        total_rows += rows;
        let _ = event_sender.send(Event::BatchWritten);
    }
    writer.finish()?;
    info!(
        "BackgroundWriter completed: {total_rows} rows written to {:?}",
        path
    );
    let _ = event_sender.send(Event::Closed);
    Ok::<(), Error>(())
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
        let path = dir.path().join("data.arrow");
        let schema = make_schema();
        let tracker = TaskTracker::new();
        let writer = BackgroundWriter::new(&tracker, &path, schema.clone());
        let mut rx = writer.subscribe();
        // write a couple batches
        writer.write(make_batch(&schema, 0, 5)).await.unwrap();
        writer.write(make_batch(&schema, 5, 5)).await.unwrap();
        // Drop sender side to close channel -> triggers flush & Closed event after task drains
        drop(writer);
        // Wait for task completion
        tracker.close();
        tracker.wait().await;
        // Collect events (order: some Received + BatchWritten .., finally Closed)
        let mut saw_closed = false;
        let mut saw_batch = false;
        for _ in 0..10 {
            // bounded to avoid hanging
            match rx.try_recv() {
                Ok(Event::BatchWritten) => saw_batch = true,
                Ok(Event::Closed) => {
                    saw_closed = true;
                    break;
                }
                Ok(Event::Received) => {}
                Err(broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
                Err(_) => break,
            }
        }
        assert!(saw_batch, "expected at least one BatchWritten event");
        assert!(saw_closed, "expected Closed event");
        // File should exist and be non-empty
        let meta = std::fs::metadata(&path).expect("file metadata");
        assert!(meta.len() > 0, "arrow file should have content");
    }
}

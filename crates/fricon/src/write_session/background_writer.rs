use std::{
    fs::File,
    io::{BufWriter, Error as IoError, Seek},
    path::{Path, PathBuf},
    result::Result as StdResult,
};

use arrow::{
    array::RecordBatch, compute::BatchCoalescer, datatypes::SchemaRef, error::ArrowError,
    ipc::writer::FileWriter,
};
use thiserror::Error;
use tokio::{
    sync::mpsc,
    task::{JoinError, JoinHandle},
};
use tokio_util::task::TaskTracker;
use tracing::{error, info};

use crate::utils::chunk_path;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Arrow error: {0}")]
    ArrowError(#[from] ArrowError),
    #[error("IO error: {0}")]
    Io(#[from] IoError),
    #[error("Send error: {0}")]
    Send(String),
    #[error("Task join error: {0}")]
    JoinError(#[from] JoinError),
}
pub type Result<T> = StdResult<T, Error>;

/// Background writer that persists incoming batches to chunked Arrow IPC files.
///
/// Responsibilities extracted from `WriteSession`:
/// - Own the mpsc sender for `RecordBatch`
/// - Send chunk completed notifications via mpsc channel
/// - (Previously had a separate closed notification channel; removed in favor
///   of awaiting join)
/// - Spawn blocking write / coalesce task
pub struct BackgroundWriter {
    sender: mpsc::Sender<RecordBatch>,
    // Join handle for the blocking write task so callers can await completion and capture errors.
    join: JoinHandle<Result<()>>,
}

impl BackgroundWriter {
    pub fn new(
        tracker: &TaskTracker,
        dir_path: impl Into<PathBuf>,
        schema: SchemaRef,
        chunk_completed_sender: mpsc::Sender<PathBuf>,
    ) -> Self {
        let dir_path = dir_path.into();
        let (sender, receiver) = mpsc::channel(32);

        // Spawn and keep the JoinHandle so we can explicitly await completion.
        let join = tracker.spawn_blocking(move || {
            blocking_write_task(&dir_path, &schema, receiver, &chunk_completed_sender)
        });
        Self { sender, join }
    }

    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        self.sender
            .send(batch)
            .await
            .map_err(|e| Error::Send(e.to_string()))?;
        Ok(())
    }

    /// Signal no more input (by dropping sender) and await background task
    /// completion.
    pub async fn shutdown(self) -> Result<()> {
        let join = self.join;
        // Dropping sender closes channel; writer task drains remaining buffered
        // batches.
        drop(self.sender);
        // Join returns Result<Result<()>>; map outer join errors.
        match join.await {
            Ok(inner) => inner,
            Err(e) => Err(Error::JoinError(e)),
        }
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

    fn finish_chunk(mut self, chunk_completed_sender: &mpsc::Sender<PathBuf>) -> Result<()> {
        self.writer.finish()?;
        let chunk_path = chunk_path(&self.dir_path, self.chunk_index);
        info!(
            "Chunk {} completed: {} rows written to {:?}",
            self.chunk_index, self.total_rows, chunk_path
        );
        let _ = chunk_completed_sender.blocking_send(chunk_path);
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
    chunk_completed_sender: &mpsc::Sender<PathBuf>,
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
                chunk_writer.finish_chunk(chunk_completed_sender)?;
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
    chunk_writer.finish_chunk(chunk_completed_sender)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc};

    use arrow::{
        array::Int32Array,
        datatypes::{DataType, Field, Schema},
    };
    use tempfile::tempdir;

    use super::*;

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

        // Create channels for testing
        let (chunk_completed_sender, mut chunk_completed_receiver) = mpsc::channel(16);

        let writer = BackgroundWriter::new(
            &tracker,
            dataset_dir,
            schema.clone(),
            chunk_completed_sender,
        );
        writer.write(make_batch(&schema, 0, 5)).await.unwrap();
        writer.write(make_batch(&schema, 5, 5)).await.unwrap();
        // Explicitly shutdown ensuring join awaited
        writer.shutdown().await.unwrap();

        // Wait for both signals concurrently
        let chunk_result = chunk_completed_receiver.recv().await;
        assert!(chunk_result.is_some(), "expected chunk completion");

        // File should exist and be non-empty (data_chunk_0.arrow)
        let chunk_path = chunk_path(dir.path(), 0);
        let meta = fs::metadata(&chunk_path).expect("file metadata");
        assert!(meta.len() > 0, "arrow file should have content");
    }

    #[tokio::test]
    async fn background_writer_chunking() {
        let dir = tempdir().unwrap();
        let dataset_dir = dir.path();
        let schema = make_schema();
        let tracker = TaskTracker::new();

        // Create channels for testing
        let (chunk_completed_sender, _chunk_completed_receiver) = mpsc::channel(16);

        let writer = BackgroundWriter::new(
            &tracker,
            dataset_dir,
            schema.clone(),
            chunk_completed_sender,
        );

        // Write a few batches
        for i in 0..10 {
            let batch = make_batch(&schema, i * 10, 10);
            writer.write(batch).await.unwrap();
        }

        // Explicitly shutdown
        writer.shutdown().await.unwrap();

        // Check that at least the first chunk file exists
        let chunk0_path = chunk_path(dir.path(), 0);
        assert!(chunk0_path.exists(), "data_chunk_0.arrow should exist");
        let meta = fs::metadata(&chunk0_path).expect("chunk 0 metadata");
        assert!(meta.len() > 0, "chunk 0 should have content");
    }
}

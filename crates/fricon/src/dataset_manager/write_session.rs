use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use arrow::{
    array::RecordBatch, compute::BatchCoalescer, datatypes::SchemaRef, ipc::writer::FileWriter,
};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinError;
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
    JoinError(#[from] JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Event {
    Received,
    BatchWritten,
    Closed,
}

pub struct WriteSession {
    sender: mpsc::Sender<RecordBatch>,
    #[allow(dead_code)]
    event_sender: broadcast::Sender<Event>,
}

impl WriteSession {
    pub fn new(tracker: &TaskTracker, path: impl AsRef<Path>, schema: SchemaRef) -> Self {
        let path = path.as_ref().to_path_buf();
        let (sender, receiver) = mpsc::channel(32);
        let (event_sender, _) = broadcast::channel(16);

        let event_sender_for_task = event_sender.clone();

        tracker.spawn(async move {
            if let Err(e) = Self::write_task(path, schema, receiver, event_sender_for_task).await {
                error!("Write task failed: {}", e);
            }
        });

        Self {
            sender,
            event_sender,
        }
    }

    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        let _ = self.event_sender.send(Event::Received);
        self.sender
            .send(batch)
            .await
            .map_err(|e| Error::Send(e.to_string()))?;
        Ok(())
    }

    // TODO: Will be used by real-time plotting
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_sender.subscribe()
    }

    async fn write_task(
        path: PathBuf,
        schema: SchemaRef,
        mut receiver: mpsc::Receiver<RecordBatch>,
        event_sender: broadcast::Sender<Event>,
    ) -> Result<()> {
        const TARGET_BATCH_SIZE: usize = 4096;
        const BIGGEST_COALESCE_BATCH_SIZE: usize = 64 * 1024 * 1024; // 64MB

        let path_clone = path.clone();
        let event_sender_clone = event_sender.clone();

        let result = tokio::task::spawn_blocking(move || {
            let file = File::create_new(&path_clone)?;
            let buf_writer = BufWriter::new(file);
            let mut writer = FileWriter::try_new(buf_writer, &schema)?;

            let mut total_rows = 0;
            let mut coalescer = BatchCoalescer::new(schema.clone(), TARGET_BATCH_SIZE)
                .with_biggest_coalesce_batch_size(Some(BIGGEST_COALESCE_BATCH_SIZE));

            while let Some(batch) = receiver.blocking_recv() {
                coalescer.push_batch(batch)?;

                while let Some(coalesced_batch) = coalescer.next_completed_batch() {
                    let rows = coalesced_batch.num_rows();
                    writer.write(&coalesced_batch)?;
                    total_rows += rows;

                    let _ = event_sender_clone.send(Event::BatchWritten);
                }
            }

            coalescer.finish_buffered_batch()?;

            while let Some(coalesced_batch) = coalescer.next_completed_batch() {
                let rows = coalesced_batch.num_rows();
                writer.write(&coalesced_batch)?;
                total_rows += rows;

                let _ = event_sender_clone.send(Event::BatchWritten);
            }

            writer.finish()?;
            info!(
                "Write session completed: {} rows written to {:?}",
                total_rows, path_clone
            );
            let _ = event_sender_clone.send(Event::Closed);

            Ok::<(), Error>(())
        })
        .await;

        result??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::{
        array::{Int32Array, StringArray},
        datatypes::{DataType, Field, Schema},
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    use tempfile::tempdir;
    use tokio::time::timeout;
    use tokio_util::task::TaskTracker;

    // Helper function to create a test session and return common components
    fn setup_test_session() -> (
        tempfile::TempDir,
        PathBuf,
        Schema,
        TaskTracker,
        WriteSession,
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.arrow");
        let schema = create_test_schema();
        let tracker = TaskTracker::new();
        let session = WriteSession::new(&tracker, &file_path, Arc::new(schema.clone()));
        (temp_dir, file_path, schema, tracker, session)
    }

    // Helper function to wait for and assert a specific event
    async fn expect_event(
        mut events: broadcast::Receiver<Event>,
        expected: Event,
        timeout_ms: u64,
    ) -> broadcast::Receiver<Event> {
        let event = timeout(Duration::from_millis(timeout_ms), events.recv())
            .await
            .unwrap()
            .unwrap();
        match (event, expected) {
            (Event::Received, Event::Received)
            | (Event::BatchWritten, Event::BatchWritten)
            | (Event::Closed, Event::Closed) => {}
            (actual, expected) => panic!("Expected {expected:?}, got {actual:?}"),
        }
        events
    }

    // Helper function to wait for Closed event, skipping BatchWritten events
    async fn wait_for_closed_event(mut events: broadcast::Receiver<Event>) {
        // Wait a bit for the session to close
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Try to receive Closed event, skip any BatchWritten events
        let start_time = Instant::now();
        while start_time.elapsed() < Duration::from_millis(200) {
            match timeout(Duration::from_millis(50), events.recv()).await {
                Ok(Ok(Event::Closed)) => {
                    // Found Closed event, return
                    return;
                }
                Ok(Ok(Event::BatchWritten)) | Err(_) => {
                    // Skip BatchWritten events or timeout, continue loop
                }
                Ok(Ok(other)) => panic!("Unexpected event: {other:?}"),
                Ok(Err(_)) => panic!("Failed to receive event"),
            }
        }
        panic!("Did not receive Closed event within timeout");
    }

    fn create_test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ])
    }

    fn create_test_batch(schema: &Schema, start_id: i32, num_rows: i32) -> RecordBatch {
        let id_array = Int32Array::from_iter_values(start_id..(start_id + num_rows));
        let name_array =
            StringArray::from_iter_values((0..num_rows).map(|i| format!("name_{}", start_id + i)));

        RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![Arc::new(id_array) as _, Arc::new(name_array) as _],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_write_session_basic() {
        let (_temp_dir, file_path, schema, _tracker, session) = setup_test_session();

        let batch = create_test_batch(&schema, 0, 10);
        session.write(batch).await.unwrap();

        drop(session);
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_write_session_events() {
        let (_temp_dir, _file_path, schema, _tracker, session) = setup_test_session();
        let mut events = session.subscribe();

        let batch = create_test_batch(&schema, 0, 5);
        session.write(batch).await.unwrap();

        // Should receive Received event immediately
        events = expect_event(events, Event::Received, 100).await;

        drop(session);
        wait_for_closed_event(events).await;
    }

    #[tokio::test]
    async fn test_write_multiple_batches() {
        use arrow::ipc::reader::FileReader;
        let (_temp_dir, file_path, schema, _tracker, session) = setup_test_session();
        let mut events = session.subscribe();

        for i in 0..3 {
            let batch = create_test_batch(&schema, i * 10, 10);
            session.write(batch).await.unwrap();

            // Should receive Received event for each batch
            events = expect_event(events, Event::Received, 100).await;
        }

        drop(session);
        wait_for_closed_event(events).await;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let file = std::fs::File::open(&file_path).unwrap();
        let reader = FileReader::try_new(file, None).unwrap();

        let mut total_rows = 0;
        for batch in reader {
            total_rows += batch.unwrap().num_rows();
        }
        assert_eq!(total_rows, 30);
    }

    #[tokio::test]
    async fn test_batch_coalescing() {
        use arrow::ipc::reader::FileReader;
        let (_temp_dir, file_path, schema, _tracker, session) = setup_test_session();

        // Write multiple small batches that should be coalesced
        for i in 0..10 {
            let batch = create_test_batch(&schema, i * 100, 100); // 100 rows each
            session.write(batch).await.unwrap();
        }

        drop(session);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify the file was created
        assert!(file_path.exists());

        // Read back and verify coalescing occurred
        let file = std::fs::File::open(&file_path).unwrap();
        let reader = FileReader::try_new(file, None).unwrap();

        let mut batch_count = 0;
        let mut total_rows = 0;
        for batch in reader {
            let batch = batch.unwrap();
            batch_count += 1;
            total_rows += batch.num_rows();
            println!("Batch {} has {} rows", batch_count, batch.num_rows());
        }

        // Should have 1000 rows total
        assert_eq!(total_rows, 1000);
        // Should have fewer batches than input due to coalescing
        assert!(batch_count < 10);
    }
}

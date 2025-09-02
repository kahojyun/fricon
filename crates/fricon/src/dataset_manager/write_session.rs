use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use arrow::{array::RecordBatch, datatypes::SchemaRef, ipc::writer::FileWriter};
use futures::{Stream, StreamExt};
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
        self.sender
            .send(batch)
            .await
            .map_err(|e| Error::Send(e.to_string()))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_sender.subscribe()
    }

    #[allow(dead_code)]
    pub async fn write_stream<S, E>(&self, mut stream: S) -> Result<()>
    where
        S: Stream<Item = std::result::Result<RecordBatch, E>> + Send + 'static + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        while let Some(result) = stream.next().await {
            let batch = result.map_err(|e| Error::Send(e.to_string()))?;
            self.write(batch).await?;
        }
        Ok(())
    }

    async fn write_task(
        path: PathBuf,
        schema: SchemaRef,
        mut receiver: mpsc::Receiver<RecordBatch>,
        event_sender: broadcast::Sender<Event>,
    ) -> Result<()> {
        let path_clone = path.clone();
        let event_sender_clone = event_sender.clone();

        let result = tokio::task::spawn_blocking(move || {
            let file = File::create_new(&path_clone)?;
            let buf_writer = BufWriter::new(file);
            let mut writer = FileWriter::try_new(buf_writer, &schema)?;

            let mut total_rows = 0;
            while let Some(batch) = receiver.blocking_recv() {
                let rows = batch.num_rows();
                writer.write(&batch)?;
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
    use std::{sync::Arc, time::Duration};
    use tempfile::tempdir;
    use tokio::time::timeout;
    use tokio_util::task::TaskTracker;

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
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.arrow");
        let schema = create_test_schema();

        let tracker = TaskTracker::new();
        let session = WriteSession::new(&tracker, &file_path, Arc::new(schema.clone()));

        let batch = create_test_batch(&schema, 0, 10);
        session.write(batch).await.unwrap();

        drop(session);
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_write_session_events() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.arrow");
        let schema = create_test_schema();

        let tracker = TaskTracker::new();
        let session = WriteSession::new(&tracker, &file_path, Arc::new(schema.clone()));
        let mut events = session.subscribe();

        let batch = create_test_batch(&schema, 0, 5);
        session.write(batch).await.unwrap();

        let event = timeout(Duration::from_millis(100), events.recv())
            .await
            .unwrap()
            .unwrap();
        match event {
            Event::BatchWritten => {}
            Event::Closed => panic!("Expected BatchWritten event"),
        }

        drop(session);

        let event = timeout(Duration::from_millis(100), events.recv())
            .await
            .unwrap()
            .unwrap();
        match event {
            Event::Closed => {}
            Event::BatchWritten => panic!("Expected Closed event"),
        }
    }

    #[tokio::test]
    async fn test_write_multiple_batches() {
        use arrow::ipc::reader::FileReader;
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.arrow");
        let schema = create_test_schema();

        let tracker = TaskTracker::new();
        let session = WriteSession::new(&tracker, &file_path, Arc::new(schema.clone()));
        let mut events = session.subscribe();

        for i in 0..3 {
            let batch = create_test_batch(&schema, i * 10, 10);
            session.write(batch).await.unwrap();

            let event = timeout(Duration::from_millis(100), events.recv())
                .await
                .unwrap()
                .unwrap();
            match event {
                Event::BatchWritten => {}
                Event::Closed => panic!("Expected BatchWritten event"),
            }
        }

        drop(session);
        let event = timeout(Duration::from_millis(100), events.recv())
            .await
            .unwrap()
            .unwrap();
        match event {
            Event::Closed => {}
            Event::BatchWritten => panic!("Expected Closed event"),
        }

        tokio::time::sleep(Duration::from_millis(10)).await;

        let file = std::fs::File::open(&file_path).unwrap();
        let reader = FileReader::try_new(file, None).unwrap();

        let mut total_rows = 0;
        for batch in reader {
            total_rows += batch.unwrap().num_rows();
        }
        assert_eq!(total_rows, 30);
    }
}

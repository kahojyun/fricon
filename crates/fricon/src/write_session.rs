use crate::live::LiveDataset;
use arrow::{
    array::RecordBatch, compute::BatchCoalescer, datatypes::SchemaRef, ipc::writer::FileWriter,
};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
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
#[derive(Clone)]
pub struct WriteSession {
    sender: mpsc::Sender<RecordBatch>,
    #[allow(dead_code)]
    event_sender: broadcast::Sender<Event>,
    live: LiveDataset,
}
impl WriteSession {
    pub fn new(tracker: &TaskTracker, path: impl AsRef<Path>, schema: SchemaRef) -> Self {
        let path = path.as_ref().to_path_buf();
        let live = LiveDataset::new(schema.clone(), path.clone());
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
            live,
        }
    }
    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        let _ = self.event_sender.send(Event::Received);
        self.live.append(batch.clone());
        self.sender
            .send(batch)
            .await
            .map_err(|e| Error::Send(e.to_string()))?;
        Ok(())
    }
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
        const BIGGEST_COALESCE_BATCH_SIZE: usize = 64 * 1024 * 1024;
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
    pub fn live(&self) -> LiveDataset {
        self.live.clone()
    }
}

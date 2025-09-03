use arrow::{array::RecordBatch, datatypes::SchemaRef};
use std::path::Path;
use tokio::sync::broadcast;
use tokio_util::task::TaskTracker;

use crate::background_writer::{BackgroundWriter, Event, Result};
use crate::live::LiveDataset;
/// High-level write session combining a `BackgroundWriter` with a `LiveDataset`.
///
/// Separation of concerns:
/// - `BackgroundWriter` handles persistence & events
/// - `WriteSession` enriches by updating the in-memory `LiveDataset`
pub struct WriteSession {
    writer: BackgroundWriter,
    live: LiveDataset,
}

impl WriteSession {
    pub fn new(tracker: &TaskTracker, path: impl AsRef<Path>, schema: SchemaRef) -> Self {
        let path_ref = path.as_ref();
        let live = LiveDataset::new(schema.clone(), path_ref.to_path_buf());
        let writer = BackgroundWriter::new(tracker, path_ref, schema);
        Self { writer, live }
    }
    pub async fn write(&self, batch: RecordBatch) -> Result<()> {
        self.live.append(batch.clone());
        self.writer.write(batch).await
    }
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.writer.subscribe()
    }
    pub fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle {
            live: self.live.clone(),
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

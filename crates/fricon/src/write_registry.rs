use std::{
    collections::HashMap,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use arrow_schema::SchemaRef;
use tokio_util::task::TaskTracker;

use crate::write_session::{
    WriteSession, WriteSessionHandle, background_writer::Result as BackgroundResult,
};

#[derive(Clone, Default)]
pub struct WriteSessionRegistry {
    inner: Arc<RwLock<HashMap<i32, WriteSessionHandle>>>,
}

impl WriteSessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn create(
        &self,
        id: i32,
        tracker: &TaskTracker,
        path: impl Into<PathBuf>,
        schema: SchemaRef,
    ) -> WriteSessionGuard {
        let session = WriteSession::new(tracker, path, schema);
        if let Ok(mut m) = self.inner.write() {
            m.insert(id, session.handle());
        }
        WriteSessionGuard {
            id,
            registry: self.clone(),
            session: Some(session),
        }
    }
    pub fn get(&self, id: i32) -> Option<WriteSessionHandle> {
        self.inner.read().ok().and_then(|m| m.get(&id).cloned())
    }
    fn remove(&self, id: i32) {
        if let Ok(mut m) = self.inner.write() {
            m.remove(&id);
        }
    }
}

/// Owner of a `WriteSession`, ensuring it is properly cleaned up.
pub struct WriteSessionGuard {
    id: i32,
    registry: WriteSessionRegistry,
    // Wrapped so we can drop the underlying WriteSession (closing its senders)
    // without dropping the guard itself. This lets us keep the registry entry
    // alive until the background writer signals Closed, ensuring readers can
    // still obtain a live handle while final flush is in progress.
    session: Option<WriteSession>,
}
impl Deref for WriteSessionGuard {
    type Target = WriteSession;
    fn deref(&self) -> &Self::Target {
        self.session
            .as_ref()
            .expect("Session should be available as finish() hasn't been called yet")
    }
}
impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        // If finish() already ran we will have explicitly removed; removal is
        // idempotent.
        self.registry.remove(self.id);
    }
}

impl WriteSessionGuard {
    /// Flush remaining buffered batches and wait until the underlying
    /// background writer signals closed. This guarantees that all chunk
    /// files have been finalized on disk.
    pub async fn finish(mut self) -> BackgroundResult<()> {
        if let Some(session) = self.session.take() {
            // Await background writer completion so chunk files are guaranteed finalized.
            let res = session.shutdown().await;
            // Remove from registry regardless of success/failure (idempotent).
            self.registry.remove(self.id);
            return res;
        }
        // Already taken; ensure registry removal.
        self.registry.remove(self.id);
        Ok(())
    }
}

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use arrow::datatypes::SchemaRef;
use tokio_util::task::TaskTracker;

use crate::write_session::background_writer::{
    Error as BackgroundError, Event, Result as BackgroundResult,
};
use crate::write_session::{WriteSession, WriteSessionHandle};

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
            .expect("session already taken in finish()")
    }
}
impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        // If finish() already ran we will have explicitly removed; removal is idempotent.
        self.registry.remove(self.id);
    }
}

impl WriteSessionGuard {
    /// Flush remaining buffered batches and wait until the underlying background writer
    /// signals `Closed`. This guarantees that all chunk files have been finalized on disk.
    pub async fn finish(mut self) -> BackgroundResult<()> {
        // Subscribe before dropping the session so we can observe Closed.
        let mut rx = self.session.as_ref().unwrap().subscribe();
        let id = self.id;
        // Drop only the underlying WriteSession (closes mpsc sender) but keep guard so
        // registry entry is still present until we observe Closed.
        drop(self.session.take());
        let res = loop {
            match rx.recv().await {
                Ok(Event::Closed) => break Ok(()),
                Ok(_) => { /* ignore other events */ }
                Err(e) => {
                    break Err(BackgroundError::Send(format!(
                        "session {id} channel closed before Closed event: {e}"
                    )));
                }
            }
        };
        // Now that background writer is fully closed, explicitly remove.
        self.registry.remove(self.id);
        res
    }
}

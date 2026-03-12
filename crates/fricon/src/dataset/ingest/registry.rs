use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use arrow_schema::SchemaRef;
use tracing::debug;

use crate::dataset::ingest::{IngestError, WriteSessionHandle, session::WriteSession};

#[derive(Clone, Default)]
pub(crate) struct WriteSessionRegistry {
    inner: Arc<RwLock<HashMap<i32, WriteSessionHandle>>>,
}

pub(crate) struct WriteSessionGuard {
    id: i32,
    registry: WriteSessionRegistry,
    session: Option<WriteSession>,
}

impl WriteSessionGuard {
    fn session_mut(&mut self) -> &mut WriteSession {
        self.session.as_mut().expect("Write session missing")
    }

    pub(crate) fn write_batch(
        &mut self,
        batch: arrow_array::RecordBatch,
    ) -> Result<(), IngestError> {
        self.session_mut().write(batch)
    }

    pub(crate) fn commit_session(mut self) -> Result<(), IngestError> {
        if let Some(session) = self.session.take() {
            session.finish()?;
        }
        debug!(dataset.id = self.id, "Write session committed");
        Ok(())
    }

    pub(crate) fn abort_session(mut self) -> Result<(), IngestError> {
        if let Some(session) = self.session.take() {
            session.abort()?;
        }
        debug!(dataset.id = self.id, "Write session aborted");
        Ok(())
    }
}

impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            debug!(
                dataset.id = self.id,
                "Write session dropped without commit, aborting"
            );
            let _ = session.abort();
        }
        self.registry.remove(self.id);
    }
}

impl WriteSessionRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn start_session(
        &self,
        id: i32,
        path: PathBuf,
        schema: SchemaRef,
    ) -> WriteSessionGuard {
        let session = WriteSession::new(schema, path);
        if let Ok(mut m) = self.inner.write() {
            m.insert(id, session.handle());
        }
        debug!(dataset.id = id, "Write session started");
        WriteSessionGuard {
            id,
            registry: self.clone(),
            session: Some(session),
        }
    }

    pub(crate) fn get(&self, id: i32) -> Option<WriteSessionHandle> {
        self.inner.read().ok().and_then(|m| m.get(&id).cloned())
    }
    fn remove(&self, id: i32) {
        if let Ok(mut m) = self.inner.write() {
            m.remove(&id);
        }
    }
}

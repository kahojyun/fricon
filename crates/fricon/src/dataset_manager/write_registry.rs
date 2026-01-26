use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use arrow_schema::SchemaRef;

use crate::dataset_manager::{
    Error,
    write_session::{WriteSession, WriteSessionHandle},
};

#[derive(Clone, Default)]
pub struct WriteSessionRegistry {
    inner: Arc<RwLock<HashMap<i32, WriteSessionHandle>>>,
}

pub struct WriteSessionGuard {
    id: i32,
    registry: WriteSessionRegistry,
    session: Option<WriteSession>,
}

impl WriteSessionGuard {
    pub fn session_mut(&mut self) -> &mut WriteSession {
        self.session.as_mut().expect("Write session missing")
    }

    pub fn write(&mut self, batch: arrow_array::RecordBatch) -> Result<(), Error> {
        self.session_mut().write(batch)
    }

    pub fn commit(mut self) -> Result<(), Error> {
        if let Some(session) = self.session.take() {
            session.finish()?;
        }
        Ok(())
    }

    pub fn abort(mut self) -> Result<(), Error> {
        if let Some(session) = self.session.take() {
            session.abort()?;
        }
        Ok(())
    }
}

impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            let _ = session.abort();
        }
        self.registry.remove(self.id);
    }
}

impl WriteSessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_session(&self, id: i32, path: PathBuf, schema: SchemaRef) -> WriteSessionGuard {
        let session = WriteSession::new(schema, path);
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

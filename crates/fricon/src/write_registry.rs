use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, RwLock};

use arrow::datatypes::SchemaRef;
use tokio_util::task::TaskTracker;

use crate::write_session::WriteSession;

#[derive(Clone, Default)]
pub struct WriteSessionRegistry {
    inner: Arc<RwLock<HashMap<i32, WriteSession>>>,
}

impl WriteSessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn create(
        &self,
        id: i32,
        tracker: &TaskTracker,
        path: impl AsRef<Path>,
        schema: SchemaRef,
    ) -> WriteSessionGuard {
        let session = WriteSession::new(tracker, path, schema);
        if let Ok(mut m) = self.inner.write() {
            m.insert(id, session.clone());
        }
        WriteSessionGuard {
            id,
            registry: self.clone(),
            session,
        }
    }
    pub fn get(&self, id: i32) -> Option<WriteSession> {
        self.inner.read().ok().and_then(|m| m.get(&id).cloned())
    }
    fn remove(&self, id: i32) {
        if let Ok(mut m) = self.inner.write() {
            m.remove(&id);
        }
    }
}

pub struct WriteSessionGuard {
    id: i32,
    registry: WriteSessionRegistry,
    session: WriteSession,
}
impl Deref for WriteSessionGuard {
    type Target = WriteSession;
    fn deref(&self) -> &Self::Target {
        &self.session
    }
}
impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        self.registry.remove(self.id);
    }
}

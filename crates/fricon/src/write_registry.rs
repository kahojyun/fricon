use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use arrow_schema::SchemaRef;

use crate::{
    DatasetManagerError,
    write_session::{WriteSession, WriteSessionHandle},
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
        path: PathBuf,
        schema: SchemaRef,
    ) -> Result<WriteSession, DatasetManagerError> {
        let session = WriteSession::new(id, self.clone(), schema, path)?;
        if let Ok(mut m) = self.inner.write() {
            m.insert(id, session.handle());
        }
        Ok(session)
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

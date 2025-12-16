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

impl WriteSessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_session<R>(
        &self,
        id: i32,
        path: PathBuf,
        schema: SchemaRef,
        f: impl FnOnce(&mut WriteSession) -> Result<R, Error>,
    ) -> Result<R, Error> {
        struct Guard(i32, WriteSessionRegistry);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.1.remove(self.0);
            }
        }

        let mut session = WriteSession::new(schema, path);
        if let Ok(mut m) = self.inner.write() {
            m.insert(id, session.handle());
        }
        let _guard = Guard(id, self.clone());
        let result = f(&mut session)?;
        session.finish()?;
        Ok(result)
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

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
        self.finalize_writer()?;
        debug!(
            dataset.id = self.id,
            "Write session finalized for aborted dataset"
        );
        Ok(())
    }

    fn finalize_writer(&mut self) -> Result<(), IngestError> {
        if let Some(session) = self.session.take() {
            session.finalize_writer()?;
        }
        Ok(())
    }
}

impl Drop for WriteSessionGuard {
    fn drop(&mut self) {
        if self.session.is_some() {
            debug!(
                dataset.id = self.id,
                "Write session dropped without commit, finalizing persisted partial data"
            );
            let _ = self.finalize_writer();
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_schema::{DataType, Field, Schema};
    use tempfile::TempDir;

    use super::WriteSessionRegistry;
    use crate::dataset::storage::ChunkReader;

    fn test_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]))
    }

    fn test_batch(values: Vec<i32>) -> RecordBatch {
        RecordBatch::try_new(test_schema(), vec![Arc::new(Int32Array::from(values))]).unwrap()
    }

    fn setup_session_dir() -> TempDir {
        TempDir::new().expect("temp dir")
    }

    #[test]
    fn committed_session_marks_complete_and_persists_data() {
        let dir = setup_session_dir();
        let registry = WriteSessionRegistry::new();
        let mut guard = registry.start_session(1, dir.path().to_owned(), test_schema());

        guard.write_batch(test_batch(vec![1, 2, 3])).unwrap();
        let handle = registry.get(1).expect("handle exists during session");
        assert!(!handle.is_complete());
        assert_eq!(handle.num_rows(), 3);

        guard.commit_session().unwrap();

        let mut reader = ChunkReader::new(dir.path().to_owned(), Some(test_schema()));
        reader.read_all().unwrap();
        assert_eq!(reader.num_rows(), 3);
    }

    #[test]
    fn aborted_session_persists_data_without_marking_complete() {
        let dir = setup_session_dir();
        let registry = WriteSessionRegistry::new();
        let mut guard = registry.start_session(1, dir.path().to_owned(), test_schema());

        guard.write_batch(test_batch(vec![10, 20])).unwrap();
        guard.abort_session().unwrap();

        let mut reader = ChunkReader::new(dir.path().to_owned(), Some(test_schema()));
        reader.read_all().unwrap();
        assert_eq!(reader.num_rows(), 2);
    }

    #[test]
    fn dropped_session_finalizes_and_cleans_up_registry() {
        let dir = setup_session_dir();
        let registry = WriteSessionRegistry::new();
        let mut guard = registry.start_session(1, dir.path().to_owned(), test_schema());

        guard.write_batch(test_batch(vec![7])).unwrap();
        assert!(registry.get(1).is_some());
        drop(guard);

        assert!(registry.get(1).is_none());

        let mut reader = ChunkReader::new(dir.path().to_owned(), Some(test_schema()));
        reader.read_all().unwrap();
        assert_eq!(reader.num_rows(), 1);
    }

    #[test]
    fn empty_commit_succeeds_with_no_persisted_data() {
        let dir = setup_session_dir();
        let registry = WriteSessionRegistry::new();
        let guard = registry.start_session(1, dir.path().to_owned(), test_schema());

        guard.commit_session().unwrap();

        let mut reader = ChunkReader::new(dir.path().to_owned(), Some(test_schema()));
        reader.read_all().unwrap();
        assert_eq!(reader.num_rows(), 0);
    }
}

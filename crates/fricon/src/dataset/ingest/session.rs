use std::{
    borrow::Cow,
    ops::RangeBounds,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::dataset::{
    ingest::{IngestError, storage::InProgressTable},
    storage::ChunkWriter,
};

pub(super) struct WriteSession {
    writer: ChunkWriter,
    in_progress_table: Arc<Mutex<InProgressTable>>,
}

impl WriteSession {
    pub(super) fn new(schema: SchemaRef, dir_path: PathBuf) -> Self {
        let writer = ChunkWriter::new(schema.clone(), dir_path.clone());
        let in_progress_table = InProgressTable::new(schema, dir_path);
        let in_progress_table = Arc::new(Mutex::new(in_progress_table));
        Self {
            writer,
            in_progress_table,
        }
    }

    pub(super) fn write(&mut self, batch: RecordBatch) -> Result<(), IngestError> {
        self.in_progress_table_mut().push(batch.clone())?;
        if self.writer.write(batch)? {
            self.in_progress_table_mut().continue_read_chunks()?;
        }
        Ok(())
    }

    pub(super) fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle(self.in_progress_table.clone())
    }

    pub(super) fn finish(self) -> Result<(), IngestError> {
        self.in_progress_table_mut().mark_complete();
        self.writer.finish()?;
        Ok(())
    }

    pub(super) fn finalize_writer(self) -> Result<(), IngestError> {
        self.writer.finish()?;
        Ok(())
    }

    fn in_progress_table_mut(&self) -> MutexGuard<'_, InProgressTable> {
        self.in_progress_table
            .lock()
            .expect("Should not be poisoned.")
    }
}

/// A shareable handle to a `WriteSession`, allowing concurrent access.
#[derive(Clone)]
pub(crate) struct WriteSessionHandle(Arc<Mutex<InProgressTable>>);

impl WriteSessionHandle {
    fn inner(&self) -> MutexGuard<'_, InProgressTable> {
        self.0.lock().expect("Should be poisoned")
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.0.lock().expect("Should not be poisoned").is_complete()
    }

    pub(crate) fn schema(&self) -> SchemaRef {
        self.inner().schema().clone()
    }

    pub(crate) fn num_rows(&self) -> usize {
        self.inner().num_rows()
    }

    pub(crate) fn snapshot_status(&self) -> (usize, bool) {
        let inner = self.inner();
        (inner.num_rows(), inner.is_complete())
    }

    pub(crate) fn snapshot_range<R>(&self, range: R) -> Vec<RecordBatch>
    where
        R: RangeBounds<usize> + Copy,
    {
        let inner = self.inner();
        inner.range(range).map(Cow::into_owned).collect()
    }

    pub(crate) fn snapshot_range_with_schema<R>(&self, range: R) -> (SchemaRef, Vec<RecordBatch>)
    where
        R: RangeBounds<usize> + Copy,
    {
        let inner = self.inner();
        let schema = inner.schema().clone();
        let batches = inner.range(range).map(Cow::into_owned).collect();
        (schema, batches)
    }
}

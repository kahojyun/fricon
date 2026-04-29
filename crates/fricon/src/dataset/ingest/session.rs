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
    semantics::{materialize_record_ids, materialized_schema},
    storage::ChunkWriter,
};

pub(super) struct WriteSession {
    writer: ChunkWriter,
    in_progress_table: Arc<Mutex<InProgressTable>>,
    storage_schema: SchemaRef,
    next_record_id: u64,
}

impl WriteSession {
    pub(super) fn new(schema: &SchemaRef, dir_path: PathBuf) -> Self {
        let storage_schema = materialized_schema(schema.as_ref());
        let writer = ChunkWriter::new(storage_schema.clone(), dir_path.clone());
        let in_progress_table = InProgressTable::new(storage_schema.clone(), dir_path);
        let in_progress_table = Arc::new(Mutex::new(in_progress_table));
        Self {
            writer,
            in_progress_table,
            storage_schema,
            next_record_id: 0,
        }
    }

    pub(super) fn write(&mut self, batch: &RecordBatch) -> Result<(), IngestError> {
        let (batch, next_record_id) =
            materialize_record_ids(self.storage_schema.clone(), batch, self.next_record_id)?;
        self.in_progress_table_mut().push(batch.clone())?;
        if self.writer.write(batch)? {
            self.in_progress_table_mut().continue_read_chunks()?;
        }
        self.next_record_id = next_record_id;
        Ok(())
    }

    pub(super) fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle(self.in_progress_table.clone())
    }

    pub(super) fn num_rows(&self) -> usize {
        self.in_progress_table_mut().num_rows()
    }

    pub(super) fn finish(self) -> Result<(), IngestError> {
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

    pub(crate) fn schema(&self) -> SchemaRef {
        self.inner().schema().clone()
    }

    pub(crate) fn num_rows(&self) -> usize {
        self.inner().num_rows()
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Float64Array, RecordBatch, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};
    use tempfile::TempDir;

    use super::WriteSession;
    use crate::dataset::{semantics::RECORD_ID_COLUMN, storage::ChunkReader};

    fn user_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![Field::new(
            "signal",
            DataType::Float64,
            false,
        )]))
    }

    fn batch(values: Vec<f64>) -> RecordBatch {
        RecordBatch::try_new(user_schema(), vec![Arc::new(Float64Array::from(values))])
            .expect("batch")
    }

    #[test]
    fn write_session_materializes_monotonic_record_ids() {
        let dir = TempDir::new().expect("temp dir");
        let mut session = WriteSession::new(&user_schema(), dir.path().to_owned());

        session
            .write(&batch(vec![10.0, 20.0]))
            .expect("first write");
        session.write(&batch(vec![30.0])).expect("second write");
        session.finish().expect("finish");

        let mut reader = ChunkReader::new(dir.path().to_owned(), None);
        reader.read_all().expect("read chunks");
        let batches: Vec<_> = reader.range(..).map(std::borrow::Cow::into_owned).collect();
        let stored =
            arrow_select::concat::concat_batches(reader.schema().expect("schema"), &batches)
                .expect("concat");

        assert_eq!(stored.schema().field(0).name(), RECORD_ID_COLUMN);
        let record_ids = stored
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .expect("record ids");
        assert_eq!(record_ids.values(), &[0, 1, 2]);
    }

    #[test]
    fn write_session_empty_batch_does_not_advance_record_ids() {
        let dir = TempDir::new().expect("temp dir");
        let mut session = WriteSession::new(&user_schema(), dir.path().to_owned());

        session.write(&batch(Vec::new())).expect("empty write");
        session.write(&batch(vec![10.0])).expect("second write");
        session.finish().expect("finish");

        let mut reader = ChunkReader::new(dir.path().to_owned(), None);
        reader.read_all().expect("read chunks");
        let batches: Vec<_> = reader.range(..).map(std::borrow::Cow::into_owned).collect();
        let stored =
            arrow_select::concat::concat_batches(reader.schema().expect("schema"), &batches)
                .expect("concat");
        let record_ids = stored
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .expect("record ids");

        assert_eq!(record_ids.values(), &[0]);
    }
}

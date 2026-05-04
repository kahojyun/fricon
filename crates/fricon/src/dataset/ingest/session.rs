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
    schema::DatasetError,
    semantics::{ScanPlan, materialize_record_ids, materialized_schema},
    storage::{
        ChunkWriter,
        layout::ChunkKind,
        logical_index::{build_logical_index_batch, logical_index_schema},
    },
};

pub(super) struct WriteSession {
    writer: ChunkWriter,
    in_progress_table: Arc<Mutex<InProgressTable>>,
    user_schema: SchemaRef,
    storage_schema: SchemaRef,
    scan_plan: Option<ScanPlan>,
    logical_index_writer: Option<ChunkWriter>,
    next_record_id: u64,
}

impl WriteSession {
    pub(super) fn new(
        schema: &SchemaRef,
        dir_path: PathBuf,
        scan_plan: Option<ScanPlan>,
        logical_index_sidecar: bool,
    ) -> Self {
        let storage_schema = materialized_schema(schema.as_ref());
        let writer = ChunkWriter::new(storage_schema.clone(), dir_path.clone());
        let logical_index_writer = scan_plan.as_ref().and_then(|scan_plan| {
            logical_index_sidecar.then(|| {
                ChunkWriter::new_with_kind(
                    logical_index_schema(scan_plan),
                    dir_path.clone(),
                    ChunkKind::LogicalIndex,
                )
            })
        });
        let in_progress_table = InProgressTable::new(storage_schema.clone(), dir_path);
        let in_progress_table = Arc::new(Mutex::new(in_progress_table));
        Self {
            writer,
            in_progress_table,
            user_schema: schema.clone(),
            storage_schema,
            scan_plan,
            logical_index_writer,
            next_record_id: 0,
        }
    }

    pub(super) fn write(
        &mut self,
        batch: &RecordBatch,
        logical_indices: Option<&RecordBatch>,
    ) -> Result<(), IngestError> {
        if batch.schema() != self.user_schema {
            return Err(IngestError::Dataset(DatasetError::SchemaMismatch));
        }
        let (batch, next_record_id) =
            materialize_record_ids(self.storage_schema.clone(), batch, self.next_record_id)?;
        match (
            &self.scan_plan,
            &mut self.logical_index_writer,
            logical_indices,
        ) {
            (Some(scan_plan), Some(writer), Some(logical_indices)) => {
                let logical_index_batch =
                    build_logical_index_batch(scan_plan, &batch, logical_indices)?;
                writer.write(logical_index_batch)?;
            }
            (_, Some(_), None) | (_, None, Some(_)) | (None, Some(_), Some(_)) => {
                return Err(IngestError::Dataset(DatasetError::SchemaMismatch));
            }
            (_, None, None) => {}
        }
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
        if let Some(writer) = self.logical_index_writer {
            writer.finish()?;
        }
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

    use arrow_array::{Float64Array, Int64Array, RecordBatch, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};
    use tempfile::TempDir;

    use super::WriteSession;
    use crate::dataset::{
        ingest::IngestError,
        schema::DatasetError,
        semantics::{RECORD_ID_COLUMN, ScanAxis, ScanAxisValue, ScanPlan},
        storage::{
            ChunkReader,
            layout::ChunkKind,
            logical_index::{logical_index_schema, logical_index_values_schema},
        },
    };

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

    fn logical_batch(scan_plan: &ScanPlan, gate: Vec<i64>, bias: Vec<i64>) -> RecordBatch {
        RecordBatch::try_new(
            logical_index_values_schema(scan_plan),
            vec![
                Arc::new(Int64Array::from(gate)),
                Arc::new(Int64Array::from(bias)),
            ],
        )
        .expect("logical batch")
    }

    fn scan_plan() -> ScanPlan {
        ScanPlan::new(vec![
            ScanAxis::static_values("gate", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
            ScanAxis::static_values("bias", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
        ])
    }

    #[test]
    fn write_session_materializes_monotonic_record_ids() {
        let dir = TempDir::new().expect("temp dir");
        let mut session = WriteSession::new(&user_schema(), dir.path().to_owned(), None, false);

        session
            .write(&batch(vec![10.0, 20.0]), None)
            .expect("first write");
        session
            .write(&batch(vec![30.0]), None)
            .expect("second write");
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
        let mut session = WriteSession::new(&user_schema(), dir.path().to_owned(), None, false);

        session
            .write(&batch(Vec::new()), None)
            .expect("empty write");
        session
            .write(&batch(vec![10.0]), None)
            .expect("second write");
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

    #[test]
    fn write_session_rejects_same_type_schema_mismatch_before_materializing_ids() {
        let dir = TempDir::new().expect("temp dir");
        let mut session = WriteSession::new(&user_schema(), dir.path().to_owned(), None, false);
        let mismatched_schema = Arc::new(Schema::new(vec![Field::new(
            "renamed_signal",
            DataType::Float64,
            false,
        )]));
        let mismatched_batch = RecordBatch::try_new(
            mismatched_schema,
            vec![Arc::new(Float64Array::from(vec![99.0]))],
        )
        .expect("mismatched batch");

        session
            .write(&batch(vec![10.0]), None)
            .expect("first write");
        let error = session
            .write(&mismatched_batch, None)
            .expect_err("schema mismatch should fail");
        session
            .write(&batch(vec![20.0]), None)
            .expect("second write");
        session.finish().expect("finish");

        assert!(matches!(
            error,
            IngestError::Dataset(DatasetError::SchemaMismatch)
        ));
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

        assert_eq!(record_ids.values(), &[0, 1]);
    }

    #[test]
    fn write_session_persists_logical_index_sidecar_with_record_ids() {
        let dir = TempDir::new().expect("temp dir");
        let scan_plan = scan_plan();
        let mut session = WriteSession::new(
            &user_schema(),
            dir.path().to_owned(),
            Some(scan_plan.clone()),
            true,
        );

        session
            .write(
                &batch(vec![10.0, 20.0]),
                Some(&logical_batch(&scan_plan, vec![1, 0], vec![0, 1])),
            )
            .expect("write with logical indices");
        session.finish().expect("finish");

        let mut reader = ChunkReader::new_with_kind(
            dir.path().to_owned(),
            Some(logical_index_schema(&scan_plan)),
            ChunkKind::LogicalIndex,
        );
        reader.read_all().expect("read sidecar chunks");
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
        let gate = stored
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("gate indices");
        let bias = stored
            .column(2)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("bias indices");

        assert_eq!(record_ids.values(), &[0, 1]);
        assert_eq!(gate.values(), &[1, 0]);
        assert_eq!(bias.values(), &[0, 1]);
    }

    #[test]
    fn write_session_rejects_logical_index_row_count_mismatch() {
        let dir = TempDir::new().expect("temp dir");
        let scan_plan = scan_plan();
        let mut session = WriteSession::new(
            &user_schema(),
            dir.path().to_owned(),
            Some(scan_plan.clone()),
            true,
        );

        let error = session
            .write(
                &batch(vec![10.0, 20.0]),
                Some(&logical_batch(&scan_plan, vec![1], vec![0])),
            )
            .expect_err("mismatched logical indices should fail");

        assert!(matches!(
            error,
            IngestError::Dataset(DatasetError::SchemaMismatch)
        ));
    }

    #[test]
    fn write_session_rejects_out_of_range_static_logical_index() {
        let dir = TempDir::new().expect("temp dir");
        let scan_plan = scan_plan();
        let mut session = WriteSession::new(
            &user_schema(),
            dir.path().to_owned(),
            Some(scan_plan.clone()),
            true,
        );

        let error = session
            .write(
                &batch(vec![10.0]),
                Some(&logical_batch(&scan_plan, vec![2], vec![0])),
            )
            .expect_err("out-of-range logical index should fail");

        assert!(matches!(
            error,
            IngestError::Dataset(DatasetError::InvalidFilter)
        ));
    }
}

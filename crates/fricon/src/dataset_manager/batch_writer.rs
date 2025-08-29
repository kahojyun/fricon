use std::io::Write;

use arrow::{array::RecordBatch, datatypes::Schema, error::ArrowError, ipc::writer::FileWriter};
use tracing::error;

pub struct BatchWriter<W: Write> {
    inner: FileWriter<W>,
    buffer: Vec<RecordBatch>,
    mem_count: usize,
    finished: bool,
}

impl<W: Write> BatchWriter<W> {
    const MEM_THRESHOLD: usize = 32 * 1024 * 1024;

    pub fn new(writer: W, schema: &Schema) -> Result<Self, ArrowError> {
        let inner = FileWriter::try_new(writer, schema)?;
        Ok(Self {
            inner,
            buffer: vec![],
            mem_count: 0,
            finished: false,
        })
    }

    pub fn write(&mut self, batch: RecordBatch) -> Result<(), ArrowError> {
        if batch.schema() != *self.inner.schema() {
            return Err(ArrowError::SchemaError("Schema mismatch".to_string()));
        }
        if batch.num_rows() == 0 {
            return Ok(());
        }
        self.mem_count += batch.get_array_memory_size();
        self.buffer.push(batch);
        if self.mem_count > Self::MEM_THRESHOLD {
            self.flush()?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), ArrowError> {
        self.finish_inner()
    }

    fn finish_inner(&mut self) -> Result<(), ArrowError> {
        if !self.finished {
            self.flush()?;
            self.inner.finish()?;
            self.finished = true;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), ArrowError> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let batches = arrow::compute::concat_batches(self.inner.schema(), self.buffer.iter())?;
        self.buffer.clear();
        self.mem_count = 0;
        self.inner.write(&batches)
    }
}

impl<W: Write> Drop for BatchWriter<W> {
    fn drop(&mut self) {
        if let Err(e) = self.finish_inner() {
            error!("Failed to finish arrow file writing: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, sync::Arc};

    use arrow::{
        array::{Int32Array, StringArray},
        datatypes::{DataType, Field, SchemaRef},
        error::Result as ArrowResult,
        ipc::reader::FileReader,
    };

    use super::*;

    fn create_test_schema() -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]))
    }

    fn create_test_batch(schema: &Schema, start_id: i32, num_rows: i32) -> RecordBatch {
        let id_array = Int32Array::from_iter_values(start_id..(start_id + num_rows));
        let name_array =
            StringArray::from_iter_values((0..num_rows).map(|i| format!("name_{}", start_id + i)));

        RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![Arc::new(id_array) as _, Arc::new(name_array) as _],
        )
        .unwrap()
    }

    #[test]
    fn batch_writer_basic_flow() -> ArrowResult<()> {
        let schema = create_test_schema();
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = BatchWriter::new(&mut buffer, &schema)?;

        let batch1 = create_test_batch(&schema, 0, 10);
        let batch2 = create_test_batch(&schema, 10, 5);

        writer.write(batch1.clone())?;
        writer.write(batch2.clone())?;
        writer.finish()?;

        let written_data = buffer.into_inner();
        let reader = FileReader::try_new(Cursor::new(written_data), None)?;

        let mut read_batches = Vec::new();
        for batch in reader {
            read_batches.push(batch?);
        }

        assert_eq!(read_batches.len(), 1);
        let combined_batch = arrow::compute::concat_batches(&schema, vec![&batch1, &batch2])?;
        assert_eq!(read_batches[0], combined_batch);

        Ok(())
    }

    #[test]
    fn batch_writer_schema_mismatch() -> ArrowResult<()> {
        let schema1 = create_test_schema();
        let schema2 = Schema::new(vec![Field::new("other_id", DataType::Int64, false)]);

        let mut buffer = Cursor::new(Vec::new());
        let mut writer = BatchWriter::new(&mut buffer, &schema1)?;

        let batch_ok = create_test_batch(&schema1, 0, 1);
        let batch_bad = RecordBatch::try_new(
            Arc::new(schema2),
            vec![Arc::new(arrow::array::Int64Array::from_iter_values([1])) as _],
        )
        .unwrap();

        writer.write(batch_ok)?;
        let result = writer.write(batch_bad);

        assert!(result.is_err());
        match result.unwrap_err() {
            ArrowError::SchemaError(msg) => assert_eq!(msg, "Schema mismatch"),
            e => panic!("Unexpected error type: {e:?}"),
        }

        Ok(())
    }

    #[test]
    fn batch_writer_drop_finishes() -> ArrowResult<()> {
        let schema = create_test_schema();
        let mut buffer = Cursor::new(Vec::new());

        {
            let mut writer = BatchWriter::new(&mut buffer, &schema)?;
            writer.write(create_test_batch(&schema, 0, 10))?;
            writer.write(create_test_batch(&schema, 10, 5))?;
        }

        let written_data = buffer.into_inner();
        let reader = FileReader::try_new(Cursor::new(written_data), None)?;

        let mut read_batches = Vec::new();
        for batch in reader {
            read_batches.push(batch?);
        }

        assert_eq!(read_batches.len(), 1);
        let expected_batch = arrow::compute::concat_batches(
            &schema,
            vec![
                &create_test_batch(&schema, 0, 10),
                &create_test_batch(&schema, 10, 5),
            ],
        )?;
        assert_eq!(read_batches[0], expected_batch);

        Ok(())
    }

    #[test]
    fn batch_writer_empty_finish() -> ArrowResult<()> {
        let schema = create_test_schema();
        let mut buffer = Cursor::new(Vec::new());
        let writer = BatchWriter::new(&mut buffer, &schema)?;

        writer.finish()?;

        let written_data = buffer.into_inner();
        let reader = FileReader::try_new(Cursor::new(written_data), None)?;

        let mut read_batches = Vec::new();
        for batch in reader {
            read_batches.push(batch?);
        }

        assert!(read_batches.is_empty());

        Ok(())
    }
}

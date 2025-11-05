mod in_progress;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::{
    DatasetManagerError, dataset_fs::ChunkWriter, write_session::in_progress::InProgressTable,
};

pub struct WriteSession {
    writer: ChunkWriter,
    in_progress_table: Arc<Mutex<InProgressTable>>,
}

impl WriteSession {
    pub fn new(schema: SchemaRef, dir_path: PathBuf) -> Self {
        let writer = ChunkWriter::new(schema.clone(), dir_path.clone());
        let in_progress_table = InProgressTable::new(schema, dir_path);
        let in_progress_table = Arc::new(Mutex::new(in_progress_table));
        Self {
            writer,
            in_progress_table,
        }
    }

    pub fn write(&mut self, batch: RecordBatch) -> Result<(), DatasetManagerError> {
        self.in_progress_table_mut().push(batch.clone())?;
        if self.writer.write(batch)? {
            self.in_progress_table_mut().continue_read_chunks()?;
        }
        Ok(())
    }

    pub fn handle(&self) -> WriteSessionHandle {
        WriteSessionHandle(self.in_progress_table.clone())
    }

    pub fn finish(self) -> Result<(), DatasetManagerError> {
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
pub struct WriteSessionHandle(Arc<Mutex<InProgressTable>>);

impl WriteSessionHandle {
    pub fn live(&self) -> &Arc<Mutex<InProgressTable>> {
        &self.0
    }
}

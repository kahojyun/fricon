use std::path::PathBuf;

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::{DatasetManagerError, dataset::ChunkedTable, dataset_fs::ChunkReader};

#[derive(Debug)]
pub struct InProgressTable {
    in_memory: ChunkedTable,
    reader: ChunkReader,
}

impl InProgressTable {
    pub fn new(schema: SchemaRef, dir_path: PathBuf) -> Self {
        Self {
            in_memory: ChunkedTable::new(schema.clone()),
            reader: ChunkReader::new(dir_path, Some(schema)),
        }
    }

    pub fn push(&mut self, batch: RecordBatch) -> Result<(), DatasetManagerError> {
        self.in_memory.push_back(batch)?;
        Ok(())
    }

    pub fn continue_read_chunks(&mut self) -> Result<(), DatasetManagerError> {
        self.reader.read_all()?;
        self.in_memory.release_front(self.reader.num_rows());
        Ok(())
    }
}

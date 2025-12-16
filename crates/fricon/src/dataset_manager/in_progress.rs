use std::{borrow::Cow, collections::Bound, ops::RangeBounds, path::PathBuf};

use arrow_array::RecordBatch;
use arrow_schema::SchemaRef;
use tokio::sync::watch;

use crate::{dataset::ChunkedTable, dataset_fs::ChunkReader, dataset_manager::Error};

#[derive(Debug)]
pub struct WriteProgress {
    pub row_count: usize,
}

#[derive(Debug)]
pub struct InProgressTable {
    in_memory: ChunkedTable,
    reader: ChunkReader,
    sender: watch::Sender<WriteProgress>,
}

impl InProgressTable {
    pub fn new(schema: SchemaRef, dir_path: PathBuf) -> Self {
        Self {
            in_memory: ChunkedTable::new(schema.clone()),
            reader: ChunkReader::new(dir_path, Some(schema)),
            sender: watch::Sender::new(WriteProgress { row_count: 0 }),
        }
    }

    pub fn schema(&self) -> &SchemaRef {
        self.in_memory.schema()
    }

    pub fn subscribe(&self) -> watch::Receiver<WriteProgress> {
        self.sender.subscribe()
    }

    pub fn push(&mut self, batch: RecordBatch) -> Result<(), Error> {
        self.in_memory.push_back(batch)?;
        self.sender.send_replace(WriteProgress {
            row_count: self.num_rows(),
        });
        Ok(())
    }

    pub fn continue_read_chunks(&mut self) -> Result<(), Error> {
        self.reader.read_all()?;
        self.in_memory.release_front(self.reader.num_rows());
        Ok(())
    }

    pub fn num_rows(&self) -> usize {
        self.in_memory.last_offset()
    }

    pub fn range<R>(&self, range: R) -> impl Iterator<Item = Cow<'_, RecordBatch>>
    where
        R: RangeBounds<usize>,
    {
        self.range_impl(range.start_bound().cloned(), range.end_bound().cloned())
    }

    fn range_impl(
        &self,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> impl Iterator<Item = Cow<'_, RecordBatch>> {
        let mid = self.in_memory.first_offset();
        self.reader
            .range((start, Bound::Excluded(mid)))
            .chain(self.in_memory.range((Bound::Included(mid), end)))
    }
}

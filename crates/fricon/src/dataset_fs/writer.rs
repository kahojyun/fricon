use std::{
    fs::File,
    io::{BufWriter, Seek},
    path::{Path, PathBuf},
};

use arrow_array::RecordBatch;
use arrow_ipc::writer::FileWriter;
use arrow_schema::{Schema, SchemaRef};
use arrow_select::concat::concat_batches;
use tracing::{error, warn};

use crate::dataset_fs::{Error, chunk_path};

const MAX_BATCH_BYTE_SIZE: usize = 64 * 1024 * 1024;
const MAX_CHUNK_BYTE_SIZE: u64 = 256 * 1024 * 1024;

pub struct ChunkWriter {
    dir_path: PathBuf,
    schema: SchemaRef,
    next_chunk_index: usize,
    current_writer: Option<InnerWriter>,
}

impl ChunkWriter {
    pub fn new(schema: SchemaRef, dir_path: PathBuf) -> Self {
        Self {
            dir_path,
            schema,
            next_chunk_index: 0,
            current_writer: None,
        }
    }

    /// Write a [`RecordBatch`], return true if current chunk file is completed.
    pub fn write(&mut self, batch: RecordBatch) -> Result<bool, Error> {
        let writer = self.current_writer()?;
        writer.write(batch)?;
        if writer.written_size >= MAX_CHUNK_BYTE_SIZE {
            self.finish_current_writer()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.finish_current_writer()
    }

    fn current_writer(&mut self) -> Result<&mut InnerWriter, Error> {
        if self.current_writer.is_none() {
            self.current_writer = Some(self.create_writer()?);
        }
        Ok(self.current_writer.as_mut().expect("Not none here."))
    }

    fn create_writer(&mut self) -> Result<InnerWriter, Error> {
        let writer = InnerWriter::new(&self.dir_path, self.next_chunk_index, &self.schema)?;
        self.next_chunk_index += 1;
        Ok(writer)
    }

    fn finish_current_writer(&mut self) -> Result<(), Error> {
        if let Some(writer) = self.current_writer.take() {
            writer.finish()?;
        }
        Ok(())
    }
}

impl Drop for ChunkWriter {
    fn drop(&mut self) {
        if self.current_writer.is_some() {
            warn!("ChunkWriter dropped with an active writer.");
            if let Err(e) = self.finish_current_writer() {
                error!("Error dropping ChunkWriter: {e}");
            }
        }
    }
}

struct InnerWriter {
    inner: FileWriter<BufWriter<File>>,
    buffered_batches: Vec<RecordBatch>,
    buffered_size: usize,
    written_size: u64,
}

impl InnerWriter {
    fn new(dir_path: &Path, chunk_index: usize, schema: &Schema) -> Result<InnerWriter, Error> {
        let chunk_path = chunk_path(dir_path, chunk_index);
        let file = File::create(chunk_path)?;
        let writer = FileWriter::try_new(BufWriter::new(file), schema)?;
        Ok(InnerWriter {
            inner: writer,
            buffered_batches: vec![],
            buffered_size: 0,
            written_size: 0,
        })
    }

    fn write(&mut self, batch: RecordBatch) -> Result<(), Error> {
        self.push_to_buffer(batch)?;
        if self.buffered_size >= MAX_BATCH_BYTE_SIZE {
            self.flush()?;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(), Error> {
        self.flush()?;
        self.inner.finish()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        if !self.buffered_batches.is_empty() {
            let batch = self.drain_buffer()?;
            self.inner.write(&batch)?;
            self.written_size = self.inner.get_mut().stream_position()?;
        }
        Ok(())
    }

    fn push_to_buffer(&mut self, batch: RecordBatch) -> Result<(), Error> {
        if batch.schema() == *self.inner.schema() {
            self.buffered_size += batch.get_array_memory_size();
            self.buffered_batches.push(batch);
            Ok(())
        } else {
            Err(Error::SchemaMismatch)
        }
    }

    fn drain_buffer(&mut self) -> Result<RecordBatch, Error> {
        let group = concat_batches(self.inner.schema(), &self.buffered_batches)?;
        self.buffered_batches.clear();
        self.buffered_size = 0;
        Ok(group)
    }
}

use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{ensure, Context, Result};
use arrow::{array::RecordBatch, datatypes::Schema, ipc::writer::FileWriter};
use tracing::info;

pub struct Writer {
    inner: FileWriter<BufWriter<File>>,
    buffer: Vec<RecordBatch>,
    mem_count: usize,
}

impl Writer {
    const MEM_THRESHOLD: usize = 32 * 1024 * 1024;
    pub fn new(path: &Path, schema: &Schema) -> Result<Self> {
        let filename = path.join("dataset.arrow");
        info!("Create dataset at {:?}", filename);
        let file = File::create_new(filename).context("Failed to create new dataset file.")?;
        let inner = FileWriter::try_new_buffered(file, schema)
            .context("Failed to create arrow ipc file writer")?;
        Ok(Self {
            inner,
            buffer: vec![],
            mem_count: 0,
        })
    }

    pub fn write(&mut self, batch: RecordBatch) -> Result<()> {
        ensure!(&batch.schema() == self.inner.schema());
        batch.get_array_memory_size();
        self.mem_count += batch.get_array_memory_size();
        self.buffer.push(batch);
        if self.mem_count > Self::MEM_THRESHOLD {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let batches = arrow::compute::concat_batches(self.inner.schema(), self.buffer.iter())
            .expect("Should be ensured that all batches have the same schema.");
        self.buffer.clear();
        self.mem_count = 0;
        self.inner
            .write(&batches)
            .context("Failed to write record batch to dataset file.")
    }

    pub fn finish(mut self) -> Result<()> {
        self.flush()?;
        self.inner
            .finish()
            .context("Failed to finish dataset writing.")
    }
}

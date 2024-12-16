use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use anyhow::{ensure, Context, Result};
use arrow::{
    array::RecordBatch,
    datatypes::Schema,
    ipc::writer::{FileWriter, IpcWriteOptions},
};
use parquet::{
    arrow::ArrowWriter,
    basic::{Compression, ZstdLevel},
    file::properties::WriterProperties,
};
use tracing::info;

pub struct Writer {
    inner: FileWriter<BufWriter<File>>,
    buffer: Vec<RecordBatch>,
    mem_count: usize,
}

impl Writer {
    // 64 MiB
    const MEM_THRESHOLD: usize = 1 << 26;
    pub fn new(path: &Path, schema: &Schema) -> Result<Self> {
        let filename = path.join("dataset.arrow");
        info!("Create dataset at {:?}", filename);
        let file = File::create_new(filename).context("Failed to create new dataset file.")?;
        let options = IpcWriteOptions::default()
            .try_with_compression(Some(arrow::ipc::CompressionType::ZSTD))
            .unwrap();
        let inner = FileWriter::try_new_with_options(BufWriter::new(file), schema, options)
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
        // self.inner
        //     .write(batch)
        //     .context("Failed to write record batch.")
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let batches = arrow::compute::concat_batches(self.inner.schema(), self.buffer.iter())?;
        self.inner.write(&batches)?;
        self.buffer.clear();
        self.mem_count = 0;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.flush()?;
        self.inner
            .finish()
            .context("Failed to finish dataset writing.")
    }
}

pub struct ParquetWriter {
    inner: ArrowWriter<File>,
}

impl ParquetWriter {
    pub fn new(path: &Path, schema: Arc<Schema>) -> Result<Self> {
        let filename = path.join("dataset.parquet");
        info!("Create dataset at {:?}", filename);
        let file = File::create_new(filename).context("Failed to create new dataset file.")?;
        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(
                ZstdLevel::try_new(3).expect("Should between 1 and 22."),
            ))
            .build();
        let inner = ArrowWriter::try_new(file, schema, Some(props))
            .context("Failed to create parquet file writer")?;
        Ok(Self { inner })
    }

    pub fn write(&mut self, batch: &RecordBatch) -> Result<()> {
        self.inner
            .write(batch)
            .context("Failed to write record batch.")
    }

    pub fn finish(self) -> Result<()> {
        self.inner
            .close()
            .context("Failed to finish dataset writing.")?;
        Ok(())
    }
}

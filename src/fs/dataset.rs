//! Manage dataset.
//!
//! A dataset is a folder containing a single [arrow] file and a JSON file for metadata. The
//! metadata can be updated, but the arrow file can be written only once.
use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use anyhow::{ensure, Context, Result};
use arrow::{array::RecordBatch, datatypes::Schema, ipc::writer::FileWriter};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

const DATASET_NAME: &str = "dataset.arrow";
const METADATA_NAME: &str = "metadata.json";

pub fn create_new(path: &Path, metadata: &Metadata, schema: &Schema) -> Result<Writer> {
    ensure!(
        !path.exists(),
        "Cannot create new dataset at already existing path {:?}",
        path
    );
    info!("Create dataset at {:?}", path);
    fs::create_dir_all(path).with_context(|| format!("Failed to create dataset at {path:?}"))?;
    let metadata_path = path.join(METADATA_NAME);
    metadata.write_to(&metadata_path)?;
    let dataset_path = path.join(DATASET_NAME);
    let dataset_file = File::create_new(&dataset_path)
        .with_context(|| format!("Failed to create new dataset file at {dataset_path:?}"))?;
    Writer::new(dataset_file, schema)
}

#[expect(dead_code)]
pub fn metadata(path: &Path) -> Result<Metadata> {
    let metadata_path = path.join(METADATA_NAME);
    let metadata_file = File::open(&metadata_path)
        .with_context(|| format!("Failed to open metadata file at {metadata_path:?}"))?;
    let metadata = serde_json::from_reader(metadata_file)
        .with_context(|| format!("Failed to deserialize metadata file at {metadata_path:?}"))?;
    Ok(metadata)
}

#[expect(dead_code)]
pub fn update_info(path: &Path, info: &Info) -> Result<()> {
    let metadata_path = path.join(METADATA_NAME);
    let metadata_file = File::create(&metadata_path).with_context(|| {
        format!("Failed to create or update metadata file at {metadata_path:?}")
    })?;
    serde_json::to_writer(metadata_file, info)
        .with_context(|| format!("Failed to serialize metadata file at {metadata_path:?}"))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub uid: Uuid,
    pub info: Info,
}

impl Metadata {
    fn write_to(&self, path: &Path) -> Result<()> {
        let metadata_file = File::create(path)
            .with_context(|| format!("Failed to create metadata file at {path:?}"))?;
        serde_json::to_writer(metadata_file, self)
            .with_context(|| format!("Failed to serialize metadata file at {path:?}"))
    }

    #[expect(dead_code)]
    fn read_from(path: &Path) -> Result<Self> {
        let metadata_file = File::open(path)
            .with_context(|| format!("Failed to open metadata file at {path:?}"))?;
        serde_json::from_reader(metadata_file)
            .with_context(|| format!("Failed to deserialize metadata file at {path:?}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

pub struct Writer {
    inner: FileWriter<BufWriter<File>>,
    buffer: Vec<RecordBatch>,
    mem_count: usize,
}

impl Writer {
    const MEM_THRESHOLD: usize = 32 * 1024 * 1024;
    fn new(file: File, schema: &Schema) -> Result<Self> {
        let inner = FileWriter::try_new_buffered(file, schema)
            .context("Failed to create arrow ipc file writer")?;
        Ok(Self {
            inner,
            buffer: vec![],
            mem_count: 0,
        })
    }

    pub fn write(&mut self, batch: RecordBatch) -> Result<()> {
        ensure!(
            &batch.schema() == self.inner.schema(),
            "Record batch schema mismatch."
        );
        batch.get_array_memory_size();
        self.mem_count += batch.get_array_memory_size();
        self.buffer.push(batch);
        if self.mem_count > Self::MEM_THRESHOLD {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
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

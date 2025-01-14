//! Manage dataset.
//!
//! A dataset is a folder containing a single [arrow] file and a JSON file for metadata. The
//! metadata can be updated, but the arrow file can be written only once.
use std::{
    fs::{self, File},
    io::BufWriter,
};

use anyhow::{ensure, Context, Result};
use arrow::{array::RecordBatch, datatypes::Schema, ipc::writer::FileWriter};
use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::{paths::DatasetPath, workspace::Workspace};

pub const DATASET_NAME: &str = "dataset.arrow";

// TODO: check dead code
pub struct Dataset {
    _workspace: Workspace,
    id: i64,
    _info: Info,
}

impl Dataset {
    pub fn create(workspace: Workspace, id: i64, info: Info, schema: &Schema) -> Result<Writer> {
        let path = workspace.root().data_dir().join(&info.path);
        ensure!(
            !path.exists(),
            "Cannot create new dataset at already existing path {:?}",
            path
        );
        info!("Create dataset at {:?}", path);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create dataset at {path:?}"))?;
        let dataset_path = path.join(DATASET_NAME);
        let dataset_file = File::create_new(&dataset_path)
            .with_context(|| format!("Failed to create new dataset file at {dataset_path:?}"))?;
        Writer::new(
            dataset_file,
            schema,
            Self {
                _workspace: workspace,
                id,
                _info: info,
            },
        )
    }

    pub const fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub index_columns: Vec<String>,
    pub path: DatasetPath,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub struct Writer {
    inner: FileWriter<BufWriter<File>>,
    buffer: Vec<RecordBatch>,
    mem_count: usize,
    dataset: Dataset,
}

impl Writer {
    const MEM_THRESHOLD: usize = 32 * 1024 * 1024;
    fn new(file: File, schema: &Schema, dataset: Dataset) -> Result<Self> {
        let inner = FileWriter::try_new_buffered(file, schema)
            .context("Failed to create arrow ipc file writer")?;
        Ok(Self {
            inner,
            buffer: vec![],
            mem_count: 0,
            dataset,
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

    pub fn finish(mut self) -> Result<Dataset> {
        self.flush()?;
        self.inner
            .finish()
            .context("Failed to finish dataset writing.")?;
        Ok(self.dataset)
    }
}

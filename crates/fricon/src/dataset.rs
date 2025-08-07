//! Working with datasets.
//!
//! A dataset is a folder containing a single [arrow] file and a JSON file for
//! metadata. The metadata can be updated, but the arrow file can be written
//! only once.
mod batch_writer;

use std::{
    fs::{self, File},
    io::BufWriter,
    mem,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use arrow::{array::RecordBatch, error::ArrowError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::workspace::Workspace;

use self::batch_writer::BatchWriter;

pub(crate) const DATASET_NAME: &str = "dataset.arrow";
pub(crate) const METADATA_NAME: &str = "metadata.json";

// TODO: check dead code
pub struct Dataset {
    _workspace: Workspace,
    id: i64,
    _metadata: Metadata,
    path: PathBuf,
}

impl Dataset {
    pub fn create(
        path: impl Into<PathBuf>,
        metadata: Metadata,
        workspace: Workspace,
        id: i64,
    ) -> Result<Writer> {
        let path = path.into();
        ensure!(
            !path.exists(),
            "Cannot create new dataset at already existing path {:?}",
            path
        );
        info!("Create dataset at {:?}", path);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create dataset at {}", path.display()))?;
        let dataset = Self {
            _workspace: workspace,
            id,
            _metadata: metadata,
            path,
        };
        let dataset_path = dataset.arrow_file();
        let dataset_file = File::create_new(&dataset_path).with_context(|| {
            format!(
                "Failed to create new dataset file at {}",
                dataset_path.display()
            )
        })?;
        Ok(Writer::new(dataset_file, dataset))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn arrow_file(&self) -> PathBuf {
        self.path.join(DATASET_NAME)
    }

    #[must_use]
    pub fn metadata_file(&self) -> PathBuf {
        self.path.join(METADATA_NAME)
    }

    #[must_use]
    pub const fn id(&self) -> Option<i64> {
        Some(self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub index_columns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub struct Writer {
    inner: WriterState,
    dataset: Dataset,
}

impl Writer {
    fn new(file: File, dataset: Dataset) -> Self {
        Self {
            inner: WriterState::NotStarted(file),
            dataset,
        }
    }

    pub fn write(&mut self, batch: RecordBatch) -> Result<()> {
        match mem::replace(&mut self.inner, WriterState::Failed) {
            WriterState::NotStarted(file) => {
                let mut writer = BatchWriter::new(BufWriter::new(file), &batch.schema())?;
                writer.write(batch)?;
                self.inner = WriterState::InProgress(writer);
            }
            WriterState::InProgress(mut writer) => {
                let result = writer.write(batch);
                if matches!(result, Err(ArrowError::SchemaError(_))) {
                    // Allow recovery from schema errors
                    self.inner = WriterState::InProgress(writer);
                }
                result?;
            }
            WriterState::Failed => {
                bail!("Writer is in a failed state.");
            }
        }
        Ok(())
    }

    pub fn finish(self) -> Result<Dataset> {
        match self.inner {
            WriterState::InProgress(writer) => {
                writer.finish()?;
                Ok(self.dataset)
            }
            WriterState::NotStarted(_) => {
                bail!("No data written to the writer.");
            }
            WriterState::Failed => {
                bail!("Writer is in a failed state.");
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum WriterState {
    NotStarted(File),
    InProgress(BatchWriter<BufWriter<File>>),
    Failed,
}

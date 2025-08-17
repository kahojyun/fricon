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
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    database::{self, NewTag, Tag},
    paths::dataset_path_from_uuid,
};
use crate::{
    database::{DatasetTag, PoolExt},
    workspace::Workspace,
};

use self::batch_writer::BatchWriter;

pub(crate) const DATASET_NAME: &str = "dataset.arrow";
pub(crate) const METADATA_NAME: &str = "metadata.json";

pub struct Dataset {
    workspace: Workspace,
    db_row: database::Dataset,
    tags: Vec<database::Tag>,
    path: PathBuf,
}

impl Dataset {
    pub fn create(
        workspace: Workspace,
        dataset: database::Dataset,
        tags: Vec<database::Tag>,
    ) -> Result<Writer> {
        let dataset_path = dataset_path_from_uuid(dataset.uuid.0);
        let path = workspace.root().data_dir().join(dataset_path);
        ensure!(
            !path.exists(),
            "Cannot create new dataset at already existing path {:?}",
            path
        );
        info!("Create dataset at {:?}", path);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create dataset at {}", path.display()))?;
        let metadata = Metadata::from_database_models(&dataset, &tags);
        let dataset = Self {
            workspace,
            db_row: dataset,
            tags,
            path,
        };
        let dataset_path = dataset.arrow_file();
        let dataset_file = File::create_new(&dataset_path).with_context(|| {
            format!(
                "Failed to create new dataset file at {}",
                dataset_path.display()
            )
        })?;
        let metadata_path = dataset.metadata_file();
        metadata.save(&metadata_path)?;
        Ok(Writer::new(dataset_file, dataset))
    }

    pub(crate) fn new(
        workspace: Workspace,
        dataset: database::Dataset,
        tags: Vec<database::Tag>,
    ) -> Self {
        let dataset_path = dataset_path_from_uuid(dataset.uuid.0);
        let path = workspace.root().data_dir().join(dataset_path);
        Self {
            workspace,
            db_row: dataset,
            tags,
            path,
        }
    }

    pub async fn add_tags(&mut self, tags: Vec<String>) -> Result<()> {
        let dataset_id = self.db_row.id;
        let metadata_path = self.metadata_file();
        let mut metadata = self.metadata();
        self.tags = self
            .workspace
            .database()
            .interact(move |conn| {
                use database::schema;
                let tags = conn.immediate_transaction(|conn| {
                    // Insert or ignore new tags into the tags table
                    let new_tags: Vec<_> = tags.iter().map(|name| NewTag { name }).collect();
                    diesel::insert_or_ignore_into(schema::tags::table)
                        .values(new_tags)
                        .execute(conn)?;
                    // Get the IDs of the tags that were requested to be added
                    let tag_ids = schema::tags::table
                        .filter(schema::tags::name.eq_any(tags))
                        .select(schema::tags::id)
                        .load::<i32>(conn)?;
                    // Insert or ignore new entries into the datasets_tags table
                    let new_dataset_tags: Vec<_> = tag_ids
                        .into_iter()
                        .map(|tag_id| DatasetTag { dataset_id, tag_id })
                        .collect();
                    diesel::insert_or_ignore_into(schema::datasets_tags::table)
                        .values(new_dataset_tags)
                        .execute(conn)?;
                    // Reload all tags for this dataset from the database
                    let tags = schema::datasets_tags::table
                        .filter(schema::datasets_tags::dataset_id.eq(dataset_id))
                        .inner_join(schema::tags::table)
                        .select(Tag::as_select())
                        .load::<Tag>(conn)?;
                    // Update metadata and save it
                    metadata.tags = tags.iter().map(|tag| tag.name.clone()).collect();
                    metadata.save(&metadata_path)?;
                    anyhow::Ok(tags)
                })?;
                Ok(tags)
            })
            .await?;
        Ok(())
    }

    pub async fn remove_tags(&mut self, tags: Vec<String>) -> Result<()> {
        let dataset_id = self.db_row.id;
        let metadata_path = self.metadata_file();
        let mut metadata = self.metadata();
        self.tags = self
            .workspace
            .database()
            .interact(move |conn| {
                use database::schema;
                let tags_to_delete_names = tags;
                let tags = conn.immediate_transaction(|conn| {
                    // Get the IDs of the tags to be deleted
                    let tag_ids_to_delete = schema::tags::table
                        .filter(schema::tags::name.eq_any(tags_to_delete_names))
                        .select(schema::tags::id)
                        .load::<i32>(conn)?;

                    // Delete the entries from datasets_tags table
                    diesel::delete(schema::datasets_tags::table)
                        .filter(schema::datasets_tags::dataset_id.eq(dataset_id))
                        .filter(schema::datasets_tags::tag_id.eq_any(tag_ids_to_delete))
                        .execute(conn)?;

                    // Reload all tags for this dataset from the database
                    let updated_tags = schema::datasets_tags::table
                        .filter(schema::datasets_tags::dataset_id.eq(dataset_id))
                        .inner_join(schema::tags::table)
                        .select(Tag::as_select())
                        .load::<Tag>(conn)?;

                    // Update metadata and save it
                    metadata.tags = updated_tags.iter().map(|tag| tag.name.clone()).collect();
                    metadata.save(&metadata_path)?;
                    anyhow::Ok(updated_tags)
                })?;
                Ok(tags)
            })
            .await?;
        Ok(())
    }

    pub async fn update_info(&mut self, update: database::DatasetUpdate) -> Result<()> {
        let dataset_id = self.db_row.id;
        let metadata_path = self.metadata_file();
        let mut current_metadata = self.metadata();

        self.db_row = self
            .workspace
            .database()
            .interact(move |conn| {
                use database::schema::datasets::dsl::*;

                let updated_dataset = diesel::update(datasets.find(dataset_id))
                    .set(&update)
                    .get_result::<database::Dataset>(conn)
                    .context("Failed to update dataset in database")?;

                // Update the in-memory metadata struct and save it to file
                current_metadata.name = updated_dataset.name.clone();
                current_metadata.description = updated_dataset.description.clone();
                current_metadata.favorite = updated_dataset.favorite;

                current_metadata.save(&metadata_path)?;

                Ok(updated_dataset)
            })
            .await?;
        Ok(())
    }

    pub async fn delete(&mut self) -> Result<()> {
        let dataset_id = self.db_row.id;

        self.workspace
            .database()
            .interact(move |conn| {
                use database::schema::datasets::dsl::*;

                diesel::delete(datasets.find(dataset_id))
                    .execute(conn)
                    .context("Failed to delete dataset from database")?;

                Ok(())
            })
            .await?;

        fs::remove_dir_all(self.path()).context("Failed to delete dataset directory")?;

        Ok(())
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

    pub fn metadata(&self) -> Metadata {
        Metadata::from_database_models(&self.db_row, &self.tags)
    }

    #[must_use]
    pub const fn id(&self) -> i32 {
        self.db_row.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub index_columns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl Metadata {
    pub fn from_database_models(dataset: &database::Dataset, tags: &[database::Tag]) -> Self {
        let tag_names = tags.iter().map(|tag| tag.name.clone()).collect();
        Self {
            uuid: dataset.uuid.0,
            name: dataset.name.clone(),
            description: dataset.description.clone(),
            favorite: dataset.favorite,
            index_columns: dataset.index_columns.0.clone(),
            created_at: dataset.created_at.and_utc(),
            tags: tag_names,
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
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

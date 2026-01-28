//! Dataset Manager - Central hub for all dataset operations
//!
//! The `DatasetManager` centralizes all server-side dataset CRUD operations and
//! lifecycle management, providing a clean interface that abstracts database
//! operations and file system interactions.

mod in_progress;
mod tasks;
mod write_registry;
mod write_session;

use std::{
    borrow::Cow,
    cmp::Ordering,
    ops::{Bound, RangeBounds},
    path::PathBuf,
    sync::Arc,
};

use arrow_arith::boolean::and;
use arrow_array::{
    ArrayRef, BooleanArray, RecordBatch, RecordBatchOptions, RecordBatchReader, Scalar,
};
use arrow_ord::{cmp::eq, ord::make_comparator};
use arrow_schema::{Schema, SchemaRef, SortOptions};
use arrow_select::{concat::concat_batches, filter::FilterBuilder};
use chrono::{DateTime, Utc};
use derive_more::From;
use diesel::result::Error as DieselError;
use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;
use tracing::error;
use uuid::Uuid;

pub use self::write_registry::WriteSessionRegistry;
use crate::{
    DatasetDataType, DatasetSchema,
    app::{AppError, AppHandle, AppState},
    database::{self, DatabaseError, DatasetStatus},
    dataset, dataset_fs,
    dataset_fs::ChunkReader,
    dataset_manager::write_session::WriteSessionHandle,
};

fn emit_dataset_updated(state: &AppState, record: DatasetRecord) {
    let DatasetRecord { id, metadata } = record;
    let DatasetMetadata {
        name,
        description,
        favorite,
        tags,
        status,
        created_at,
        ..
    } = metadata;

    let _ = state.event_sender.send(crate::AppEvent::DatasetUpdated {
        id,
        name,
        description,
        favorite,
        tags,
        status,
        created_at,
    });
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Dataset not found: {id}")]
    NotFound { id: String },
    #[error("No dataset file found.")]
    EmptyDataset,
    #[error("Dataset write stream error: {message}")]
    BatchStream { message: String },
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Dataset(#[from] dataset::Error),
    #[error(transparent)]
    DatasetFs(#[from] dataset_fs::Error),
    #[error(transparent)]
    TaskJoin(#[from] JoinError),
    #[error(transparent)]
    App(#[from] AppError),
}

impl From<DieselError> for Error {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound {
                id: "unknown".to_string(),
            },
            other => Self::Database(other.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetRecord {
    pub id: i32,
    pub metadata: DatasetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub uid: Uuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
}

#[derive(Debug, Clone, Copy, From)]
pub enum DatasetId {
    Id(i32),
    Uid(Uuid),
}

#[derive(Clone)]
pub struct DatasetManager {
    app: AppHandle,
}

impl DatasetManager {
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn create_dataset<F, I>(
        &self,
        request: CreateDatasetRequest,
        reader: F,
    ) -> Result<DatasetRecord, Error>
    where
        F: FnOnce() -> Result<I, Error> + Send + 'static,
        I: RecordBatchReader,
    {
        self.app
            .spawn_blocking(move |state| {
                reader()
                    .and_then(|batches| {
                        tasks::do_create_dataset(
                            &state.database,
                            &state.root,
                            &state.event_sender,
                            &state.write_sessions,
                            request,
                            batches,
                        )
                    })
                    .inspect_err(|e| {
                        error!("Dataset creation failed: {e}");
                    })
            })?
            .await?
    }

    pub async fn get_dataset(&self, id: DatasetId) -> Result<DatasetRecord, Error> {
        self.app
            .spawn_blocking(move |state| tasks::do_get_dataset(&mut *state.database.get()?, id))?
            .await?
    }

    pub async fn list_datasets(
        &self,
        search: Option<&str>,
        tags: Option<&[String]>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DatasetRecord>, Error> {
        let search = search.map(str::to_string);
        let tags = tags.map(Vec::from);
        self.app
            .spawn_blocking(move |state| {
                tasks::do_list_datasets(
                    &mut *state.database.get()?,
                    search.as_deref(),
                    tags.as_deref(),
                    limit,
                    offset,
                )
            })?
            .await?
    }

    pub async fn update_dataset(&self, id: i32, update: DatasetUpdate) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                tasks::do_update_dataset(&mut conn, id, update)?;
                let record = tasks::do_get_dataset(&mut conn, DatasetId::Id(id))?;

                emit_dataset_updated(&state, record);

                Ok(())
            })?
            .await?
    }

    pub async fn add_tags(&self, id: i32, tags: Vec<String>) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                tasks::do_add_tags(&mut conn, id, &tags)?;
                let record = tasks::do_get_dataset(&mut conn, DatasetId::Id(id))?;

                emit_dataset_updated(&state, record);

                Ok(())
            })?
            .await?
    }

    pub async fn remove_tags(&self, id: i32, tags: Vec<String>) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                let mut conn = state.database.get()?;
                tasks::do_remove_tags(&mut conn, id, &tags)?;
                let record = tasks::do_get_dataset(&mut conn, DatasetId::Id(id))?;

                emit_dataset_updated(&state, record);

                Ok(())
            })?
            .await?
    }

    pub async fn delete_dataset(&self, id: i32) -> Result<(), Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_delete_dataset(&state.database, &state.root, id).inspect_err(|e| {
                    error!("Dataset deletion failed: {e}");
                })
            })?
            .await?
    }

    pub async fn get_dataset_reader(&self, id: DatasetId) -> Result<DatasetReader, Error> {
        self.app
            .spawn_blocking(move |state| {
                tasks::do_get_dataset_reader(
                    &state.database,
                    &state.root,
                    &state.write_sessions,
                    id,
                )
            })?
            .await?
    }
}

impl DatasetRecord {
    #[must_use]
    pub fn from_database_models(dataset: database::Dataset, tags: Vec<database::Tag>) -> Self {
        let metadata = DatasetMetadata {
            uid: dataset.uid.0,
            name: dataset.name,
            description: dataset.description,
            favorite: dataset.favorite,
            status: dataset.status,
            created_at: dataset.created_at.and_utc(),
            tags: tags.into_iter().map(|tag| tag.name).collect(),
        };

        Self {
            id: dataset.id,
            metadata,
        }
    }
}

enum DatasetSource {
    WriteSession(WriteSessionHandle),
    File(ChunkReader),
}

impl DatasetSource {
    fn is_complete(&self) -> bool {
        match self {
            DatasetSource::WriteSession(h) => h.is_complete(),
            DatasetSource::File(_) => true,
        }
    }
    fn write_status(&self) -> (usize, bool) {
        match self {
            DatasetSource::WriteSession(h) => h.snapshot_status(),
            DatasetSource::File(r) => (r.num_rows(), true),
        }
    }
    fn num_rows(&self) -> usize {
        match self {
            DatasetSource::WriteSession(h) => h.num_rows(),
            DatasetSource::File(r) => r.num_rows(),
        }
    }
    fn range<R>(&self, range: R) -> Vec<RecordBatch>
    where
        R: RangeBounds<usize> + Copy,
    {
        match self {
            DatasetSource::WriteSession(h) => h.snapshot_range(range),
            DatasetSource::File(r) => r.range(range).map(std::borrow::Cow::into_owned).collect(),
        }
    }
    fn select_data(&self, options: &SelectOptions) -> Result<(SchemaRef, Vec<RecordBatch>), Error> {
        let index_filters = options.index_filters.as_ref();
        let selected_columns = options.selected_columns.as_deref();
        let result = match self {
            DatasetSource::WriteSession(h) => {
                let (schema, batches) = h.snapshot_range_with_schema((options.start, options.end));
                select_data_owned(batches, &schema, index_filters, selected_columns)
            }
            DatasetSource::File(r) => select_data(
                r.range((options.start, options.end)),
                r.schema().ok_or(Error::EmptyDataset)?,
                index_filters,
                selected_columns,
            ),
        }?;
        Ok(result)
    }
}

pub struct DatasetReader {
    source: DatasetSource,
    schema: DatasetSchema,
    arrow_schema: SchemaRef,
}

pub struct SelectOptions {
    pub start: Bound<usize>,
    pub end: Bound<usize>,
    pub index_filters: Option<RecordBatch>,
    pub selected_columns: Option<Vec<usize>>,
}

#[derive(Debug, Default)]
struct Filter {
    filters: Vec<(usize, Scalar<ArrayRef>)>,
}

impl Filter {
    fn new(schema: &Schema, filters: &RecordBatch) -> Result<Self, dataset::Error> {
        if filters.schema().fields.is_empty() {
            Ok(Self { filters: vec![] })
        } else if filters.num_rows() != 1 {
            Err(dataset::Error::InvalidFilter)
        } else {
            let filters = filters
                .schema_ref()
                .fields
                .iter()
                .zip(filters.columns())
                .map(|(field, column)| {
                    let column_index = schema
                        .column_with_name(field.name())
                        .ok_or(dataset::Error::InvalidFilter)?
                        .0;
                    Ok::<_, dataset::Error>((column_index, Scalar::new(column.clone())))
                })
                .try_collect()?;
            Ok(Self { filters })
        }
    }

    fn build_predicate(&self, batch: &RecordBatch) -> Result<Option<BooleanArray>, dataset::Error> {
        Ok(self
            .filters
            .iter()
            .map(|(i, v)| eq(batch.column(*i), v))
            .reduce(|x, y| x.and_then(|x| y.and_then(|y| and(&x, &y))))
            .transpose()?)
    }
}

fn select_data<'a>(
    source: impl Iterator<Item = Cow<'a, RecordBatch>>,
    source_schema: &SchemaRef,
    index_filters: Option<&RecordBatch>,
    selected_columns: Option<&[usize]>,
) -> Result<(SchemaRef, Vec<RecordBatch>), dataset::Error> {
    let filter = if let Some(f) = index_filters {
        Filter::new(source_schema, f)?
    } else {
        Filter::default()
    };
    let (output_schema, selected_columns) = if let Some(c) = selected_columns {
        (
            Arc::new(source_schema.project(c)?),
            Either::Left(c.iter().copied()),
        )
    } else {
        (
            source_schema.clone(),
            Either::Right(0..source_schema.fields.len()),
        )
    };
    let results = source
        .map(|batch| -> Result<_, dataset::Error> {
            let mask = filter.build_predicate(&batch)?;
            let predicate = mask.map(|m| {
                let mut builder = FilterBuilder::new(&m);
                if output_schema.fields.len() > 1 {
                    builder = builder.optimize();
                }
                builder.build()
            });
            if let Some(p) = &predicate
                && p.count() == 0
            {
                Ok(None)
            } else {
                let arrays: Vec<_> = selected_columns
                    .clone()
                    .into_iter()
                    .map(|x| {
                        let array = batch.column(x);
                        if let Some(p) = &predicate {
                            p.filter(array).expect("Should have correct length")
                        } else {
                            array.clone()
                        }
                    })
                    .collect();
                let length = predicate.map_or_else(|| batch.num_rows(), |p| p.count());
                let output_batch = RecordBatch::try_new_with_options(
                    output_schema.clone(),
                    arrays,
                    &RecordBatchOptions::new().with_row_count(Some(length)),
                )?;
                Ok(Some(output_batch))
            }
        })
        .flatten_ok()
        .try_collect()?;
    Ok((output_schema, results))
}

fn select_data_owned(
    batches: Vec<RecordBatch>,
    source_schema: &SchemaRef,
    index_filters: Option<&RecordBatch>,
    selected_columns: Option<&[usize]>,
) -> Result<(SchemaRef, Vec<RecordBatch>), dataset::Error> {
    let source = batches.into_iter().map(Cow::Owned);
    select_data(source, source_schema, index_filters, selected_columns)
}

impl DatasetReader {
    fn from_handle(source: WriteSessionHandle) -> Result<Self, Error> {
        let arrow_schema = source.schema();
        let schema = arrow_schema.as_ref().try_into()?;
        Ok(Self {
            source: DatasetSource::WriteSession(source),
            schema,
            arrow_schema,
        })
    }
    fn open_dir(path: PathBuf) -> Result<Self, Error> {
        let mut reader = ChunkReader::new(path, None);
        reader.read_all()?;
        let arrow_schema = reader.schema().ok_or(Error::EmptyDataset)?.clone();
        let schema = arrow_schema.as_ref().try_into()?;
        Ok(Self {
            source: DatasetSource::File(reader),
            schema,
            arrow_schema,
        })
    }
    #[must_use]
    pub fn schema(&self) -> &DatasetSchema {
        &self.schema
    }
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.source.num_rows()
    }
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.source.is_complete()
    }
    #[must_use]
    pub fn write_status(&self) -> (usize, bool) {
        self.source.write_status()
    }
    #[must_use]
    pub fn arrow_schema(&self) -> &SchemaRef {
        &self.arrow_schema
    }
    #[must_use]
    pub fn batches(&self) -> Vec<RecordBatch> {
        self.source.range(..)
    }
    pub fn select_data(
        &self,
        options: &SelectOptions,
    ) -> Result<(SchemaRef, Vec<RecordBatch>), Error> {
        self.source.select_data(options)
    }
    #[must_use]
    pub fn index_columns(&self) -> Option<Vec<usize>> {
        if self.source.num_rows() < 2 {
            None
        } else {
            let sample = self.source.range(..2);
            let sample =
                concat_batches(&sample[0].schema(), &sample).expect("Should have same schema");
            let mut result = vec![];
            for (i, (sample_array, column_type)) in sample
                .columns()
                .iter()
                .zip(self.schema.columns().values())
                .enumerate()
            {
                if !matches!(column_type, DatasetDataType::Scalar(_)) {
                    break;
                }
                result.push(i);
                let cmp = make_comparator(sample_array, sample_array, SortOptions::default())
                    .expect("Should be self comparable");
                if cmp(0, 1) != Ordering::Equal {
                    break;
                }
            }
            Some(result)
        }
    }
}

use std::{borrow::Cow, cmp::Ordering, ops::RangeBounds, path::PathBuf, sync::Arc};

use arrow_arith::boolean::and;
use arrow_array::{ArrayRef, BooleanArray, RecordBatch, RecordBatchOptions, Scalar};
use arrow_ord::{cmp::eq, ord::make_comparator};
use arrow_schema::{Schema, SchemaRef, SortOptions};
use arrow_select::{concat::concat_batches, filter::FilterBuilder};
use itertools::{Either, Itertools};

use crate::dataset::{
    ingest::WriteSessionHandle,
    read::{ReadError, SelectOptions},
    schema::{DatasetDataType, DatasetError, DatasetSchema},
    storage::ChunkReader,
};

enum DatasetSource {
    WriteSession(WriteSessionHandle),
    File(ChunkReader),
}

impl DatasetSource {
    fn write_status(&self) -> usize {
        match self {
            Self::WriteSession(handle) => handle.num_rows(),
            Self::File(reader) => reader.num_rows(),
        }
    }

    fn num_rows(&self) -> usize {
        match self {
            Self::WriteSession(handle) => handle.num_rows(),
            Self::File(reader) => reader.num_rows(),
        }
    }

    fn range<R>(&self, range: R) -> Vec<RecordBatch>
    where
        R: RangeBounds<usize> + Copy,
    {
        match self {
            Self::WriteSession(handle) => handle.snapshot_range(range),
            Self::File(reader) => reader.range(range).map(Cow::into_owned).collect(),
        }
    }

    fn select_data(
        &self,
        options: &SelectOptions,
    ) -> Result<(SchemaRef, Vec<RecordBatch>), ReadError> {
        let index_filters = options.index_filters.as_ref();
        let selected_columns = options.selected_columns.as_deref();
        match self {
            Self::WriteSession(handle) => {
                let (schema, batches) =
                    handle.snapshot_range_with_schema((options.start, options.end));
                select_data_owned(batches, &schema, index_filters, selected_columns)
                    .map_err(Into::into)
            }
            Self::File(reader) => select_data(
                reader.range((options.start, options.end)),
                reader.schema().ok_or(ReadError::EmptyDataset)?,
                index_filters,
                selected_columns,
            )
            .map_err(Into::into),
        }
    }
}

pub struct DatasetReader {
    source: DatasetSource,
    schema: DatasetSchema,
    arrow_schema: SchemaRef,
}

#[derive(Debug, Default)]
struct Filter {
    filters: Vec<(usize, Scalar<ArrayRef>)>,
}

impl Filter {
    fn new(schema: &Schema, filters: &RecordBatch) -> Result<Self, DatasetError> {
        if filters.schema().fields.is_empty() {
            Ok(Self { filters: vec![] })
        } else if filters.num_rows() != 1 {
            Err(DatasetError::InvalidFilter)
        } else {
            let filters = filters
                .schema_ref()
                .fields
                .iter()
                .zip(filters.columns())
                .map(|(field, column)| {
                    let column_index = schema
                        .column_with_name(field.name())
                        .ok_or(DatasetError::InvalidFilter)?
                        .0;
                    Ok::<_, DatasetError>((column_index, Scalar::new(column.clone())))
                })
                .try_collect()?;
            Ok(Self { filters })
        }
    }

    fn build_predicate(&self, batch: &RecordBatch) -> Result<Option<BooleanArray>, DatasetError> {
        Ok(self
            .filters
            .iter()
            .map(|(index, value)| eq(batch.column(*index), value))
            .reduce(|left, right| left.and_then(|left| right.and_then(|right| and(&left, &right))))
            .transpose()?)
    }
}

fn select_data<'a>(
    source: impl Iterator<Item = Cow<'a, RecordBatch>>,
    source_schema: &SchemaRef,
    index_filters: Option<&RecordBatch>,
    selected_columns: Option<&[usize]>,
) -> Result<(SchemaRef, Vec<RecordBatch>), DatasetError> {
    let filter = if let Some(filters) = index_filters {
        Filter::new(source_schema, filters)?
    } else {
        Filter::default()
    };

    let (output_schema, selected_columns) = if let Some(columns) = selected_columns {
        (
            Arc::new(source_schema.project(columns)?),
            Either::Left(columns.iter().copied()),
        )
    } else {
        (
            source_schema.clone(),
            Either::Right(0..source_schema.fields.len()),
        )
    };

    let results = source
        .map(|batch| -> Result<_, DatasetError> {
            let mask = filter.build_predicate(&batch)?;
            let predicate = mask.map(|mask| {
                let mut builder = FilterBuilder::new(&mask);
                if output_schema.fields.len() > 1 {
                    builder = builder.optimize();
                }
                builder.build()
            });

            if let Some(predicate) = &predicate
                && predicate.count() == 0
            {
                Ok(None)
            } else {
                let arrays = selected_columns
                    .clone()
                    .into_iter()
                    .map(|column| {
                        let array = batch.column(column);
                        if let Some(predicate) = &predicate {
                            predicate.filter(array).expect("Should have correct length")
                        } else {
                            array.clone()
                        }
                    })
                    .collect();
                let length =
                    predicate.map_or_else(|| batch.num_rows(), |predicate| predicate.count());
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
) -> Result<(SchemaRef, Vec<RecordBatch>), DatasetError> {
    select_data(
        batches.into_iter().map(Cow::Owned),
        source_schema,
        index_filters,
        selected_columns,
    )
}

impl DatasetReader {
    pub(crate) fn from_handle(source: WriteSessionHandle) -> Result<Self, ReadError> {
        let arrow_schema = source.schema();
        let schema = arrow_schema.as_ref().try_into()?;
        Ok(Self {
            source: DatasetSource::WriteSession(source),
            schema,
            arrow_schema,
        })
    }

    pub(crate) fn open_dir(path: PathBuf) -> Result<Self, ReadError> {
        let mut reader = ChunkReader::new(path, None);
        reader.read_all()?;
        let arrow_schema = reader.schema().ok_or(ReadError::EmptyDataset)?.clone();
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
    pub fn write_status(&self) -> usize {
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
    ) -> Result<(SchemaRef, Vec<RecordBatch>), ReadError> {
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
            for (index, (sample_array, column_type)) in sample
                .columns()
                .iter()
                .zip(self.schema.columns().values())
                .enumerate()
            {
                if !matches!(column_type, DatasetDataType::Scalar(_)) {
                    break;
                }
                result.push(index);
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

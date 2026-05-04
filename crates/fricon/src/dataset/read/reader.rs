use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    ops::RangeBounds,
    path::{Path, PathBuf},
    sync::Arc,
};

use arrow_arith::boolean::and;
use arrow_array::{ArrayRef, BooleanArray, RecordBatch, RecordBatchOptions, Scalar, UInt64Array};
use arrow_ord::{cmp::eq, ord::make_comparator};
use arrow_schema::{Schema, SchemaRef, SortOptions};
use arrow_select::{concat::concat_batches, filter::FilterBuilder};
use itertools::Itertools;

use crate::dataset::{
    ingest::WriteSessionHandle,
    interpret::{
        DatasetInterpretation, ResolvedLogicalIndexPoint, resolve_from_compatibility_inference,
        resolve_from_manifest_with_compatibility_inference, resolve_logical_index_points,
    },
    read::{ReadError, SelectOptions},
    schema::{DatasetDataType, DatasetError, DatasetSchema},
    semantics::{
        DatasetSemanticManifest, IndexRealization, ManifestError, RECORD_ID_COLUMN, ScanAxisMode,
        ScanAxisValue, is_hidden_system_column, read_manifest_optional,
    },
    storage::{
        ChunkReader,
        error::DatasetFsError,
        logical_index::{
            logical_index_column, logical_index_record_ids, read_logical_index_batches,
        },
    },
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
        output_schema: SchemaRef,
        physical_columns: &[usize],
    ) -> Result<(SchemaRef, Vec<RecordBatch>), ReadError> {
        let index_filters = options.index_filters.as_ref();
        match self {
            Self::WriteSession(handle) => {
                let (schema, batches) =
                    handle.snapshot_range_with_schema((options.start, options.end));
                select_data_owned(
                    batches,
                    &schema,
                    index_filters,
                    output_schema,
                    physical_columns,
                )
                .map_err(Into::into)
            }
            Self::File(reader) => select_data(
                reader.range((options.start, options.end)),
                reader.schema().ok_or(ReadError::EmptyDataset)?,
                index_filters,
                output_schema,
                physical_columns,
            )
            .map_err(Into::into),
        }
    }
}

pub struct DatasetReader {
    source: DatasetSource,
    schema: Option<DatasetSchema>,
    physical_arrow_schema: SchemaRef,
    arrow_schema: SchemaRef,
    visible_columns: Vec<usize>,
    manifest: Option<DatasetSemanticManifest>,
    dataset_path: Option<PathBuf>,
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
    output_schema: SchemaRef,
    selected_columns: &[usize],
) -> Result<(SchemaRef, Vec<RecordBatch>), DatasetError> {
    let filter = if let Some(filters) = index_filters {
        Filter::new(source_schema, filters)?
    } else {
        Filter::default()
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
                    .iter()
                    .copied()
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
    output_schema: SchemaRef,
    selected_columns: &[usize],
) -> Result<(SchemaRef, Vec<RecordBatch>), DatasetError> {
    select_data(
        batches.into_iter().map(Cow::Owned),
        source_schema,
        index_filters,
        output_schema,
        selected_columns,
    )
}

fn visible_projection_from_manifest(
    physical_schema: &SchemaRef,
    manifest: Option<&DatasetSemanticManifest>,
) -> Result<(SchemaRef, Vec<usize>), DatasetError> {
    let visible_columns: Vec<_> = physical_schema
        .fields()
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            let manifest_column = manifest.and_then(|manifest| manifest.columns.get(field.name()));
            (!is_hidden_system_column(field.name(), manifest_column)).then_some(index)
        })
        .collect();
    let visible_schema = Arc::new(physical_schema.project(&visible_columns)?);
    Ok((visible_schema, visible_columns))
}

fn visible_projection_legacy(
    physical_schema: &SchemaRef,
) -> Result<(SchemaRef, Vec<usize>), DatasetError> {
    let visible_columns: Vec<_> = (0..physical_schema.fields().len()).collect();
    let visible_schema = Arc::new(physical_schema.project(&visible_columns)?);
    Ok((visible_schema, visible_columns))
}

fn project_batch(
    batch: &RecordBatch,
    output_schema: SchemaRef,
    physical_columns: &[usize],
) -> Result<RecordBatch, DatasetError> {
    let arrays = physical_columns
        .iter()
        .map(|&index| batch.column(index).clone())
        .collect();
    Ok(RecordBatch::try_new_with_options(
        output_schema,
        arrays,
        &RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
    )?)
}

impl DatasetReader {
    pub(crate) fn from_handle(
        source: WriteSessionHandle,
        manifest: Option<DatasetSemanticManifest>,
        dataset_path: Option<PathBuf>,
    ) -> Result<Self, ReadError> {
        let physical_arrow_schema = source.schema();
        if let Some(manifest) = manifest.as_ref() {
            manifest
                .validate_against_arrow_schema(physical_arrow_schema.as_ref())
                .map_err(ManifestError::from)?;
        }
        let (arrow_schema, visible_columns) =
            visible_projection_from_manifest(&physical_arrow_schema, manifest.as_ref())?;
        let schema = arrow_schema.as_ref().try_into().ok();
        Ok(Self {
            source: DatasetSource::WriteSession(source),
            schema,
            physical_arrow_schema,
            arrow_schema,
            visible_columns,
            manifest,
            dataset_path,
        })
    }

    pub(crate) fn open_dir(path: &Path) -> Result<Self, ReadError> {
        let mut reader = ChunkReader::new(path.to_owned(), None);
        reader.read_all()?;
        let physical_arrow_schema = reader.schema().ok_or(ReadError::EmptyDataset)?.clone();
        let manifest = read_manifest_optional(path)?;
        if let Some(manifest) = manifest.as_ref() {
            manifest
                .validate_against_arrow_schema(physical_arrow_schema.as_ref())
                .map_err(ManifestError::from)?;
        }
        let (arrow_schema, visible_columns) = if manifest.is_some() {
            visible_projection_from_manifest(&physical_arrow_schema, manifest.as_ref())?
        } else {
            visible_projection_legacy(&physical_arrow_schema)?
        };
        let schema = arrow_schema.as_ref().try_into().ok();
        Ok(Self {
            source: DatasetSource::File(reader),
            schema,
            physical_arrow_schema,
            arrow_schema,
            visible_columns,
            manifest,
            dataset_path: Some(path.to_owned()),
        })
    }

    pub fn schema(&self) -> Result<&DatasetSchema, ReadError> {
        self.schema
            .as_ref()
            .ok_or(ReadError::Dataset(DatasetError::IncompatibleType))
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
        self.source
            .range(..)
            .iter()
            .map(|batch| {
                project_batch(batch, self.arrow_schema.clone(), &self.visible_columns)
                    .expect("visible dataset projection should be valid")
            })
            .collect()
    }

    pub fn select_data(
        &self,
        options: &SelectOptions,
    ) -> Result<(SchemaRef, Vec<RecordBatch>), ReadError> {
        let visible_columns: Vec<_> = options.selected_columns.as_ref().map_or_else(
            || (0..self.visible_columns.len()).collect(),
            std::clone::Clone::clone,
        );
        let output_schema = Arc::new(
            self.arrow_schema
                .project(&visible_columns)
                .map_err(DatasetError::from)?,
        );
        let physical_columns: Vec<usize> = visible_columns
            .iter()
            .map(|&index| {
                self.visible_columns.get(index).copied().ok_or_else(|| {
                    DatasetError::Arrow(arrow_schema::ArrowError::InvalidArgumentError(format!(
                        "selected dataset column index out of bounds: {index}"
                    )))
                })
            })
            .try_collect()?;
        self.source
            .select_data(options, output_schema, &physical_columns)
    }

    pub fn interpret(&self) -> Result<DatasetInterpretation, ReadError> {
        if let Some(manifest) = &self.manifest {
            manifest
                .validate_against_arrow_schema(self.physical_arrow_schema.as_ref())
                .map_err(ManifestError::from)?;
            let compatibility_index_columns =
                if manifest.compatibility.allow_inference && manifest.scan_plan.is_none() {
                    self.try_index_columns().ok().flatten()
                } else {
                    None
                };
            return Ok(resolve_from_manifest_with_compatibility_inference(
                self.physical_arrow_schema.as_ref(),
                manifest,
                &self.visible_columns,
                compatibility_index_columns,
            ));
        }

        Ok(resolve_from_compatibility_inference(
            self.schema()?,
            self.try_index_columns()?,
        ))
    }

    pub fn logical_index_points(&self) -> Result<Vec<ResolvedLogicalIndexPoint>, ReadError> {
        let record_ids = self.record_ids()?;
        self.logical_index_points_for_record_ids(&record_ids)
    }

    pub fn logical_index_points_for_record_ids(
        &self,
        record_ids: &[u64],
    ) -> Result<Vec<ResolvedLogicalIndexPoint>, ReadError> {
        let Some(manifest) = &self.manifest else {
            return Ok(Vec::new());
        };
        manifest
            .validate_against_arrow_schema(self.physical_arrow_schema.as_ref())
            .map_err(ManifestError::from)?;
        if manifest.scan_plan.is_none() {
            return Ok(Vec::new());
        }
        if manifest.realization.index_realization == IndexRealization::Sidecar {
            if record_ids.is_empty() {
                return Ok(Vec::new());
            }
            let Some(path) = &self.dataset_path else {
                return Err(ReadError::DatasetFs(DatasetFsError::ChunkNotFound));
            };
            return Self::sidecar_logical_index_points(path, manifest, record_ids);
        }
        Ok(resolve_logical_index_points(manifest, record_ids))
    }

    fn sidecar_logical_index_points(
        path: &Path,
        manifest: &DatasetSemanticManifest,
        record_ids: &[u64],
    ) -> Result<Vec<ResolvedLogicalIndexPoint>, ReadError> {
        let scan_plan = manifest
            .scan_plan
            .as_ref()
            .expect("sidecar realization should require scan plan");
        let batches = read_logical_index_batches(path, scan_plan)?;
        if batches.is_empty() {
            return Err(ReadError::DatasetFs(DatasetFsError::ChunkNotFound));
        }
        let record_id_set = record_ids.iter().copied().collect::<HashSet<_>>();
        let mut matched_record_ids = HashSet::new();
        let mut latest_by_indices: HashMap<Vec<u64>, ResolvedLogicalIndexPoint> = HashMap::new();
        for batch in &batches {
            let record_ids = logical_index_record_ids(batch)?;
            let axis_columns = (0..scan_plan.axes.len())
                .map(|ordinal| logical_index_column(batch, ordinal))
                .collect::<Result<Vec<_>, _>>()?;
            for row in 0..batch.num_rows() {
                let record_id = record_ids.value(row);
                if !record_id_set.contains(&record_id) {
                    continue;
                }
                matched_record_ids.insert(record_id);
                let indices = axis_columns
                    .iter()
                    .map(|column| column.value(row))
                    .collect::<Vec<_>>();
                let coordinates = indices
                    .iter()
                    .zip(&scan_plan.axes)
                    .map(|(index, axis)| match &axis.mode {
                        ScanAxisMode::Static { values } => {
                            let index =
                                usize::try_from(*index).map_err(|_| DatasetError::InvalidFilter)?;
                            values
                                .get(index)
                                .cloned()
                                .ok_or(DatasetError::InvalidFilter)
                        }
                        ScanAxisMode::ImplicitIndex => Ok(ScanAxisValue::Int(
                            i64::try_from(*index).unwrap_or(i64::MAX),
                        )),
                    })
                    .collect::<Result<Vec<_>, DatasetError>>()?;
                let point = ResolvedLogicalIndexPoint {
                    record_id,
                    indices: indices.clone(),
                    coordinates,
                };
                latest_by_indices
                    .entry(indices)
                    .and_modify(|current| {
                        if record_id > current.record_id {
                            *current = point.clone();
                        }
                    })
                    .or_insert(point);
            }
        }
        if matched_record_ids != record_id_set {
            return Err(ReadError::Dataset(DatasetError::SchemaMismatch));
        }
        let mut points = latest_by_indices.into_values().collect::<Vec<_>>();
        points.sort_by_key(|point| point.record_id);
        Ok(points)
    }

    pub fn record_ids(&self) -> Result<Vec<u64>, ReadError> {
        self.record_ids_range(..)
    }

    pub fn record_ids_range<R>(&self, range: R) -> Result<Vec<u64>, ReadError>
    where
        R: RangeBounds<usize> + Copy,
    {
        let record_id_index = self
            .physical_arrow_schema
            .column_with_name(RECORD_ID_COLUMN)
            .ok_or_else(|| {
                DatasetError::Arrow(arrow_schema::ArrowError::SchemaError(format!(
                    "missing record id column {RECORD_ID_COLUMN}"
                )))
            })?
            .0;
        let mut record_ids = Vec::new();
        for batch in self.source.range(range) {
            let column = batch
                .column(record_id_index)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or(DatasetError::IncompatibleType)?;
            record_ids.extend(column.values().iter().copied());
        }
        Ok(record_ids)
    }

    #[must_use]
    pub fn index_columns(&self) -> Option<Vec<usize>> {
        self.try_index_columns().ok().flatten()
    }

    pub fn try_index_columns(&self) -> Result<Option<Vec<usize>>, ReadError> {
        let schema = self.schema()?;
        if self.source.num_rows() < 2 {
            Ok(None)
        } else {
            let samples = self
                .source
                .range(..2)
                .iter()
                .map(|batch| project_batch(batch, self.arrow_schema.clone(), &self.visible_columns))
                .try_collect::<_, Vec<_>, _>()?;
            let sample =
                concat_batches(&self.arrow_schema, &samples).expect("Should have same schema");
            let mut result = vec![];
            for (index, (sample_array, column_type)) in sample
                .columns()
                .iter()
                .zip(schema.columns().values())
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
            Ok(Some(result))
        }
    }
}

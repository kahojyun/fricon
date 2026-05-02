use std::{collections::HashMap, ops::Bound, sync::Arc};

use anyhow::{Context, Result, bail};
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use arrow_select::{concat::concat_batches, filter::FilterBuilder};
use fricon::{
    DatasetDataType, DatasetInterpretation, DatasetReader, DatasetSchema, InterpretationSource,
    ResolvedDuplicatePolicy, ResolvedLogicalIndexPoint, ResolvedScanAxisMode, ScalarKind,
    SelectOptions, dataset::semantics::ScanAxisValue,
};

use super::types::ChartCommonOptions;
use crate::desktop_runtime::session::WorkspaceSession;

pub(crate) const COLUMN_PREFIX: &str = "column:";
pub(crate) const LOGICAL_INDEX_PREFIX: &str = "logicalIndex:";

#[derive(Debug, Clone)]
pub(crate) struct AxisField {
    pub(crate) id: String,
    pub(crate) column_name: String,
    pub(crate) label: String,
}

#[derive(Debug)]
pub(crate) struct PreparedChartData {
    pub(crate) batch: RecordBatch,
    pub(crate) schema: DatasetSchema,
    pub(crate) index_columns: Option<Vec<usize>>,
}

pub(crate) fn column_id(name: &str) -> String {
    format!("{COLUMN_PREFIX}{name}")
}

pub(crate) fn logical_index_id(name: &str) -> String {
    format!("{LOGICAL_INDEX_PREFIX}{name}")
}

pub(crate) fn resolve_column_name(value: &str) -> String {
    value
        .strip_prefix(COLUMN_PREFIX)
        .unwrap_or(value)
        .to_string()
}

pub(crate) fn resolve_axis_column_name(value: &str) -> String {
    if value.starts_with(LOGICAL_INDEX_PREFIX) {
        value.to_string()
    } else {
        resolve_column_name(value)
    }
}

pub(crate) async fn prepare_chart_data(
    session: &WorkspaceSession,
    id: i32,
    common: &ChartCommonOptions,
    filters: &[(String, serde_json::Value)],
) -> Result<PreparedChartData> {
    let dataset = session.dataset(id).await?;
    let interpretation = dataset.interpret()?;
    let source_schema = dataset.schema()?.clone();
    let (start, end) = resolve_row_range(&dataset, common.start, common.end);
    let (output_schema, batches) = dataset.select_data(&SelectOptions {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
        index_filters: None,
        selected_columns: None,
    })?;
    let batch = concat_or_empty(output_schema, batches)?;
    prepare_batch_from_reader(
        &dataset,
        &source_schema,
        &interpretation,
        batch,
        start,
        end,
        filters,
    )
}

pub(crate) async fn load_axis_rows(
    session: &WorkspaceSession,
    id: i32,
    exclude_fields: &[String],
) -> Result<(Vec<AxisField>, Vec<Vec<serde_json::Value>>)> {
    let dataset = session.dataset(id).await?;
    let interpretation = dataset.interpret()?;
    let source_schema = dataset.schema()?.clone();
    let end = dataset.num_rows();
    let (output_schema, batches) = dataset.select_data(&SelectOptions {
        start: Bound::Included(0),
        end: Bound::Excluded(end),
        index_filters: None,
        selected_columns: None,
    })?;
    let batch = concat_or_empty(output_schema, batches)?;
    let prepared = prepare_batch_from_reader(
        &dataset,
        &source_schema,
        &interpretation,
        batch,
        0,
        end,
        &[],
    )?;
    let fields = axis_fields(&interpretation, &prepared.batch)
        .into_iter()
        .filter(|field| !exclude_fields.iter().any(|excluded| excluded == &field.id))
        .collect::<Vec<_>>();

    let rows = (0..prepared.batch.num_rows())
        .map(|row| {
            fields
                .iter()
                .map(|field| {
                    let column = prepared
                        .batch
                        .column_by_name(&field.column_name)
                        .with_context(|| format!("Axis '{}' not found", field.column_name))?;
                    json_value_at(column.as_ref(), row)
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((fields, rows))
}

pub(crate) fn apply_semantic_filters(
    batch: RecordBatch,
    filters: &[(String, serde_json::Value)],
) -> Result<RecordBatch> {
    if filters.is_empty() || batch.num_rows() == 0 {
        return Ok(batch);
    }

    let mut mask = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let mut keep = true;
        for (field, expected) in filters {
            let column_name = resolve_axis_column_name(field);
            let column = batch
                .column_by_name(&column_name)
                .with_context(|| format!("Filter field '{field}' not found"))?;
            if json_value_at(column.as_ref(), row)? != *expected {
                keep = false;
                break;
            }
        }
        mask.push(keep);
    }
    filter_batch(&batch, &mask)
}

fn prepare_batch_from_reader(
    dataset: &DatasetReader,
    source_schema: &DatasetSchema,
    interpretation: &DatasetInterpretation,
    batch: RecordBatch,
    start: usize,
    end: usize,
    filters: &[(String, serde_json::Value)],
) -> Result<PreparedChartData> {
    let mut schema = source_schema.clone();
    let mut batch = append_logical_axes(dataset, interpretation, &mut schema, batch, start, end)?;
    batch = apply_semantic_filters(batch, filters)?;
    let index_columns = resolve_index_columns(interpretation, &schema).or_else(|| {
        fallback_to_inferred_index_columns(interpretation)
            .then(|| dataset.try_index_columns().ok().flatten())
            .flatten()
    });
    Ok(PreparedChartData {
        batch,
        schema,
        index_columns,
    })
}

fn fallback_to_inferred_index_columns(interpretation: &DatasetInterpretation) -> bool {
    interpretation.source == InterpretationSource::CompatibilityInference
        || (interpretation.scan_axes.is_empty()
            && interpretation.chart_axis_candidate_columns.is_empty())
}

fn append_logical_axes(
    dataset: &DatasetReader,
    interpretation: &DatasetInterpretation,
    schema: &mut DatasetSchema,
    batch: RecordBatch,
    start: usize,
    end: usize,
) -> Result<RecordBatch> {
    if interpretation.scan_axes.is_empty() {
        return Ok(batch);
    }

    let point_by_record_id = dataset
        .logical_index_points()?
        .into_iter()
        .map(|point| (point.record_id, point))
        .collect::<HashMap<_, _>>();
    let record_ids = dataset.record_ids_range((Bound::Included(start), Bound::Excluded(end)))?;
    if record_ids.len() != batch.num_rows() {
        bail!("Logical index rows do not match selected chart rows");
    }

    let keep_mask = if interpretation.duplicate_policy == ResolvedDuplicatePolicy::LatestByRecordId
    {
        record_ids
            .iter()
            .map(|record_id| point_by_record_id.contains_key(record_id))
            .collect::<Vec<_>>()
    } else {
        vec![true; record_ids.len()]
    };
    let kept_record_ids = record_ids
        .into_iter()
        .zip(&keep_mask)
        .filter_map(|(record_id, keep)| (*keep).then_some(record_id))
        .collect::<Vec<_>>();
    let mut batch = filter_batch(&batch, &keep_mask)?;

    let mut fields = batch.schema().fields().iter().cloned().collect::<Vec<_>>();
    let mut arrays = batch.columns().to_vec();
    let mut columns = schema.columns().clone();
    for (axis_index, axis) in interpretation
        .scan_axes
        .iter()
        .enumerate()
        .filter(|(_, axis)| scan_axis_is_numeric(&axis.mode))
    {
        let id = logical_index_id(&axis.name);
        let values = logical_axis_values(&kept_record_ids, &point_by_record_id, axis_index);
        fields.push(Arc::new(Field::new(&id, DataType::Float64, true)));
        arrays.push(numeric_axis_array(&values));
        columns.insert(id, DatasetDataType::Scalar(ScalarKind::Numeric));
    }

    *schema = DatasetSchema::new(columns);
    for (axis_index, axis) in interpretation
        .scan_axes
        .iter()
        .enumerate()
        .filter(|(_, axis)| !scan_axis_is_numeric(&axis.mode))
    {
        let id = logical_index_id(&axis.name);
        let values = logical_axis_values(&kept_record_ids, &point_by_record_id, axis_index);
        fields.push(Arc::new(Field::new(
            &id,
            logical_axis_data_type(&axis.mode),
            true,
        )));
        arrays.push(logical_axis_array(&axis.mode, &values));
    }
    batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)?;
    Ok(batch)
}

fn axis_fields(interpretation: &DatasetInterpretation, batch: &RecordBatch) -> Vec<AxisField> {
    let mut fields = Vec::new();
    if interpretation.scan_axes.is_empty() {
        fields.extend(
            interpretation
                .logical_index_columns
                .iter()
                .filter_map(|ordinal| {
                    let column = interpretation
                        .columns
                        .iter()
                        .find(|column| column.visible_ordinal == Some(*ordinal))?;
                    Some(AxisField {
                        id: column_id(&column.name),
                        column_name: column.name.clone(),
                        label: column.label.clone().unwrap_or_else(|| column.name.clone()),
                    })
                }),
        );
    } else {
        fields.extend(interpretation.scan_axes.iter().filter_map(|axis| {
            let id = logical_index_id(&axis.name);
            batch.schema().field_with_name(&id).ok().map(|_| AxisField {
                id: id.clone(),
                column_name: id,
                label: axis.label.clone().unwrap_or_else(|| axis.name.clone()),
            })
        }));
    }
    fields
}

fn resolve_index_columns(
    interpretation: &DatasetInterpretation,
    schema: &DatasetSchema,
) -> Option<Vec<usize>> {
    let mut indices = Vec::new();
    for axis in interpretation
        .scan_axes
        .iter()
        .filter(|axis| scan_axis_is_numeric(&axis.mode))
    {
        let id = logical_index_id(&axis.name);
        if let Some((index, _, _)) = schema.columns().get_full(&id) {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

fn scan_axis_is_numeric(mode: &ResolvedScanAxisMode) -> bool {
    match mode {
        ResolvedScanAxisMode::ImplicitIndex => true,
        ResolvedScanAxisMode::Static { values } => values
            .iter()
            .all(|value| matches!(value, ScanAxisValue::Int(_) | ScanAxisValue::Float(_))),
    }
}

fn logical_axis_values(
    record_ids: &[u64],
    point_by_record_id: &HashMap<u64, ResolvedLogicalIndexPoint>,
    axis_index: usize,
) -> Vec<Option<ScanAxisValue>> {
    record_ids
        .iter()
        .map(|record_id| {
            point_by_record_id
                .get(record_id)
                .and_then(|point| point.coordinates.get(axis_index))
                .cloned()
        })
        .collect()
}

fn logical_axis_data_type(mode: &ResolvedScanAxisMode) -> DataType {
    match mode {
        ResolvedScanAxisMode::ImplicitIndex => DataType::Float64,
        ResolvedScanAxisMode::Static { values } => {
            if values
                .iter()
                .all(|value| matches!(value, ScanAxisValue::Bool(_)))
            {
                DataType::Boolean
            } else if values
                .iter()
                .all(|value| matches!(value, ScanAxisValue::Int(_) | ScanAxisValue::Float(_)))
            {
                DataType::Float64
            } else {
                DataType::Utf8
            }
        }
    }
}

fn logical_axis_array(mode: &ResolvedScanAxisMode, values: &[Option<ScanAxisValue>]) -> ArrayRef {
    match logical_axis_data_type(mode) {
        DataType::Boolean => Arc::new(BooleanArray::from(
            values
                .iter()
                .map(|value| match value {
                    Some(ScanAxisValue::Bool(value)) => Some(*value),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )),
        DataType::Float64 => numeric_axis_array(values),
        DataType::Utf8 => Arc::new(StringArray::from(
            values
                .iter()
                .map(|value| value.as_ref().map(scan_axis_value_to_string))
                .collect::<Vec<_>>(),
        )),
        _ => unreachable!("logical scan axes only use bool, numeric, or string arrays"),
    }
}

fn numeric_axis_array(values: &[Option<ScanAxisValue>]) -> ArrayRef {
    Arc::new(Float64Array::from(
        values
            .iter()
            .map(|value| match value.as_ref()? {
                ScanAxisValue::Int(value) => Some(*value as f64),
                ScanAxisValue::Float(value) => Some(*value),
                ScanAxisValue::Bool(_) | ScanAxisValue::String(_) => None,
            })
            .collect::<Vec<_>>(),
    ))
}

fn scan_axis_value_to_string(value: &ScanAxisValue) -> String {
    match value {
        ScanAxisValue::String(value) => value.clone(),
        ScanAxisValue::Int(value) => value.to_string(),
        ScanAxisValue::Float(value) => value.to_string(),
        ScanAxisValue::Bool(value) => value.to_string(),
    }
}

fn json_value_at(array: &dyn Array, row: usize) -> Result<serde_json::Value> {
    if array.is_null(row) {
        return Ok(serde_json::Value::Null);
    }
    match array.data_type() {
        DataType::Float64 => {
            let array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .context("Expected Float64Array")?;
            Ok(serde_json::Value::from(array.value(row)))
        }
        DataType::Boolean => {
            let array = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .context("Expected BooleanArray")?;
            Ok(serde_json::Value::from(array.value(row)))
        }
        DataType::Utf8 => {
            let array = array
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Expected StringArray")?;
            Ok(serde_json::Value::from(array.value(row)))
        }
        other => bail!("Unsupported semantic axis data type: {other}"),
    }
}

fn filter_batch(batch: &RecordBatch, mask: &[bool]) -> Result<RecordBatch> {
    if mask.iter().all(|keep| *keep) {
        return Ok(batch.clone());
    }
    let mask = BooleanArray::from(mask.to_vec());
    let predicate = FilterBuilder::new(&mask).build();
    let arrays = batch
        .columns()
        .iter()
        .map(|array| {
            predicate
                .filter(array)
                .context("Failed to filter chart data")
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(RecordBatch::try_new(batch.schema(), arrays)?)
}

fn concat_or_empty(schema: SchemaRef, batches: Vec<RecordBatch>) -> Result<RecordBatch> {
    if batches.is_empty() {
        Ok(RecordBatch::new_empty(schema))
    } else {
        concat_batches(&schema, &batches).context("Failed to concat chart batches")
    }
}

fn resolve_row_range(
    dataset: &DatasetReader,
    start: Option<usize>,
    end: Option<usize>,
) -> (usize, usize) {
    let total_rows = dataset.num_rows();
    let start = start.unwrap_or(0).min(total_rows);
    let end = end.unwrap_or(total_rows).min(total_rows).max(start);
    (start, end)
}

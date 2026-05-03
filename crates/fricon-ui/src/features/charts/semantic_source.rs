use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Bound,
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use arrow_array::{
    Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int64Array, RecordBatch,
    RecordBatchOptions, StringArray, StructArray, UInt64Array,
};
use arrow_schema::{DataType, Field, Fields, Schema, SchemaRef};
use arrow_select::{concat::concat_batches, filter::FilterBuilder};
#[cfg(test)]
use fricon::{
    ColumnMeaning, PhysicalColumnOrdinal, ResolvedColumn, ResolvedIndexRealization,
    ResolvedLogicalIndexReference, ResolvedPhysicalColumnReference, ResolvedSemanticReference,
    VisibleColumnOrdinal, dataset::semantics::DatasetDType,
};
use fricon::{
    DatasetDataType, DatasetInterpretation, DatasetReader, DatasetSchema, InterpretationSource,
    ResolvedDuplicatePolicy, ResolvedLogicalIndexPoint, ResolvedScanAxisMode, ScalarKind,
    SelectOptions, dataset::semantics::ScanAxisValue,
    physical_column_id as core_physical_column_id,
};
use indexmap::IndexMap;

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
    pub(crate) row_indices: Vec<usize>,
}

pub(crate) fn column_id(name: &str) -> String {
    core_physical_column_id(name)
}

pub(crate) fn resolve_column_name(value: &str) -> String {
    let Some(column_name) = value.strip_prefix(COLUMN_PREFIX) else {
        return value.to_string();
    };
    if column_name.starts_with(LOGICAL_INDEX_PREFIX) || column_name.starts_with(COLUMN_PREFIX) {
        value.to_string()
    } else {
        column_name.to_string()
    }
}

pub(crate) fn resolve_axis_column_name(value: &str) -> String {
    if value.starts_with(LOGICAL_INDEX_PREFIX) {
        value.to_string()
    } else {
        resolve_column_name(value)
    }
}

fn resolve_source_column_name(value: &str) -> String {
    value
        .strip_prefix(COLUMN_PREFIX)
        .unwrap_or(value)
        .to_string()
}

fn resolve_axis_column_name_for_interpretation(
    interpretation: &DatasetInterpretation,
    value: &str,
) -> String {
    if let Some(axis) = interpretation.logical_axis_for_id(value) {
        return axis.id.clone();
    }
    if let Some(column) = interpretation.physical_column_for_id(value) {
        return alias_physical_column_name(&column.name, true);
    }
    resolve_axis_column_name(value)
}

fn resolve_source_column_name_for_interpretation(
    interpretation: &DatasetInterpretation,
    value: &str,
) -> Option<String> {
    if interpretation.logical_axis_for_id(value).is_some() {
        return None;
    }
    Some(interpretation.physical_column_for_id(value).map_or_else(
        || resolve_source_column_name(value),
        |column| column.name.clone(),
    ))
}

pub(crate) async fn prepare_chart_data(
    session: &WorkspaceSession,
    id: i32,
    common: &ChartCommonOptions,
    filters: &[(String, serde_json::Value)],
    selected_columns: Option<&[usize]>,
) -> Result<PreparedChartData> {
    let dataset = session.dataset(id).await?;
    let interpretation = dataset.interpret()?;
    let source_schema = dataset.schema()?.clone();
    let alias_physical_columns = true;
    let (start, end) = resolve_row_range(&dataset, common.start, common.end);
    let selected_physical_columns = selected_physical_columns(
        &source_schema,
        Some(&interpretation),
        selected_columns,
        filters,
    );
    let (output_schema, batches) = dataset.select_data(&SelectOptions {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
        index_filters: None,
        selected_columns: selected_physical_columns.clone(),
    })?;
    let batch = concat_or_empty(output_schema, &batches)?;
    let selected_schema = selected_physical_columns.as_deref().map_or_else(
        || project_all_schema(&source_schema, alias_physical_columns),
        |columns| project_schema(&source_schema, columns, alias_physical_columns),
    )?;
    let batch = rename_batch(&batch, &selected_schema)?;
    prepare_batch_from_reader(
        &dataset,
        &selected_schema,
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
    let selected_columns = axis_row_selected_columns(&dataset, &interpretation)?;
    if interpretation.scan_axes.is_empty() && selected_columns.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let batch = if selected_columns.is_empty() {
        empty_row_count_batch(end)?
    } else {
        let (output_schema, batches) = dataset.select_data(&SelectOptions {
            start: Bound::Included(0),
            end: Bound::Excluded(end),
            index_filters: None,
            selected_columns: Some(selected_columns.clone()),
        })?;
        concat_or_empty(output_schema, &batches)?
    };
    let selected_schema = project_schema(&source_schema, &selected_columns, false)?;
    let prepared = prepare_batch_from_reader(
        &dataset,
        &selected_schema,
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

fn semantic_filter_mask(
    interpretation: &DatasetInterpretation,
    batch: &RecordBatch,
    filters: &[(String, serde_json::Value)],
) -> Result<Option<Vec<bool>>> {
    if filters.is_empty() || batch.num_rows() == 0 {
        return Ok(None);
    }
    let mut mask = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let mut keep = true;
        for (field, expected) in filters {
            let column_name = resolve_axis_column_name_for_interpretation(interpretation, field);
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
    Ok(Some(mask))
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
    let mut row_indices = (start..end).collect::<Vec<_>>();
    let mut batch = append_logical_axes(
        dataset,
        interpretation,
        &mut schema,
        batch,
        &mut row_indices,
        start,
        end,
    )?;
    if let Some(mask) = semantic_filter_mask(interpretation, &batch, filters)? {
        batch = filter_batch(&batch, &mask)?;
        row_indices = filter_row_indices(&row_indices, &mask);
    }
    let index_columns = resolve_index_columns(interpretation, &schema).or_else(|| {
        fallback_to_inferred_index_columns(interpretation)
            .then(|| {
                let full_schema = dataset.schema().ok()?;
                let indices = dataset.try_index_columns().ok().flatten()?;
                Some(map_full_index_columns(full_schema, &schema, &indices))
            })
            .flatten()
    });
    Ok(PreparedChartData {
        batch,
        schema,
        index_columns,
        row_indices,
    })
}

fn fallback_to_inferred_index_columns(interpretation: &DatasetInterpretation) -> bool {
    interpretation.source == InterpretationSource::CompatibilityInference
        || interpretation.scan_axes.is_empty()
}

fn axis_row_selected_columns(
    dataset: &DatasetReader,
    interpretation: &DatasetInterpretation,
) -> Result<Vec<usize>> {
    if !interpretation.scan_axes.is_empty() {
        return Ok(Vec::new());
    }
    let Some(index_columns) = dataset.try_index_columns()? else {
        return Ok(Vec::new());
    };
    Ok(index_columns)
}

fn selected_physical_columns(
    source_schema: &DatasetSchema,
    interpretation: Option<&DatasetInterpretation>,
    selected_columns: Option<&[usize]>,
    filters: &[(String, serde_json::Value)],
) -> Option<Vec<usize>> {
    let selected_columns = selected_columns?;

    let mut physical_columns = Vec::new();
    for &index in selected_columns {
        if index < source_schema.columns().len() {
            push_unique(&mut physical_columns, index);
        }
    }
    for (field, _) in filters {
        let column_name = interpretation.map_or_else(
            || Some(resolve_source_column_name(field)),
            |interpretation| resolve_source_column_name_for_interpretation(interpretation, field),
        );
        let Some(column_name) = column_name else {
            continue;
        };
        if let Some((index, _, _)) = source_schema.columns().get_full(&column_name) {
            push_unique(&mut physical_columns, index);
        }
    }
    Some(physical_columns)
}

fn empty_row_count_batch(row_count: usize) -> Result<RecordBatch> {
    Ok(RecordBatch::try_new_with_options(
        Arc::new(Schema::new(Vec::<Field>::new())),
        Vec::new(),
        &RecordBatchOptions::new().with_row_count(Some(row_count)),
    )?)
}

fn push_unique(columns: &mut Vec<usize>, index: usize) {
    if !columns.contains(&index) {
        columns.push(index);
    }
}

fn project_all_schema(
    source_schema: &DatasetSchema,
    alias_physical: bool,
) -> Result<DatasetSchema> {
    let columns = (0..source_schema.columns().len()).collect::<Vec<_>>();
    project_schema(source_schema, &columns, alias_physical)
}

fn project_schema(
    source_schema: &DatasetSchema,
    columns: &[usize],
    alias_physical: bool,
) -> Result<DatasetSchema> {
    let mut projected = IndexMap::new();
    for &index in columns {
        let (name, dtype) = source_schema
            .columns()
            .get_index(index)
            .with_context(|| format!("Selected dataset column index out of bounds: {index}"))?;
        projected.insert(alias_physical_column_name(name, alias_physical), *dtype);
    }
    Ok(DatasetSchema::new(projected))
}

fn alias_physical_column_name(name: &str, alias_physical: bool) -> String {
    if alias_physical && (name.starts_with(LOGICAL_INDEX_PREFIX) || name.starts_with(COLUMN_PREFIX))
    {
        column_id(name)
    } else {
        name.to_string()
    }
}

fn rename_batch(batch: &RecordBatch, schema: &DatasetSchema) -> Result<RecordBatch> {
    let arrow_schema = Arc::new(schema.to_arrow_schema());
    let columns = batch
        .columns()
        .iter()
        .zip(arrow_schema.fields())
        .map(|(column, field)| coerce_column_for_field(column, field))
        .collect::<Result<Vec<_>>>()?;
    Ok(RecordBatch::try_new(arrow_schema, columns)?)
}

fn coerce_column_for_field(column: &ArrayRef, field: &Field) -> Result<ArrayRef> {
    if column.data_type() == field.data_type() {
        return Ok(column.clone());
    }
    if matches!(field.data_type(), DataType::Float64) {
        return coerce_numeric_to_float64(column.as_ref());
    }
    Ok(column.clone())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Chart transforms normalize supported numeric axis candidates to Float64"
)]
fn coerce_numeric_to_float64(column: &dyn Array) -> Result<ArrayRef> {
    match column.data_type() {
        DataType::Float32 => {
            let array = column
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Expected Float32Array")?;
            Ok(Arc::new(
                array
                    .iter()
                    .map(|value| value.map(f64::from))
                    .collect::<Float64Array>(),
            ))
        }
        DataType::Int64 => {
            let array = column
                .as_any()
                .downcast_ref::<Int64Array>()
                .context("Expected Int64Array")?;
            Ok(Arc::new(
                array
                    .iter()
                    .map(|value| value.map(|value| value as f64))
                    .collect::<Float64Array>(),
            ))
        }
        DataType::UInt64 => {
            let array = column
                .as_any()
                .downcast_ref::<UInt64Array>()
                .context("Expected UInt64Array")?;
            Ok(Arc::new(
                array
                    .iter()
                    .map(|value| value.map(|value| value as f64))
                    .collect::<Float64Array>(),
            ))
        }
        _ => bail!(
            "Cannot use {} column as a Float64 chart coordinate",
            column.data_type()
        ),
    }
}

fn map_full_index_columns(
    full_schema: &DatasetSchema,
    selected_schema: &DatasetSchema,
    indices: &[usize],
) -> Vec<usize> {
    indices
        .iter()
        .filter_map(|&index| {
            let name = full_schema.columns().get_index(index)?.0;
            selected_schema
                .columns()
                .get_full(name)
                .or_else(|| {
                    selected_schema
                        .columns()
                        .get_full(&alias_physical_column_name(name, true))
                })
                .map(|(selected_index, _, _)| selected_index)
        })
        .collect()
}

fn append_logical_axes(
    dataset: &DatasetReader,
    interpretation: &DatasetInterpretation,
    schema: &mut DatasetSchema,
    batch: RecordBatch,
    row_indices: &mut Vec<usize>,
    start: usize,
    end: usize,
) -> Result<RecordBatch> {
    if interpretation.scan_axes.is_empty() {
        return Ok(batch);
    }

    let record_ids = dataset.record_ids_range((Bound::Included(start), Bound::Excluded(end)))?;
    if record_ids.len() != batch.num_rows() {
        bail!("Logical index rows do not match selected chart rows");
    }
    if row_indices.len() != batch.num_rows() {
        bail!("Logical row indices do not match selected chart rows");
    }
    let point_by_record_id = logical_points_by_record_id(
        dataset.logical_index_points_for_record_ids(&record_ids)?,
        interpretation.duplicate_policy,
    );

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
    *row_indices = filter_row_indices(row_indices, &keep_mask);
    let mut batch = filter_batch(&batch, &keep_mask)?;

    let mut fields = batch.schema().fields().iter().cloned().collect::<Vec<_>>();
    let mut arrays = batch.columns().to_vec();
    let mut columns = schema.columns().clone();
    for (axis_index, axis) in interpretation.scan_axes.iter().enumerate() {
        let id = axis.id.clone();
        let values = logical_axis_values(&kept_record_ids, &point_by_record_id, axis_index);
        if axis.numeric_axis {
            fields.push(Arc::new(Field::new(&id, DataType::Float64, true)));
            arrays.push(numeric_axis_array(&values));
            columns.insert(id, DatasetDataType::Scalar(ScalarKind::Numeric));
        } else {
            fields.push(Arc::new(Field::new(
                &id,
                logical_axis_data_type(&axis.mode),
                true,
            )));
            arrays.push(logical_axis_array(&axis.mode, &values));
            columns.insert(id, DatasetDataType::Scalar(ScalarKind::Complex));
        }
    }
    *schema = DatasetSchema::new(columns);
    batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)?;
    Ok(batch)
}

fn filter_row_indices(row_indices: &[usize], mask: &[bool]) -> Vec<usize> {
    row_indices
        .iter()
        .copied()
        .zip(mask)
        .filter_map(|(row, keep)| (*keep).then_some(row))
        .collect()
}

fn axis_fields(interpretation: &DatasetInterpretation, batch: &RecordBatch) -> Vec<AxisField> {
    interpretation
        .group_axes
        .iter()
        .filter_map(|reference| match reference {
            fricon::ResolvedSemanticReference::PhysicalColumn(column) => {
                axis_field_column_name(batch, &column.name).map(|column_name| AxisField {
                    id: column.id.clone(),
                    column_name,
                    label: column.label.clone().unwrap_or_else(|| column.name.clone()),
                })
            }
            fricon::ResolvedSemanticReference::LogicalIndex(axis) => batch
                .schema()
                .field_with_name(&axis.id)
                .ok()
                .map(|_| AxisField {
                    id: axis.id.clone(),
                    column_name: axis.id.clone(),
                    label: axis.label.clone().unwrap_or_else(|| axis.name.clone()),
                }),
        })
        .collect()
}

fn axis_field_column_name(batch: &RecordBatch, physical_name: &str) -> Option<String> {
    if batch.schema().field_with_name(physical_name).is_ok() {
        return Some(physical_name.to_string());
    }
    let aliased_name = alias_physical_column_name(physical_name, true);
    batch
        .schema()
        .field_with_name(&aliased_name)
        .ok()
        .map(|_| aliased_name)
}

fn resolve_index_columns(
    interpretation: &DatasetInterpretation,
    schema: &DatasetSchema,
) -> Option<Vec<usize>> {
    let mut indices = Vec::new();
    for reference in interpretation
        .sweep_axes
        .iter()
        .filter(|reference| reference.numeric_axis())
    {
        match reference {
            fricon::ResolvedSemanticReference::PhysicalColumn(column) => {
                let column_name = alias_physical_column_name(&column.name, true);
                if let Some((index, _, _)) = schema.columns().get_full(&column_name) {
                    indices.push(index);
                }
            }
            fricon::ResolvedSemanticReference::LogicalIndex(axis) => {
                if let Some((index, _, _)) = schema.columns().get_full(&axis.id) {
                    indices.push(index);
                }
            }
        }
    }
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

fn logical_points_by_record_id(
    points: Vec<ResolvedLogicalIndexPoint>,
    duplicate_policy: ResolvedDuplicatePolicy,
) -> HashMap<u64, ResolvedLogicalIndexPoint> {
    if duplicate_policy != ResolvedDuplicatePolicy::LatestByRecordId {
        return points
            .into_iter()
            .map(|point| (point.record_id, point))
            .collect();
    }

    let mut latest_by_indices: HashMap<Vec<u64>, ResolvedLogicalIndexPoint> = HashMap::new();
    for point in points {
        match latest_by_indices.entry(point.indices.clone()) {
            Entry::Occupied(mut entry) if point.record_id > entry.get().record_id => {
                entry.insert(point);
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(point);
            }
        }
    }
    latest_by_indices
        .into_values()
        .map(|point| (point.record_id, point))
        .collect()
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

#[expect(
    clippy::cast_precision_loss,
    reason = "Logical integer scan coordinates are plotted on Float64 chart axes"
)]
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
        DataType::Struct(fields) if is_complex_fields(fields) => {
            let array = array
                .as_any()
                .downcast_ref::<StructArray>()
                .context("Expected complex StructArray")?;
            let real = array
                .column(0)
                .as_any()
                .downcast_ref::<Float64Array>()
                .context("Expected complex real Float64Array")?
                .value(row);
            let imag = array
                .column(1)
                .as_any()
                .downcast_ref::<Float64Array>()
                .context("Expected complex imag Float64Array")?
                .value(row);
            Ok(serde_json::json!({
                "real": real,
                "imag": imag,
            }))
        }
        other => bail!("Unsupported semantic axis data type: {other}"),
    }
}

fn is_complex_fields(fields: &Fields) -> bool {
    fields.as_ref().as_array::<2>().is_some_and(|[real, imag]| {
        real.name() == "real"
            && imag.name() == "imag"
            && matches!(real.data_type(), DataType::Float64)
            && matches!(imag.data_type(), DataType::Float64)
    })
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

fn concat_or_empty(schema: SchemaRef, batches: &[RecordBatch]) -> Result<RecordBatch> {
    if batches.is_empty() {
        Ok(RecordBatch::new_empty(schema))
    } else {
        concat_batches(&schema, batches).context("Failed to concat chart batches")
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

#[cfg(test)]
mod tests {
    use arrow_array::StructArray;

    use super::*;

    fn point(record_id: u64, indices: Vec<u64>) -> ResolvedLogicalIndexPoint {
        ResolvedLogicalIndexPoint {
            record_id,
            indices,
            coordinates: Vec::new(),
        }
    }

    #[test]
    fn latest_logical_points_are_projected_within_selected_points() {
        let projected = logical_points_by_record_id(
            vec![
                point(0, vec![0, 0]),
                point(1, vec![0, 1]),
                point(2, vec![0, 0]),
            ],
            ResolvedDuplicatePolicy::LatestByRecordId,
        );

        assert!(!projected.contains_key(&0));
        assert!(projected.contains_key(&1));
        assert!(projected.contains_key(&2));
    }

    #[test]
    fn bounded_logical_points_do_not_consider_later_replacements() {
        let projected = logical_points_by_record_id(
            vec![point(0, vec![0, 0]), point(1, vec![0, 1])],
            ResolvedDuplicatePolicy::LatestByRecordId,
        );

        assert!(projected.contains_key(&0));
        assert!(projected.contains_key(&1));
    }

    #[test]
    fn manifest_without_scan_axes_still_uses_inferred_index_fallback() {
        let interpretation = DatasetInterpretation {
            columns: Vec::new(),
            semantic_references: Vec::new(),
            value_references: Vec::new(),
            plotted_coordinates: Vec::new(),
            sweep_axes: Vec::new(),
            group_axes: Vec::new(),
            filter_axes: Vec::new(),
            chart_axis_candidates: Vec::new(),
            value_columns: Vec::new(),
            logical_index_columns: Vec::new(),
            chart_axis_candidate_columns: vec![fricon::VisibleColumnOrdinal(0)],
            duplicate_policy: ResolvedDuplicatePolicy::LatestByRecordId,
            index_realization: fricon::ResolvedIndexRealization::None,
            scan_axes: Vec::new(),
            source: InterpretationSource::Manifest,
        };

        assert!(fallback_to_inferred_index_columns(&interpretation));
    }

    #[test]
    fn selected_physical_columns_keep_requested_payloads_and_filter_axes() {
        let source_schema = DatasetSchema::new(IndexMap::from([
            (
                "quantity".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
            (
                "axis".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
        ]));

        let selected = selected_physical_columns(
            &source_schema,
            None,
            Some(&[0, 2]),
            &[("column:axis".to_string(), serde_json::json!(1.0))],
        )
        .expect("selected columns");

        assert_eq!(selected, vec![0, 1]);
    }

    #[test]
    fn rename_batch_normalizes_supported_numeric_columns_to_float64() {
        let source_schema = Arc::new(Schema::new(vec![Field::new(
            "axis",
            DataType::Int64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            source_schema,
            vec![Arc::new(Int64Array::from(vec![1_i64, 2_i64])) as ArrayRef],
        )
        .expect("source batch");
        let target_schema = DatasetSchema::new(IndexMap::from([(
            "axis".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        )]));

        let renamed = rename_batch(&batch, &target_schema).expect("renamed batch");
        let axis = renamed
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("Float64 chart axis");

        assert_eq!(axis.values(), &[1.0, 2.0]);
    }

    #[test]
    fn prefixed_physical_columns_keep_column_namespace() {
        assert_eq!(
            resolve_column_name("column:logicalIndex:gate"),
            "column:logicalIndex:gate"
        );
        assert_eq!(
            resolve_axis_column_name("column:logicalIndex:gate"),
            "column:logicalIndex:gate"
        );
        assert_eq!(
            resolve_source_column_name("column:logicalIndex:gate"),
            "logicalIndex:gate"
        );

        let source_schema = DatasetSchema::new(IndexMap::from([(
            "logicalIndex:gate".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        )]));
        let projected = project_schema(&source_schema, &[0], true).expect("project schema");

        assert!(projected.columns().contains_key("column:logicalIndex:gate"));
        assert_eq!(
            map_full_index_columns(&source_schema, &projected, &[0]),
            vec![0]
        );
    }

    #[test]
    fn axis_fields_find_unaliased_prefixed_compatibility_columns() {
        let source_schema = Arc::new(Schema::new(vec![Field::new(
            "logicalIndex:gate",
            DataType::Float64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            source_schema,
            vec![Arc::new(Float64Array::from(vec![1.0])) as ArrayRef],
        )
        .expect("source batch");
        let reference =
            ResolvedSemanticReference::PhysicalColumn(ResolvedPhysicalColumnReference {
                id: "column:logicalIndex:gate".to_string(),
                name: "logicalIndex:gate".to_string(),
                physical_ordinal: PhysicalColumnOrdinal(0),
                visible_ordinal: Some(VisibleColumnOrdinal(0)),
                dtype: DatasetDType::Float64,
                meaning: ColumnMeaning::CompatibilityIndex,
                is_index: true,
                is_system: false,
                is_compatibility: true,
                hidden_by_default: false,
                is_chart_axis_candidate: true,
                unit: None,
                label: None,
                is_complex: false,
                is_trace: false,
                numeric_axis: true,
            });
        let interpretation = DatasetInterpretation {
            columns: Vec::<ResolvedColumn>::new(),
            semantic_references: vec![reference.clone()],
            value_references: Vec::new(),
            plotted_coordinates: vec![reference.clone()],
            sweep_axes: vec![reference.clone()],
            group_axes: vec![reference.clone()],
            filter_axes: vec![reference.clone()],
            chart_axis_candidates: vec![reference],
            value_columns: Vec::new(),
            logical_index_columns: vec![VisibleColumnOrdinal(0)],
            chart_axis_candidate_columns: vec![VisibleColumnOrdinal(0)],
            duplicate_policy: ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder,
            index_realization: ResolvedIndexRealization::None,
            scan_axes: Vec::new(),
            source: InterpretationSource::CompatibilityInference,
        };

        let fields = axis_fields(&interpretation, &batch);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].id, "column:logicalIndex:gate");
        assert_eq!(fields[0].column_name, "logicalIndex:gate");
    }

    #[test]
    fn index_columns_use_sweep_axes_not_chart_axis_candidates() {
        let schema = DatasetSchema::new(IndexMap::from([
            (
                "signal".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
            (
                "physicalAxis".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
            (
                "logicalIndex:gate".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
        ]));
        let logical_gate = ResolvedSemanticReference::LogicalIndex(ResolvedLogicalIndexReference {
            id: "logicalIndex:gate".to_string(),
            name: "gate".to_string(),
            axis_ordinal: 0,
            label: None,
            hidden_by_default: false,
            numeric_axis: true,
            is_compatibility: false,
        });
        let chart_axis =
            ResolvedSemanticReference::PhysicalColumn(ResolvedPhysicalColumnReference {
                id: "column:physicalAxis".to_string(),
                name: "physicalAxis".to_string(),
                physical_ordinal: PhysicalColumnOrdinal(1),
                visible_ordinal: Some(VisibleColumnOrdinal(1)),
                dtype: DatasetDType::Float64,
                meaning: ColumnMeaning::UserValue,
                is_index: false,
                is_system: false,
                is_compatibility: false,
                hidden_by_default: false,
                is_chart_axis_candidate: true,
                unit: None,
                label: None,
                is_complex: false,
                is_trace: false,
                numeric_axis: true,
            });
        let interpretation = DatasetInterpretation {
            columns: Vec::<ResolvedColumn>::new(),
            semantic_references: vec![logical_gate.clone(), chart_axis.clone()],
            value_references: Vec::new(),
            plotted_coordinates: vec![logical_gate.clone(), chart_axis.clone()],
            sweep_axes: vec![logical_gate],
            group_axes: Vec::new(),
            filter_axes: Vec::new(),
            chart_axis_candidates: vec![chart_axis],
            value_columns: Vec::new(),
            logical_index_columns: Vec::new(),
            chart_axis_candidate_columns: Vec::new(),
            duplicate_policy: ResolvedDuplicatePolicy::LatestByRecordId,
            index_realization: ResolvedIndexRealization::Implicit,
            scan_axes: Vec::new(),
            source: InterpretationSource::Manifest,
        };

        let index_columns = resolve_index_columns(&interpretation, &schema);

        assert_eq!(index_columns, Some(vec![2]));
    }

    #[test]
    fn json_value_at_serializes_complex_struct_axes() {
        let array = StructArray::new(
            vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ]
            .into(),
            vec![
                Arc::new(Float64Array::from(vec![1.0])) as ArrayRef,
                Arc::new(Float64Array::from(vec![2.0])) as ArrayRef,
            ],
            None,
        );

        assert_eq!(
            json_value_at(&array, 0).expect("complex value"),
            serde_json::json!({"real": 1.0, "imag": 2.0})
        );
    }
}

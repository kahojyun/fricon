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
use indexmap::IndexMap;

use crate::dataset::{
    interpret::{
        DatasetInterpretation, ResolvedDuplicatePolicy, ResolvedLogicalIndexPoint,
        ResolvedScanAxisMode, ResolvedSemanticKind, ResolvedSemanticReference, physical_column_id,
    },
    read::{DatasetReader, SelectOptions},
    schema::{DatasetDataType, DatasetSchema, ScalarKind},
    semantics::ScanAxisValue,
};

const LOGICAL_INDEX_PREFIX: &str = "logicalIndex:";
const COLUMN_PREFIX: &str = "column:";

#[derive(Debug, Clone)]
pub struct SemanticProjectionOptions<'a> {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub filters: &'a [(String, serde_json::Value)],
    pub selected_columns: Option<&'a [usize]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedSemanticAxis {
    pub id: String,
    pub column_name: String,
    pub label: String,
    pub semantic_kind: ResolvedSemanticKind,
    pub numeric: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectedSemanticRoles {
    pub sweep_columns: Vec<usize>,
    pub group_columns: Vec<usize>,
    pub filter_columns: Vec<usize>,
}

#[derive(Debug)]
pub struct ProjectedSemanticSource {
    pub batch: RecordBatch,
    pub schema: DatasetSchema,
    pub row_indices: Vec<usize>,
    pub semantic_column_names: HashMap<String, String>,
    pub roles: ProjectedSemanticRoles,
    pub group_axes: Vec<ProjectedSemanticAxis>,
    pub filter_axes: Vec<ProjectedSemanticAxis>,
}

impl ProjectedSemanticSource {
    #[must_use]
    pub fn sweep_columns(&self) -> Option<&[usize]> {
        (!self.roles.sweep_columns.is_empty()).then_some(&self.roles.sweep_columns)
    }

    #[must_use]
    pub fn group_columns(&self) -> Option<&[usize]> {
        (!self.roles.group_columns.is_empty()).then_some(&self.roles.group_columns)
    }
}

pub fn project_semantic_source(
    dataset: &DatasetReader,
    options: &SemanticProjectionOptions<'_>,
) -> Result<ProjectedSemanticSource> {
    let interpretation = dataset.interpret()?;
    let source_schema = dataset.schema()?.clone();
    let (start, end) = resolve_row_range(dataset, options.start, options.end);
    let selected_physical_columns = selected_physical_columns(
        &source_schema,
        &interpretation,
        options.selected_columns,
        options.filters,
    );
    let selected_schema = selected_physical_columns.as_deref().map_or_else(
        || project_all_schema(&source_schema),
        |columns| project_schema(&source_schema, columns),
    )?;
    let batch = if matches!(selected_physical_columns.as_deref(), Some([])) {
        empty_row_count_batch(end.saturating_sub(start))?
    } else {
        let (output_schema, batches) = dataset.select_data(&SelectOptions {
            start: Bound::Included(start),
            end: Bound::Excluded(end),
            index_filters: None,
            selected_columns: selected_physical_columns.clone(),
        })?;
        concat_or_empty(output_schema, &batches)?
    };
    let batch = rename_batch(&batch, &selected_schema)?;

    project_selected_batch(
        dataset,
        &selected_schema,
        &interpretation,
        batch,
        start,
        end,
        options.filters,
    )
}

fn project_selected_batch(
    dataset: &DatasetReader,
    source_schema: &DatasetSchema,
    interpretation: &DatasetInterpretation,
    batch: RecordBatch,
    start: usize,
    end: usize,
    filters: &[(String, serde_json::Value)],
) -> Result<ProjectedSemanticSource> {
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
    let semantic_column_names = semantic_column_names(interpretation, &schema);
    let roles = projected_roles(interpretation, &schema);
    let group_axes = projected_axes(&interpretation.group_axes, &batch);
    let filter_axes = projected_axes(&interpretation.filter_axes, &batch);
    Ok(ProjectedSemanticSource {
        batch,
        schema,
        row_indices,
        semantic_column_names,
        roles,
        group_axes,
        filter_axes,
    })
}

fn selected_physical_columns(
    source_schema: &DatasetSchema,
    interpretation: &DatasetInterpretation,
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
        let Some(column_name) = source_column_name_for_semantic_id(interpretation, field) else {
            continue;
        };
        if let Some((index, _, _)) = source_schema.columns().get_full(&column_name) {
            push_unique(&mut physical_columns, index);
        }
    }
    Some(physical_columns)
}

fn source_column_name_for_semantic_id(
    interpretation: &DatasetInterpretation,
    value: &str,
) -> Option<String> {
    if interpretation.logical_axis_for_id(value).is_some() {
        return None;
    }
    Some(interpretation.physical_column_for_id(value).map_or_else(
        || {
            value
                .strip_prefix(COLUMN_PREFIX)
                .unwrap_or(value)
                .to_string()
        },
        |column| column.name.clone(),
    ))
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
            let column_name = projected_column_name_for_semantic_id(interpretation, field);
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

fn project_all_schema(source_schema: &DatasetSchema) -> Result<DatasetSchema> {
    let columns = (0..source_schema.columns().len()).collect::<Vec<_>>();
    project_schema(source_schema, &columns)
}

fn project_schema(source_schema: &DatasetSchema, columns: &[usize]) -> Result<DatasetSchema> {
    let mut projected = IndexMap::new();
    for &index in columns {
        let (name, dtype) = source_schema
            .columns()
            .get_index(index)
            .with_context(|| format!("Selected dataset column index out of bounds: {index}"))?;
        projected.insert(alias_physical_column_name(name), *dtype);
    }
    Ok(DatasetSchema::new(projected))
}

fn empty_row_count_batch(row_count: usize) -> Result<RecordBatch> {
    Ok(RecordBatch::try_new_with_options(
        Arc::new(Schema::new(Vec::<Field>::new())),
        Vec::new(),
        &RecordBatchOptions::new().with_row_count(Some(row_count)),
    )?)
}

fn alias_physical_column_name(name: &str) -> String {
    if name.starts_with(LOGICAL_INDEX_PREFIX) || name.starts_with(COLUMN_PREFIX) {
        physical_column_id(name)
    } else {
        name.to_string()
    }
}

fn projected_column_name_for_semantic_id(
    interpretation: &DatasetInterpretation,
    value: &str,
) -> String {
    if let Some(axis) = interpretation.logical_axis_for_id(value) {
        return axis.id.clone();
    }
    if let Some(column) = interpretation.physical_column_for_id(value) {
        return alias_physical_column_name(&column.name);
    }
    if value.starts_with(LOGICAL_INDEX_PREFIX) {
        value.to_string()
    } else {
        value
            .strip_prefix(COLUMN_PREFIX)
            .unwrap_or(value)
            .to_string()
    }
}

fn rename_batch(batch: &RecordBatch, schema: &DatasetSchema) -> Result<RecordBatch> {
    let arrow_schema = Arc::new(schema.to_arrow_schema());
    if arrow_schema.fields().is_empty() {
        return Ok(RecordBatch::try_new_with_options(
            arrow_schema,
            Vec::new(),
            &RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
        )?);
    }
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

fn semantic_column_names(
    interpretation: &DatasetInterpretation,
    schema: &DatasetSchema,
) -> HashMap<String, String> {
    interpretation
        .semantic_references
        .iter()
        .filter_map(|reference| projected_column_name(reference, schema))
        .collect()
}

fn projected_column_name(
    reference: &ResolvedSemanticReference,
    schema: &DatasetSchema,
) -> Option<(String, String)> {
    match reference {
        ResolvedSemanticReference::PhysicalColumn(column) => {
            let column_name = alias_physical_column_name(&column.name);
            schema
                .columns()
                .contains_key(&column_name)
                .then(|| (column.id.clone(), column_name))
        }
        ResolvedSemanticReference::LogicalIndex(axis) => schema
            .columns()
            .contains_key(&axis.id)
            .then(|| (axis.id.clone(), axis.id.clone())),
    }
}

fn projected_roles(
    interpretation: &DatasetInterpretation,
    schema: &DatasetSchema,
) -> ProjectedSemanticRoles {
    ProjectedSemanticRoles {
        sweep_columns: projected_reference_columns(
            &interpretation.sweep_axes,
            schema,
            ResolvedSemanticReference::numeric_axis,
        ),
        group_columns: projected_reference_columns(&interpretation.group_axes, schema, |_| true),
        filter_columns: projected_reference_columns(&interpretation.filter_axes, schema, |_| true),
    }
}

fn projected_reference_columns(
    references: &[ResolvedSemanticReference],
    schema: &DatasetSchema,
    predicate: impl Fn(&ResolvedSemanticReference) -> bool,
) -> Vec<usize> {
    references
        .iter()
        .filter(|reference| predicate(reference))
        .filter_map(|reference| {
            let (_, column_name) = projected_column_name(reference, schema)?;
            schema
                .columns()
                .get_full(&column_name)
                .map(|(index, _, _)| index)
        })
        .collect()
}

fn projected_axes(
    references: &[ResolvedSemanticReference],
    batch: &RecordBatch,
) -> Vec<ProjectedSemanticAxis> {
    references
        .iter()
        .filter_map(|reference| match reference {
            ResolvedSemanticReference::PhysicalColumn(column) => {
                axis_field_column_name(batch, &column.name).map(|column_name| {
                    ProjectedSemanticAxis {
                        id: column.id.clone(),
                        column_name,
                        label: column.label.clone().unwrap_or_else(|| column.name.clone()),
                        semantic_kind: column.semantic_kind,
                        numeric: column.numeric_axis,
                    }
                })
            }
            ResolvedSemanticReference::LogicalIndex(axis) => batch
                .schema()
                .field_with_name(&axis.id)
                .ok()
                .map(|_| ProjectedSemanticAxis {
                    id: axis.id.clone(),
                    column_name: axis.id.clone(),
                    label: axis.label.clone().unwrap_or_else(|| axis.name.clone()),
                    semantic_kind: axis.semantic_kind,
                    numeric: axis.numeric_axis,
                }),
        })
        .collect()
}

fn axis_field_column_name(batch: &RecordBatch, physical_name: &str) -> Option<String> {
    if batch.schema().field_with_name(physical_name).is_ok() {
        return Some(physical_name.to_string());
    }
    let aliased_name = alias_physical_column_name(physical_name);
    batch
        .schema()
        .field_with_name(&aliased_name)
        .ok()
        .map(|_| aliased_name)
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

fn filter_row_indices(row_indices: &[usize], mask: &[bool]) -> Vec<usize> {
    row_indices
        .iter()
        .copied()
        .zip(mask)
        .filter_map(|(row, keep)| (*keep).then_some(row))
        .collect()
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

fn push_unique(columns: &mut Vec<usize>, index: usize) {
    if !columns.contains(&index) {
        columns.push(index);
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::StructArray;
    use arrow_schema::{Field, Schema};

    use super::*;
    use crate::dataset::{
        interpret::{
            ColumnMeaning, PhysicalColumnOrdinal, ResolvedIndexRealization,
            ResolvedPhysicalColumnReference, VisibleColumnOrdinal, resolve_from_manifest,
        },
        semantics::{
            DatasetDType, DatasetSemanticManifest, ManifestColumn, RECORD_ID_COLUMN, ScanAxis,
            ScanAxisValue, ScanPlan,
        },
    };

    fn point(record_id: u64, indices: Vec<u64>) -> ResolvedLogicalIndexPoint {
        ResolvedLogicalIndexPoint {
            record_id,
            indices,
            coordinates: Vec::new(),
        }
    }

    fn empty_interpretation() -> DatasetInterpretation {
        DatasetInterpretation {
            columns: Vec::new(),
            semantic_references: Vec::new(),
            value_references: Vec::new(),
            plotted_coordinates: Vec::new(),
            sweep_axes: Vec::new(),
            group_axes: Vec::new(),
            filter_axes: Vec::new(),
            chart_axis_candidates: Vec::new(),
            value_columns: Vec::new(),
            inferred_axis_columns: Vec::new(),
            chart_axis_candidate_columns: Vec::new(),
            duplicate_policy: ResolvedDuplicatePolicy::LatestByRecordId,
            index_realization: ResolvedIndexRealization::None,
            scan_axes: Vec::new(),
        }
    }

    fn physical_reference(
        id: &str,
        name: &str,
        meaning: ColumnMeaning,
        is_inferred_axis: bool,
    ) -> ResolvedSemanticReference {
        ResolvedSemanticReference::PhysicalColumn(ResolvedPhysicalColumnReference {
            id: id.to_string(),
            name: name.to_string(),
            physical_ordinal: PhysicalColumnOrdinal(0),
            visible_ordinal: Some(VisibleColumnOrdinal(0)),
            dtype: DatasetDType::Float64,
            semantic_kind: ResolvedSemanticKind::Numeric,
            meaning,
            is_system: false,
            is_inferred_axis,
            hidden_by_default: false,
            is_chart_axis_candidate: true,
            unit: None,
            label: None,
            is_complex: false,
            is_trace: false,
            numeric_axis: true,
        })
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
        let interpretation = empty_interpretation();

        let selected = selected_physical_columns(
            &source_schema,
            &interpretation,
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
        let source_schema = DatasetSchema::new(IndexMap::from([(
            "logicalIndex:gate".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        )]));
        let projected = project_schema(&source_schema, &[0]).expect("project schema");

        assert!(projected.columns().contains_key("column:logicalIndex:gate"));
    }

    #[test]
    fn projected_axes_find_unaliased_prefixed_inferred_axis_columns() {
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
        let reference = physical_reference(
            "column:logicalIndex:gate",
            "logicalIndex:gate",
            ColumnMeaning::InferredAxis,
            true,
        );

        let fields = projected_axes(&[reference], &batch);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].id, "column:logicalIndex:gate");
        assert_eq!(fields[0].column_name, "logicalIndex:gate");
    }

    #[test]
    fn roles_use_sweep_axes_not_chart_axis_candidates() {
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
        let mut physical_axis = ManifestColumn::new(DatasetDType::Float64);
        physical_axis.chart_axis = true;
        let manifest = DatasetSemanticManifest::minimal([
            (
                "signal".to_string(),
                ManifestColumn::new(DatasetDType::Float64),
            ),
            ("physicalAxis".to_string(), physical_axis),
        ])
        .with_scan_plan(Some(ScanPlan::new(vec![ScanAxis::static_values(
            "gate",
            vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)],
        )])));
        let arrow_schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
            Field::new("physicalAxis", DataType::Float64, false),
        ]);
        let interpretation = resolve_from_manifest(&arrow_schema, &manifest, &[1, 2]);

        let columns = projected_reference_columns(&interpretation.sweep_axes, &schema, |axis| {
            axis.numeric_axis()
        });

        assert_eq!(columns, vec![2]);
    }

    #[test]
    fn group_roles_include_categorical_logical_axes() {
        let schema = DatasetSchema::new(IndexMap::from([(
            "logicalIndex:gate".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        )]));
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![ScanAxis::static_values(
            "gate",
            vec![
                ScanAxisValue::String("low".to_string()),
                ScanAxisValue::String("high".to_string()),
            ],
        )])));
        let arrow_schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]);
        let interpretation = resolve_from_manifest(&arrow_schema, &manifest, &[1]);

        assert_eq!(
            interpretation.scan_axes[0].semantic_kind,
            ResolvedSemanticKind::Categorical
        );
        assert_eq!(
            projected_reference_columns(&interpretation.group_axes, &schema, |_| true),
            vec![0]
        );
        assert_eq!(
            projected_reference_columns(&interpretation.sweep_axes, &schema, |axis| {
                axis.numeric_axis()
            }),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn logical_axis_projection_preserves_numeric_boolean_and_implicit_kinds() {
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![
            ScanAxis::static_values(
                "gate",
                vec![ScanAxisValue::Float(0.0), ScanAxisValue::Float(1.0)],
            ),
            ScanAxis::static_values(
                "enabled",
                vec![ScanAxisValue::Bool(false), ScanAxisValue::Bool(true)],
            ),
            ScanAxis::implicit_index("step", None),
        ])));
        let arrow_schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]);

        let interpretation = resolve_from_manifest(&arrow_schema, &manifest, &[1]);

        assert_eq!(
            interpretation
                .scan_axes
                .iter()
                .map(|axis| axis.semantic_kind)
                .collect::<Vec<_>>(),
            vec![
                ResolvedSemanticKind::Numeric,
                ResolvedSemanticKind::Boolean,
                ResolvedSemanticKind::Numeric,
            ]
        );
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

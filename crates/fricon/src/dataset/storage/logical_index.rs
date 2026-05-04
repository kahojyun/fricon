use std::{borrow::Cow, sync::Arc};

use arrow_array::{RecordBatch, UInt64Array};
use arrow_schema::{DataType, Field, Schema, SchemaRef};

use crate::dataset::{
    schema::DatasetError,
    semantics::{RECORD_ID_COLUMN, ScanAxisMode, ScanPlan},
    storage::{ChunkReader, error::DatasetFsError, layout::ChunkKind},
};

pub(crate) fn logical_index_values_schema(scan_plan: &ScanPlan) -> SchemaRef {
    Arc::new(Schema::new(
        scan_plan
            .axes
            .iter()
            .map(|axis| Field::new(&axis.name, DataType::UInt64, false))
            .collect::<Vec<_>>(),
    ))
}

pub(crate) fn logical_index_schema(scan_plan: &ScanPlan) -> SchemaRef {
    let mut fields = Vec::with_capacity(scan_plan.axes.len() + 1);
    fields.push(Field::new(RECORD_ID_COLUMN, DataType::UInt64, false));
    fields.extend(
        scan_plan
            .axes
            .iter()
            .map(|axis| Field::new(&axis.name, DataType::UInt64, false)),
    );
    Arc::new(Schema::new(fields))
}

pub(crate) fn build_logical_index_batch(
    scan_plan: &ScanPlan,
    storage_batch: &RecordBatch,
    logical_indices: &RecordBatch,
) -> Result<RecordBatch, DatasetError> {
    let expected_values_schema = logical_index_values_schema(scan_plan);
    if logical_indices.schema() != expected_values_schema
        || logical_indices.num_rows() != storage_batch.num_rows()
    {
        return Err(DatasetError::SchemaMismatch);
    }
    validate_logical_index_bounds(scan_plan, logical_indices)?;

    let record_id_index = storage_batch
        .schema()
        .column_with_name(RECORD_ID_COLUMN)
        .ok_or(DatasetError::SchemaMismatch)?
        .0;
    let record_ids = storage_batch.column(record_id_index).clone();
    if record_ids.as_any().downcast_ref::<UInt64Array>().is_none() {
        return Err(DatasetError::IncompatibleType);
    }

    let mut arrays = Vec::with_capacity(logical_indices.num_columns() + 1);
    arrays.push(record_ids);
    arrays.extend(logical_indices.columns().iter().cloned());
    Ok(RecordBatch::try_new(
        logical_index_schema(scan_plan),
        arrays,
    )?)
}

fn validate_logical_index_bounds(
    scan_plan: &ScanPlan,
    logical_indices: &RecordBatch,
) -> Result<(), DatasetError> {
    for (axis_ordinal, axis) in scan_plan.axes.iter().enumerate() {
        let ScanAxisMode::Static { values } = &axis.mode else {
            continue;
        };
        let len = u64::try_from(values.len()).map_err(|_| DatasetError::IncompatibleType)?;
        let column = logical_indices
            .column(axis_ordinal)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .ok_or(DatasetError::IncompatibleType)?;
        if column.values().iter().any(|index| *index >= len) {
            return Err(DatasetError::InvalidFilter);
        }
    }
    Ok(())
}

pub(crate) fn read_logical_index_batches(
    dataset_path: &std::path::Path,
    scan_plan: &ScanPlan,
) -> Result<Vec<RecordBatch>, DatasetFsError> {
    let mut reader = ChunkReader::new_with_kind(
        dataset_path.to_owned(),
        Some(logical_index_schema(scan_plan)),
        ChunkKind::LogicalIndex,
    );
    reader.read_all()?;
    Ok(reader.range(..).map(Cow::into_owned).collect())
}

pub(crate) fn logical_index_record_ids(batch: &RecordBatch) -> Result<&UInt64Array, DatasetError> {
    batch
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or(DatasetError::IncompatibleType)
}

pub(crate) fn logical_index_column(
    batch: &RecordBatch,
    ordinal: usize,
) -> Result<&UInt64Array, DatasetError> {
    batch
        .column(ordinal + 1)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or(DatasetError::IncompatibleType)
}

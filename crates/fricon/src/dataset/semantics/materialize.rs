use std::sync::Arc;

use arrow_array::{ArrayRef, RecordBatch, UInt64Array};
use arrow_schema::{ArrowError, DataType, Field, Schema, SchemaRef};

use crate::dataset::{
    schema::DatasetError,
    semantics::{ManifestColumn, RECORD_ID_COLUMN, SystemColumn},
};

#[must_use]
pub(crate) fn materialized_schema(user_schema: &Schema) -> SchemaRef {
    let fields: Vec<_> = std::iter::once(Arc::new(Field::new(
        RECORD_ID_COLUMN,
        DataType::UInt64,
        false,
    )))
    .chain(user_schema.fields().iter().cloned())
    .collect();
    Arc::new(Schema::new_with_metadata(
        fields,
        user_schema.metadata().clone(),
    ))
}

pub(crate) fn materialize_record_ids(
    storage_schema: SchemaRef,
    batch: RecordBatch,
    next_record_id: u64,
) -> Result<(RecordBatch, u64), DatasetError> {
    let row_count = u64::try_from(batch.num_rows()).map_err(|_| {
        DatasetError::Arrow(ArrowError::InvalidArgumentError(
            "record batch row count does not fit in uint64".to_string(),
        ))
    })?;
    let end_record_id = next_record_id.checked_add(row_count).ok_or_else(|| {
        DatasetError::Arrow(ArrowError::InvalidArgumentError(
            "dataset record id overflow".to_string(),
        ))
    })?;

    let record_ids: ArrayRef =
        Arc::new(UInt64Array::from_iter_values(next_record_id..end_record_id));
    let columns: Vec<_> = std::iter::once(record_ids)
        .chain(batch.columns().iter().cloned())
        .collect();
    let batch = RecordBatch::try_new(storage_schema, columns)?;
    Ok((batch, end_record_id))
}

#[must_use]
pub(crate) fn is_hidden_system_column(name: &str, column: Option<&ManifestColumn>) -> bool {
    column.is_some_and(|column| column.system == Some(SystemColumn::RecordId))
        || name == RECORD_ID_COLUMN
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Float64Array, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};

    use super::{materialize_record_ids, materialized_schema};
    use crate::dataset::semantics::RECORD_ID_COLUMN;

    fn user_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![Field::new(
            "signal",
            DataType::Float64,
            false,
        )]))
    }

    #[test]
    fn materialized_schema_prepends_record_id() {
        let schema = materialized_schema(user_schema().as_ref());

        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), RECORD_ID_COLUMN);
        assert_eq!(schema.field(0).data_type(), &DataType::UInt64);
        assert!(!schema.field(0).is_nullable());
        assert_eq!(schema.field(1).name(), "signal");
    }

    #[test]
    fn materialize_record_ids_prepends_monotonic_ids() {
        let user_schema = user_schema();
        let storage_schema = materialized_schema(user_schema.as_ref());
        let batch = arrow_array::RecordBatch::try_new(
            user_schema,
            vec![Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0]))],
        )
        .expect("batch");

        let (batch, next) = materialize_record_ids(storage_schema, batch, 7).expect("materialize");

        assert_eq!(next, 10);
        let record_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .expect("record ids");
        assert_eq!(record_ids.values(), &[7, 8, 9]);
    }
}

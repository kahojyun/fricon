use std::{
    collections::{HashMap, HashSet},
    ops::Bound,
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, bail};
use arrow_array::{
    Array, ArrayRef, Float64Array, RecordBatch, StructArray, cast::AsArray, types::Float64Type,
};
use arrow_buffer::NullBuffer;
use arrow_schema::{DataType, Fields, SchemaRef};
use fricon::{DatasetDataType, ScalarKind, SelectOptions};

use crate::application::session::WorkspaceSession;

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Row {
    pub(crate) display_values: Vec<String>,
    pub(crate) value_indices: Vec<usize>,
    pub(crate) index: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ColumnUniqueValue {
    pub(crate) index: usize,
    pub(crate) display_value: String,
}

#[derive(Debug)]
pub(crate) struct TableData {
    pub(crate) fields: Vec<String>,
    pub(crate) rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

pub(crate) struct ProcessedFilterRows {
    pub(crate) unique_rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub(crate) column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

pub(crate) struct DataInternal {
    pub(crate) fields: Vec<String>,
    pub(crate) unique_rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub(crate) column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

impl DataInternal {
    fn empty() -> Self {
        Self {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        }
    }
}

pub(crate) fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn extract_scalar_value(
    array: &dyn Array,
    data_type: DatasetDataType,
    row: usize,
) -> Result<serde_json::Value> {
    if array.is_null(row) {
        return Ok(serde_json::Value::Null);
    }

    match data_type {
        DatasetDataType::Scalar(ScalarKind::Numeric) => Ok(serde_json::Value::from(
            array.as_primitive::<Float64Type>().value(row),
        )),
        DatasetDataType::Scalar(ScalarKind::Complex) => {
            let array = array
                .as_struct_opt()
                .context("Complex filter columns must use Arrow struct arrays")?;
            Ok(serde_json::json!({
                "real": array.column(0).as_primitive::<Float64Type>().value(row),
                "imag": array.column(1).as_primitive::<Float64Type>().value(row),
            }))
        }
        DatasetDataType::Trace(_, _) => {
            bail!("Filter table only supports scalar index columns")
        }
    }
}

fn collect_filter_rows(
    batches: &[RecordBatch],
    field_types: &[DatasetDataType],
) -> Result<Vec<Vec<serde_json::Value>>> {
    let mut rows = Vec::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let row = batch
                .columns()
                .iter()
                .zip(field_types.iter().copied())
                .map(|(column, data_type)| {
                    extract_scalar_value(column.as_ref(), data_type, row_index)
                })
                .collect::<Result<Vec<_>>>()?;
            rows.push(row);
        }
    }
    Ok(rows)
}

fn build_float64_array(value: &serde_json::Value) -> Result<ArrayRef> {
    let value = match value {
        serde_json::Value::Null => None,
        serde_json::Value::Number(number) => Some(
            number
                .as_f64()
                .ok_or_else(|| anyhow!("Filter value is not a valid f64: {number}"))?,
        ),
        other => bail!("Expected numeric filter value, got {other}"),
    };
    Ok(Arc::new(Float64Array::from(vec![value])))
}

fn is_complex_fields(fields: &Fields) -> bool {
    fields.len() == 2
        && fields[0].name() == "real"
        && fields[1].name() == "imag"
        && matches!(fields[0].data_type(), DataType::Float64)
        && matches!(fields[1].data_type(), DataType::Float64)
}

fn build_complex_array(fields: &Fields, value: &serde_json::Value) -> Result<ArrayRef> {
    let (real, imag, nulls) = match value {
        serde_json::Value::Null => (
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Some(NullBuffer::from(vec![false])),
        ),
        serde_json::Value::Object(values) => {
            let read_component = |name: &str| -> Result<f64> {
                values
                    .get(name)
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("Complex filter value is missing '{name}'"))
            };
            (
                Arc::new(Float64Array::from(vec![Some(read_component("real")?)])) as ArrayRef,
                Arc::new(Float64Array::from(vec![Some(read_component("imag")?)])) as ArrayRef,
                None,
            )
        }
        other => bail!("Expected complex filter value object, got {other}"),
    };

    Ok(Arc::new(StructArray::new(
        fields.clone(),
        vec![real, imag],
        nulls,
    )))
}

fn build_filter_array(data_type: &DataType, value: &serde_json::Value) -> Result<ArrayRef> {
    match data_type {
        DataType::Float64 => build_float64_array(value),
        DataType::Struct(fields) if is_complex_fields(fields) => build_complex_array(fields, value),
        other => bail!("Unsupported filter data type: {other}"),
    }
}

pub(crate) async fn load_filter_data(
    session: &WorkspaceSession,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<DataInternal> {
    let dataset = session.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();

    let Some(index_col_indices) = index_columns else {
        return Ok(DataInternal::empty());
    };

    let filtered_indices: Vec<usize> = index_col_indices
        .iter()
        .filter(|&&i| {
            let col_name = schema.columns().keys().nth(i).map(String::as_str);
            if let Some(exclude) = &exclude_columns {
                col_name.is_none_or(|name| !exclude.iter().any(|e| e == name))
            } else {
                true
            }
        })
        .copied()
        .collect();

    if filtered_indices.is_empty() {
        return Ok(DataInternal::empty());
    }

    let fields: Vec<String> = filtered_indices
        .iter()
        .filter_map(|&i| schema.columns().keys().nth(i).cloned())
        .collect();
    let field_types: Vec<DatasetDataType> = filtered_indices
        .iter()
        .filter_map(|&i| schema.columns().values().nth(i).copied())
        .collect();

    let (_, batches) = dataset
        .select_data(&SelectOptions {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
            index_filters: None,
            selected_columns: Some(filtered_indices),
        })
        .context("Failed to select index data")?;

    let rows = collect_filter_rows(&batches, &field_types)?;
    let processed = process_filter_rows(&fields, rows);
    Ok(DataInternal {
        fields,
        unique_rows: processed.unique_rows,
        column_unique_values: processed.column_unique_values,
        column_raw_values: processed.column_raw_values,
    })
}

pub(crate) async fn get_filter_table_data(
    session: &WorkspaceSession,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<TableData> {
    let data = load_filter_data(session, id, exclude_columns).await?;
    Ok(TableData {
        fields: data.fields,
        rows: data.unique_rows,
        column_unique_values: data.column_unique_values,
    })
}

pub(crate) async fn build_filter_batch(
    session: &WorkspaceSession,
    id: i32,
    exclude_columns: Option<Vec<String>>,
    indices: &[usize],
    arrow_schema: SchemaRef,
) -> Result<Option<RecordBatch>> {
    let filter_data = load_filter_data(session, id, exclude_columns).await?;

    let mut selected_filters = Vec::new();
    for (field_index, &value_index) in indices.iter().enumerate() {
        if let Some(field_name) = filter_data.fields.get(field_index)
            && let Some(value) = filter_data
                .column_raw_values
                .get(field_name)
                .and_then(|values| values.get(value_index))
        {
            selected_filters.push((field_name.as_str(), value));
        }
    }

    if selected_filters.is_empty() {
        return Ok(None);
    }

    let projection_indices: Vec<usize> = selected_filters
        .iter()
        .map(|(field_name, _)| {
            arrow_schema
                .index_of(field_name)
                .with_context(|| format!("Field '{field_name}' not found in schema"))
        })
        .collect::<Result<_>>()?;
    let filter_schema = Arc::new(
        arrow_schema
            .project(&projection_indices)
            .context("Failed to project filter schema")?,
    );
    let arrays = filter_schema
        .fields()
        .iter()
        .zip(selected_filters.into_iter())
        .map(|(field, (_, value))| build_filter_array(field.data_type(), value))
        .collect::<Result<Vec<_>>>()?;

    Ok(Some(
        RecordBatch::try_new(filter_schema, arrays).context("Failed to build filter batch")?,
    ))
}

pub(crate) fn process_filter_rows(
    fields: &[String],
    rows: Vec<Vec<serde_json::Value>>,
) -> ProcessedFilterRows {
    let mut unique_rows = Vec::new();
    let mut seen_keys = HashSet::new();
    let mut column_unique_values: HashMap<String, Vec<ColumnUniqueValue>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();
    let mut column_raw_values: HashMap<String, Vec<serde_json::Value>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();

    for (global_row_idx, values) in rows.into_iter().enumerate() {
        let key = serde_json::to_string(&values).unwrap_or_default();

        if !seen_keys.contains(&key) {
            seen_keys.insert(key);
            let display_values = values.iter().map(format_json_value).collect();
            let mut value_indices = Vec::with_capacity(values.len());

            for (col_idx, value) in values.iter().enumerate() {
                if let Some(field_name) = fields.get(col_idx) {
                    let raw_values = column_raw_values
                        .get_mut(field_name)
                        .expect("Field should exist in column_raw_values");

                    let index = if let Some(pos) = raw_values.iter().position(|v| v == value) {
                        pos
                    } else {
                        let new_index = raw_values.len();
                        raw_values.push(value.clone());

                        let display_value = format_json_value(value);
                        column_unique_values
                            .get_mut(field_name)
                            .expect("Field should exist in column_unique_values")
                            .push(ColumnUniqueValue {
                                index: new_index,
                                display_value,
                            });
                        new_index
                    };
                    value_indices.push(index);
                }
            }

            unique_rows.push(Row {
                display_values,
                value_indices,
                index: global_row_idx,
            });
        }
    }
    ProcessedFilterRows {
        unique_rows,
        column_unique_values,
        column_raw_values,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_process_filter_rows() {
        let fields = vec!["col1".to_string(), "col2".to_string()];
        let rows = vec![
            vec![json!("a"), json!(1)],
            vec![json!("a"), json!(1)],
            vec![json!("b"), json!(2)],
        ];
        let processed = process_filter_rows(&fields, rows);

        assert_eq!(processed.unique_rows.len(), 2);
        assert_eq!(processed.unique_rows[0].display_values, vec!["a", "1"]);
        assert_eq!(processed.unique_rows[0].value_indices, vec![0, 0]);
        assert_eq!(processed.unique_rows[1].display_values, vec!["b", "2"]);
        assert_eq!(processed.unique_rows[1].value_indices, vec![1, 1]);
    }

    #[test]
    fn complex_filter_values_keep_stable_display_and_indices() {
        let fields = vec!["z".to_string()];
        let rows = vec![
            vec![json!({"real": 1.0, "imag": 2.0})],
            vec![json!({"real": 1.0, "imag": 2.0})],
            vec![json!({"real": 3.0, "imag": 4.0})],
        ];

        let processed = process_filter_rows(&fields, rows);

        assert_eq!(processed.unique_rows.len(), 2);
        assert_eq!(
            processed.unique_rows[0].display_values,
            vec![r#"{"imag":2.0,"real":1.0}"#]
        );
        assert_eq!(processed.unique_rows[0].value_indices, vec![0]);
        assert_eq!(processed.unique_rows[1].value_indices, vec![1]);
    }
}

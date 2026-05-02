use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    desktop_runtime::session::WorkspaceSession,
    features::charts::{
        semantic_source,
        types::{ColumnUniqueValue, Row, TableData},
    },
};

pub(crate) struct ProcessedFilterRows {
    pub(crate) unique_rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub(crate) column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

pub(crate) struct DataInternal {
    pub(crate) fields: Vec<String>,
    pub(crate) field_labels: HashMap<String, String>,
    pub(crate) unique_rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub(crate) column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

impl DataInternal {
    fn empty() -> Self {
        Self {
            fields: vec![],
            field_labels: HashMap::new(),
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

pub(crate) async fn load_filter_data(
    session: &WorkspaceSession,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<DataInternal> {
    let exclude_columns = exclude_columns.unwrap_or_default();
    let (axis_fields, rows) =
        semantic_source::load_axis_rows(session, id, &exclude_columns).await?;
    if axis_fields.is_empty() {
        return Ok(DataInternal::empty());
    }

    let fields = axis_fields
        .iter()
        .map(|field| field.id.clone())
        .collect::<Vec<_>>();
    let field_labels = axis_fields
        .into_iter()
        .map(|field| (field.id, field.label))
        .collect::<HashMap<_, _>>();
    let processed = process_filter_rows(&fields, rows);
    Ok(DataInternal {
        fields,
        field_labels,
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
        field_labels: data.field_labels,
        rows: data.unique_rows,
        column_unique_values: data.column_unique_values,
    })
}

pub(crate) async fn build_semantic_filters(
    session: &WorkspaceSession,
    id: i32,
    exclude_columns: Option<Vec<String>>,
    indices: &[usize],
) -> Result<Vec<(String, serde_json::Value)>> {
    let filter_data = load_filter_data(session, id, exclude_columns).await?;
    let mut selected_filters = Vec::new();
    for (field_index, &value_index) in indices.iter().enumerate() {
        if let Some(field_name) = filter_data.fields.get(field_index)
            && let Some(value) = filter_data
                .column_raw_values
                .get(field_name)
                .and_then(|values| values.get(value_index))
        {
            selected_filters.push((field_name.clone(), value.clone()));
        }
    }
    Ok(selected_filters)
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

use std::collections::{HashMap, HashSet};

use serde::Serialize;

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub display_values: Vec<String>,
    pub value_indices: Vec<usize>,
    pub index: usize,
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ColumnUniqueValue {
    pub index: usize,
    pub display_value: String,
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TableData {
    pub fields: Vec<String>,
    pub rows: Vec<Row>,
    pub column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

pub struct ProcessedFilterRows {
    pub unique_rows: Vec<Row>,
    pub column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

pub struct DataInternal {
    pub fields: Vec<String>,
    pub unique_rows: Vec<Row>,
    pub column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
    pub column_raw_values: HashMap<String, Vec<serde_json::Value>>,
}

pub fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

pub fn process_filter_rows(
    fields: &[String],
    json_rows: Vec<serde_json::Map<String, serde_json::Value>>,
) -> ProcessedFilterRows {
    let mut unique_rows = Vec::new();
    let mut seen_keys = HashSet::new();
    let mut column_unique_values: HashMap<String, Vec<ColumnUniqueValue>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();
    let mut column_raw_values: HashMap<String, Vec<serde_json::Value>> =
        fields.iter().map(|f| (f.clone(), Vec::new())).collect();

    for (global_row_idx, json_row) in json_rows.into_iter().enumerate() {
        let values: Vec<serde_json::Value> = fields
            .iter()
            .map(|field| {
                json_row
                    .get(field)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            })
            .collect();

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
        let mut row1 = serde_json::Map::new();
        row1.insert("col1".to_string(), json!("a"));
        row1.insert("col2".to_string(), json!(1));

        let mut row2 = serde_json::Map::new();
        row2.insert("col1".to_string(), json!("a"));
        row2.insert("col2".to_string(), json!(1));

        let mut row3 = serde_json::Map::new();
        row3.insert("col1".to_string(), json!("b"));
        row3.insert("col2".to_string(), json!(2));

        let json_rows = vec![row1, row2, row3];
        let processed = process_filter_rows(&fields, json_rows);

        assert_eq!(processed.unique_rows.len(), 2);
        assert_eq!(processed.unique_rows[0].display_values, vec!["a", "1"]);
        assert_eq!(processed.unique_rows[0].value_indices, vec![0, 0]);
        assert_eq!(processed.unique_rows[1].display_values, vec!["b", "2"]);
        assert_eq!(processed.unique_rows[1].value_indices, vec![1, 1]);
    }
}

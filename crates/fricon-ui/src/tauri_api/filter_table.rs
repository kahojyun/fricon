use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::application::filter_table as app;

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Row {
    pub(crate) display_values: Vec<String>,
    pub(crate) value_indices: Vec<usize>,
    pub(crate) index: usize,
}

impl From<app::Row> for Row {
    fn from(value: app::Row) -> Self {
        Self {
            display_values: value.display_values,
            value_indices: value.value_indices,
            index: value.index,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColumnUniqueValue {
    pub(crate) index: usize,
    pub(crate) display_value: String,
}

impl From<app::ColumnUniqueValue> for ColumnUniqueValue {
    fn from(value: app::ColumnUniqueValue) -> Self {
        Self {
            index: value.index,
            display_value: value.display_value,
        }
    }
}

#[derive(Serialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TableData {
    pub(crate) fields: Vec<String>,
    pub(crate) rows: Vec<Row>,
    pub(crate) column_unique_values: HashMap<String, Vec<ColumnUniqueValue>>,
}

impl From<app::TableData> for TableData {
    fn from(value: app::TableData) -> Self {
        Self {
            fields: value.fields,
            rows: value.rows.into_iter().map(Into::into).collect(),
            column_unique_values: value
                .column_unique_values
                .into_iter()
                .map(|(key, values)| (key, values.into_iter().map(Into::into).collect()))
                .collect(),
        }
    }
}

#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilterTableOptions {
    #[specta(optional)]
    pub(crate) exclude_columns: Option<Vec<String>>,
}

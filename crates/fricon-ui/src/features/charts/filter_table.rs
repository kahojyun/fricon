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
    use fricon::{
        AppManager, Client, ColumnMetadata, DatasetRow, DatasetScalar, ScanAxis, ScanAxisValue,
        ScanPlan, WorkspaceRoot,
    };
    use indexmap::IndexMap;
    use serde_json::json;
    use tempfile::TempDir;

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

    #[tokio::test]
    async fn filter_data_includes_nonnumeric_logical_scan_axes() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager =
            AppManager::new_with_path(temp_dir.path())?.start(&tokio::runtime::Handle::current())?;
        let client = Client::connect(temp_dir.path()).await?;

        let rows = [10.0, 20.0, 30.0, 40.0]
            .into_iter()
            .map(|signal| {
                DatasetRow(IndexMap::from([(
                    "signal".to_string(),
                    DatasetScalar::Numeric(signal),
                )]))
            })
            .collect::<Vec<_>>();
        let scan_plan = ScanPlan::new(vec![
            ScanAxis::static_values(
                "gate",
                vec![
                    ScanAxisValue::String("low".to_string()),
                    ScanAxisValue::String("high".to_string()),
                ],
            ),
            ScanAxis::static_values("bias", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
        ]);
        let mut writer = client
            .create_dataset(
                "scan-filter-test".to_string(),
                String::new(),
                vec![],
                rows[0].to_schema(),
                Vec::new(),
                Some(scan_plan),
                false,
            )
            .await?;
        for row in rows {
            writer.write(row).await?;
        }
        let dataset = writer.finish().await?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let data = load_filter_data(&session, dataset.id(), None).await?;

        assert_eq!(data.fields, vec!["logicalIndex:gate", "logicalIndex:bias"]);
        assert_eq!(
            data.unique_rows
                .iter()
                .map(|row| row.display_values.clone())
                .collect::<Vec<_>>(),
            vec![
                vec!["low".to_string(), "0.0".to_string()],
                vec!["low".to_string(), "1.0".to_string()],
                vec!["high".to_string(), "0.0".to_string()],
                vec!["high".to_string(), "1.0".to_string()],
            ]
        );

        app_manager.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn filter_data_includes_inferred_axes_for_simple_semantic_datasets() -> anyhow::Result<()>
    {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager =
            AppManager::new_with_path(temp_dir.path())?.start(&tokio::runtime::Handle::current())?;
        let client = Client::connect(temp_dir.path()).await?;

        let rows = (1..=3)
            .map(|row_id| {
                DatasetRow(IndexMap::from([
                    (
                        "row_id".to_string(),
                        DatasetScalar::Numeric(f64::from(row_id)),
                    ),
                    (
                        "signal".to_string(),
                        DatasetScalar::Numeric(f64::from(row_id * 10)),
                    ),
                ]))
            })
            .collect::<Vec<_>>();
        let mut writer = client
            .create_dataset(
                "simple-filter-test".to_string(),
                String::new(),
                vec![],
                rows[0].to_schema(),
                Vec::new(),
                None,
                false,
            )
            .await?;
        for row in rows {
            writer.write(row).await?;
        }
        let dataset = writer.finish().await?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let data = load_filter_data(&session, dataset.id(), None).await?;

        assert_eq!(data.fields, vec!["column:row_id"]);
        assert_eq!(
            data.unique_rows
                .iter()
                .map(|row| row.display_values.clone())
                .collect::<Vec<_>>(),
            vec![
                vec!["1.0".to_string()],
                vec!["2.0".to_string()],
                vec!["3.0".to_string()],
            ]
        );

        app_manager.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn filter_data_includes_manifest_chart_axis_candidates() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager =
            AppManager::new_with_path(temp_dir.path())?.start(&tokio::runtime::Handle::current())?;
        let client = Client::connect(temp_dir.path()).await?;

        let rows = [("low", 295.0, 10.0), ("high", 300.0, 20.0)]
            .into_iter()
            .map(|(_, temperature, signal)| {
                DatasetRow(IndexMap::from([
                    (
                        "temperature".to_string(),
                        DatasetScalar::Numeric(temperature),
                    ),
                    ("signal".to_string(), DatasetScalar::Numeric(signal)),
                ]))
            })
            .collect::<Vec<_>>();
        let scan_plan = ScanPlan::new(vec![ScanAxis::static_values(
            "gate",
            vec![
                ScanAxisValue::String("low".to_string()),
                ScanAxisValue::String("high".to_string()),
            ],
        )]);
        let mut writer = client
            .create_dataset(
                "manifest-filter-axis-test".to_string(),
                String::new(),
                vec![],
                rows[0].to_schema(),
                vec![ColumnMetadata {
                    name: "temperature".to_string(),
                    unit: Some("K".to_string()),
                    label: Some("Temperature".to_string()),
                    hidden_by_default: false,
                    chart_axis: true,
                }],
                Some(scan_plan),
                false,
            )
            .await?;
        for row in rows {
            writer.write(row).await?;
        }
        let dataset = writer.finish().await?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let data = load_filter_data(&session, dataset.id(), None).await?;

        assert_eq!(data.fields, vec!["logicalIndex:gate", "column:temperature"]);
        assert_eq!(data.field_labels["column:temperature"], "Temperature");
        assert_eq!(
            data.unique_rows
                .iter()
                .map(|row| row.display_values.clone())
                .collect::<Vec<_>>(),
            vec![
                vec!["low".to_string(), "295.0".to_string()],
                vec!["high".to_string(), "300.0".to_string()],
            ]
        );

        app_manager.shutdown().await;
        Ok(())
    }
}

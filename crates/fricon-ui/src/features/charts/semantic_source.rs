use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use arrow_array::{Array, BooleanArray, Float64Array, RecordBatch, StringArray, StructArray};
use arrow_schema::{DataType, Fields};
use fricon::{
    DatasetSchema, ProjectedSemanticAxis, ProjectedSemanticSource, SemanticProjectionOptions,
    project_semantic_source,
};

use super::types::ChartCommonOptions;
use crate::desktop_runtime::session::WorkspaceSession;

pub(crate) type AxisField = ProjectedSemanticAxis;

#[derive(Debug)]
pub(crate) struct PreparedChartData {
    pub(crate) batch: RecordBatch,
    pub(crate) schema: DatasetSchema,
    pub(crate) index_columns: Option<Vec<usize>>,
    pub(crate) group_columns: Option<Vec<usize>>,
    pub(crate) row_indices: Vec<usize>,
    semantic_column_names: HashMap<String, String>,
}

impl PreparedChartData {
    pub(crate) fn resolve_column_name(&self, value: &str) -> String {
        self.semantic_column_names
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.to_string())
    }
}

fn prepared_chart_data(projection: ProjectedSemanticSource) -> PreparedChartData {
    let index_columns = projection.sweep_columns().map(<[_]>::to_vec);
    let group_columns = projection.group_columns().map(<[_]>::to_vec);
    PreparedChartData {
        batch: projection.batch,
        schema: projection.schema,
        index_columns,
        group_columns,
        row_indices: projection.row_indices,
        semantic_column_names: projection.semantic_column_names,
    }
}

pub(crate) async fn prepare_chart_data(
    session: &WorkspaceSession,
    id: i32,
    common: &ChartCommonOptions,
    filters: &[(String, serde_json::Value)],
    selected_columns: Option<&[usize]>,
) -> Result<PreparedChartData> {
    let dataset = session.dataset(id).await?;
    let projection = project_semantic_source(
        &dataset,
        &SemanticProjectionOptions {
            start: common.start,
            end: common.end,
            filters,
            selected_columns,
        },
    )?;
    Ok(prepared_chart_data(projection))
}

pub(crate) async fn load_axis_rows(
    session: &WorkspaceSession,
    id: i32,
    exclude_fields: &[String],
) -> Result<(Vec<AxisField>, Vec<Vec<serde_json::Value>>)> {
    let dataset = session.dataset(id).await?;
    let end = dataset.num_rows();
    let projection = project_semantic_source(
        &dataset,
        &SemanticProjectionOptions {
            start: Some(0),
            end: Some(end),
            filters: &[],
            selected_columns: None,
        },
    )?;

    let fields = projection
        .group_axes
        .into_iter()
        .filter(|field| !exclude_fields.iter().any(|excluded| excluded == &field.id))
        .collect::<Vec<_>>();

    let rows = (0..projection.batch.num_rows())
        .map(|row| {
            fields
                .iter()
                .map(|field| {
                    let column = projection
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{ArrayRef, StructArray};
    use arrow_schema::{Field, Schema};
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn prepared_data_resolves_core_semantic_column_names() {
        let batch = RecordBatch::new_empty(Arc::new(Schema::empty()));
        let prepared = PreparedChartData {
            batch,
            schema: DatasetSchema::new(IndexMap::new()),
            index_columns: None,
            group_columns: None,
            row_indices: Vec::new(),
            semantic_column_names: HashMap::from([(
                "column:logicalIndex:gate".to_string(),
                "column:logicalIndex:gate".to_string(),
            )]),
        };

        assert_eq!(
            prepared.resolve_column_name("column:logicalIndex:gate"),
            "column:logicalIndex:gate"
        );
        assert_eq!(prepared.resolve_column_name("plain"), "plain");
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

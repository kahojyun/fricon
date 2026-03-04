use std::{collections::HashMap, io::Cursor, ops::Bound, sync::Arc};

use anyhow::Context;
use fricon::SelectOptions;
use tauri::State;
use tracing::instrument;

use super::{AppState, TauriCommandError};
use crate::models::filter::{DataInternal, TableData, process_filter_rows};

#[derive(serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub(super) struct FilterTableOptions {
    #[specta(optional)]
    exclude_columns: Option<Vec<String>>,
}

#[instrument(level = "debug", skip(state, exclude_columns), fields(dataset_id = id))]
async fn get_filter_data_internal(
    state: &AppState,
    id: i32,
    exclude_columns: Option<Vec<String>>,
) -> Result<DataInternal, TauriCommandError> {
    let dataset = state.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();

    let Some(index_col_indices) = index_columns else {
        return Ok(DataInternal {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        });
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
        return Ok(DataInternal {
            fields: vec![],
            unique_rows: vec![],
            column_unique_values: HashMap::new(),
            column_raw_values: HashMap::new(),
        });
    }

    let fields: Vec<String> = filtered_indices
        .iter()
        .filter_map(|&i| schema.columns().keys().nth(i).cloned())
        .collect();

    let (_, batches) = dataset
        .select_data(&SelectOptions {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
            index_filters: None,
            selected_columns: Some(filtered_indices.clone()),
        })
        .context("Failed to select index data.")?;

    let mut buf = Vec::new();
    {
        let mut writer = arrow_json::ArrayWriter::new(&mut buf);
        for batch in &batches {
            writer.write(batch).context("Failed to write batch")?;
        }
        writer.finish().context("Failed to finish JSON writer")?;
    }
    let json_rows: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_slice(&buf).context("Failed to parse JSON")?;

    let processed = process_filter_rows(&fields, json_rows);
    Ok(DataInternal {
        fields,
        unique_rows: processed.unique_rows,
        column_unique_values: processed.column_unique_values,
        column_raw_values: processed.column_raw_values,
    })
}

#[instrument(
    level = "debug",
    skip(state, exclude_columns, indices, arrow_schema),
    fields(dataset_id = id)
)]
pub(in crate::commands) async fn build_filter_batch(
    state: &AppState,
    id: i32,
    exclude_columns: Option<Vec<String>>,
    indices: &[usize],
    arrow_schema: Arc<arrow_schema::Schema>,
) -> Result<Option<arrow_array::RecordBatch>, TauriCommandError> {
    let filter_data = get_filter_data_internal(state, id, exclude_columns).await?;
    let fields = filter_data.fields;
    let raw_values_map = filter_data.column_raw_values;

    let mut filter_map = serde_json::Map::new();
    for (idx, &value_idx) in indices.iter().enumerate() {
        if let Some(field_name) = fields.get(idx)
            && let Some(val) = raw_values_map
                .get(field_name)
                .and_then(|values| values.get(value_idx))
        {
            filter_map.insert(field_name.clone(), val.clone());
        }
    }

    if filter_map.is_empty() {
        return Ok(None);
    }

    let projection_indices: Vec<usize> = filter_map
        .keys()
        .map(|field_name| {
            arrow_schema
                .index_of(field_name)
                .with_context(|| format!("Field '{field_name}' not found in schema"))
        })
        .collect::<Result<_, _>>()?;
    let filter_schema = Arc::new(
        arrow_schema
            .project(&projection_indices)
            .context("Failed to project arrow schema")?,
    );

    let json_row = serde_json::Value::Object(filter_map);
    let json_array = serde_json::to_vec(&json_row).context("Failed to serialize filter to JSON")?;

    let mut reader = arrow_json::ReaderBuilder::new(filter_schema)
        .build(Cursor::new(json_array))
        .context("Failed to create JSON reader")?;

    let batch = reader
        .next()
        .context("No batch returned")?
        .context("Failed to decode filter batch")?;

    Ok(Some(batch))
}

#[tauri::command]
#[specta::specta]
pub(super) async fn get_filter_table_data(
    state: State<'_, AppState>,
    id: i32,
    options: FilterTableOptions,
) -> Result<TableData, TauriCommandError> {
    let filter_data = get_filter_data_internal(&state, id, options.exclude_columns).await?;

    Ok(TableData {
        fields: filter_data.fields,
        rows: filter_data.unique_rows,
        column_unique_values: filter_data.column_unique_values,
    })
}

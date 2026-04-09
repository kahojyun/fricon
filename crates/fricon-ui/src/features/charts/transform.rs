pub(crate) mod heatmap;
pub(crate) mod live_heatmap;
pub(crate) mod live_xy;
pub(crate) mod mapping;
pub(crate) mod xy;

use anyhow::{Context, Result, bail};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetSchema};

pub(crate) use self::{
    heatmap::build_heatmap_series, live_heatmap::build_live_heatmap_series,
    live_xy::build_live_xy_series, xy::build_xy_series,
};
use crate::features::charts::types::{XYDrawStyle, XYTraceRoleOptions};

pub(super) struct XYTraceRoles {
    pub(super) trace_group: Vec<usize>,
    pub(super) sweep: Option<usize>,
}

pub(super) fn row_series_id(row: usize) -> String {
    format!("row:{row}")
}

pub(super) fn group_series_id(group_start: usize) -> String {
    format!("group:{group_start}")
}

pub(super) fn resolve_xy_trace_roles(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &XYTraceRoleOptions,
    draw_style: XYDrawStyle,
) -> Result<XYTraceRoles> {
    let Some(index_columns) = index_columns else {
        if options.sweep_index_column.is_some()
            || options
                .trace_group_index_columns
                .as_ref()
                .is_some_and(|columns| !columns.is_empty())
        {
            bail!("Trace roles require dataset index columns");
        }
        return Ok(XYTraceRoles {
            trace_group: vec![],
            sweep: None,
        });
    };

    let trace_group = resolve_named_index_columns(
        schema,
        index_columns,
        options.trace_group_index_columns.as_deref().unwrap_or(&[]),
    )?;

    let explicit_sweep = options
        .sweep_index_column
        .as_deref()
        .map(|name| resolve_named_index_column(schema, index_columns, name))
        .transpose()?;

    if explicit_sweep.is_some_and(|sweep| trace_group.contains(&sweep)) {
        bail!("sweepIndexColumn must not also be used in traceGroupIndexColumns");
    }

    let default_sweep = if draw_style.includes_lines() {
        index_columns
            .iter()
            .rev()
            .find(|&&index| !trace_group.contains(&index))
            .copied()
    } else {
        None
    };

    Ok(XYTraceRoles {
        trace_group,
        sweep: explicit_sweep.or(default_sweep),
    })
}

pub(super) fn compute_group_starts(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    group_columns: &[usize],
) -> Vec<usize> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return vec![];
    }
    if group_columns.is_empty() {
        return vec![0];
    }

    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let group_values: Vec<Vec<f64>> = group_columns
        .iter()
        .map(|&idx| {
            let arr = batch
                .column_by_name(column_names[idx])
                .expect("group column present");
            let ds: DatasetArray = arr.clone().try_into().expect("valid group column");
            ds.as_numeric()
                .expect("numeric group column")
                .values()
                .to_vec()
        })
        .collect();

    let mut group_starts = vec![0];
    for row in 1..num_rows {
        #[expect(
            clippy::float_cmp,
            reason = "Index values are stored, not computed; exact comparison is correct"
        )]
        if group_values.iter().any(|col| col[row] != col[row - 1]) {
            group_starts.push(row);
        }
    }
    group_starts
}

pub(super) fn group_ranges(starts: &[usize], num_rows: usize) -> Vec<(usize, usize)> {
    starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = starts.get(i + 1).copied().unwrap_or(num_rows);
            (start, end)
        })
        .collect()
}

pub(super) fn last_outer_group_start(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: &[usize],
    outer_index_count: usize,
) -> usize {
    let num_rows = batch.num_rows();
    if outer_index_count == 0 || num_rows == 0 {
        return 0;
    }

    let outer_indices = &index_columns[..outer_index_count];
    let starts = compute_group_starts(batch, schema, outer_indices);
    starts.last().copied().unwrap_or(0)
}

pub(super) fn row_order_for_group(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    start: usize,
    end: usize,
    sweep: Option<usize>,
) -> Vec<usize> {
    let mut rows: Vec<usize> = (start..end).collect();
    let Some(sweep) = sweep else {
        return rows;
    };

    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let arr = batch
        .column_by_name(column_names[sweep])
        .expect("order column present");
    let ds: DatasetArray = arr.clone().try_into().expect("valid order column");
    let values = ds.as_numeric().expect("numeric order column").values();

    rows.sort_by(|&left, &right| {
        let ordering = values[left].total_cmp(&values[right]);
        if ordering.is_eq() {
            left.cmp(&right)
        } else {
            ordering
        }
    });
    rows
}

pub(super) fn make_group_label(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    group_columns: &[usize],
    row: usize,
) -> Option<String> {
    if group_columns.is_empty() {
        return None;
    }

    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let parts = group_columns
        .iter()
        .map(|&idx| {
            let name = column_names[idx];
            let arr = batch.column_by_name(name).expect("group column present");
            let ds: DatasetArray = arr.clone().try_into().expect("valid group column");
            let value = ds.as_numeric().expect("numeric group column").values()[row];
            format!("{name}={}", format_numeric_value(value))
        })
        .collect::<Vec<_>>();

    Some(parts.join(", "))
}

pub(super) fn make_group_id_suffix(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    group_columns: &[usize],
    row: usize,
) -> Option<String> {
    make_group_label(batch, schema, group_columns, row)
        .map(|label| label.replace(", ", "|").replace('=', ":"))
}

pub(super) fn format_numeric_value(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn resolve_named_index_columns(
    schema: &DatasetSchema,
    index_columns: &[usize],
    names: &[String],
) -> Result<Vec<usize>> {
    let mut resolved = Vec::new();
    for name in names {
        let index = resolve_named_index_column(schema, index_columns, name)?;
        if !resolved.contains(&index) {
            resolved.push(index);
        }
    }
    resolved.sort_by_key(|index| {
        index_columns
            .iter()
            .position(|candidate| candidate == index)
            .expect("resolved index is present in index_columns")
    });
    Ok(resolved)
}

fn resolve_named_index_column(
    schema: &DatasetSchema,
    index_columns: &[usize],
    name: &str,
) -> Result<usize> {
    let (idx, _, _) = schema
        .columns()
        .get_full(name)
        .with_context(|| format!("Column '{name}' not found"))?;
    if !index_columns.contains(&idx) {
        bail!("Column '{name}' is not an index column");
    }
    Ok(idx)
}

#[cfg(test)]
pub(super) mod test_utils {
    use std::sync::Arc;

    use arrow_array::{Float64Array, RecordBatch};
    use arrow_schema::{DataType, Field};
    use fricon::{DatasetDataType, DatasetSchema, ScalarKind};
    use indexmap::IndexMap;

    /// Build a `RecordBatch` from named Float64 columns.
    pub(crate) fn numeric_batch(columns: &[(&str, &[f64])]) -> RecordBatch {
        let fields: Vec<_> = columns
            .iter()
            .map(|(name, _)| Field::new(*name, DataType::Float64, false))
            .collect();
        let arrays: Vec<Arc<dyn arrow_array::Array>> = columns
            .iter()
            .map(|(_, vals)| Arc::new(Float64Array::from(vals.to_vec())) as _)
            .collect();
        RecordBatch::try_new(Arc::new(arrow_schema::Schema::new(fields)), arrays).unwrap()
    }

    /// Build a `DatasetSchema` where every column is `Scalar(Numeric)`.
    pub(crate) fn numeric_schema(names: &[&str]) -> DatasetSchema {
        let columns: IndexMap<String, DatasetDataType> = names
            .iter()
            .map(|n| (n.to_string(), DatasetDataType::Scalar(ScalarKind::Numeric)))
            .collect();
        DatasetSchema::new(columns)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_group_starts, group_ranges, last_outer_group_start, resolve_xy_trace_roles,
        test_utils::{numeric_batch, numeric_schema},
    };
    use crate::features::charts::types::{XYDrawStyle, XYTraceRoleOptions};

    #[test]
    fn compute_group_starts_for_named_groups() {
        let batch = numeric_batch(&[
            ("outer", &[1.0, 1.0, 1.0, 2.0, 2.0]),
            ("inner", &[10.0, 20.0, 30.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["outer", "inner"]);
        let groups = compute_group_starts(&batch, &schema, &[0]);
        assert_eq!(groups, vec![0, 3]);
        assert_eq!(
            group_ranges(&groups, batch.num_rows()),
            vec![(0, 3), (3, 5)]
        );
    }

    #[test]
    fn compute_group_starts_empty_group_columns_returns_whole_batch() {
        let batch = numeric_batch(&[("idx", &[1.0, 2.0, 3.0])]);
        let schema = numeric_schema(&["idx"]);
        assert_eq!(compute_group_starts(&batch, &schema, &[]), vec![0]);
    }

    #[test]
    fn last_outer_group_start_finds_boundary() {
        let batch = numeric_batch(&[
            ("outer", &[1.0, 1.0, 2.0, 2.0, 3.0, 3.0]),
            ("inner", &[10.0, 20.0, 10.0, 20.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["outer", "inner"]);
        assert_eq!(last_outer_group_start(&batch, &schema, &[0, 1], 1), 4);
    }

    #[test]
    fn resolve_xy_trace_roles_uses_explicit_group_and_default_sweep() {
        let schema = numeric_schema(&["outer", "middle", "inner"]);
        let roles = resolve_xy_trace_roles(
            &schema,
            Some(&[0, 1, 2]),
            &XYTraceRoleOptions {
                trace_group_index_columns: Some(vec!["outer".to_string()]),
                sweep_index_column: None,
            },
            XYDrawStyle::Line,
        )
        .unwrap();

        assert_eq!(roles.trace_group, vec![0]);
        assert_eq!(roles.sweep, Some(2));
    }

    #[test]
    fn resolve_xy_trace_roles_rejects_overlap() {
        let schema = numeric_schema(&["outer", "inner"]);
        let result = resolve_xy_trace_roles(
            &schema,
            Some(&[0, 1]),
            &XYTraceRoleOptions {
                trace_group_index_columns: Some(vec!["inner".to_string()]),
                sweep_index_column: Some("inner".to_string()),
            },
            XYDrawStyle::Line,
        );

        assert!(result.is_err());
    }
}

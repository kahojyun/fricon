pub(crate) mod heatmap;
pub(crate) mod line;
pub(crate) mod live_heatmap;
pub(crate) mod live_line;
pub(crate) mod live_scatter;
pub(crate) mod mapping;
pub(crate) mod scatter;

use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetSchema};

pub(crate) use self::{
    heatmap::build_heatmap_series, line::build_line_series,
    live_heatmap::build_live_heatmap_series, live_line::build_live_line_series,
    live_scatter::build_live_scatter_series, scatter::build_scatter_series,
};

pub(super) fn row_series_id(row: usize) -> String {
    format!("row:{row}")
}

pub(super) fn group_series_id(group_start: usize) -> String {
    format!("group:{group_start}")
}

/// Compute sweep group start indices based on outer index column transitions.
///
/// Outer indices are all index columns except the most-frequent (last) one.
/// A new group starts when any outer index value changes from the previous row.
/// Falls back to one-row-per-group when there are fewer than two index columns.
pub(super) fn compute_sweep_groups(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
) -> Vec<usize> {
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return vec![];
    }

    if let Some(idx_cols) = index_columns
        && idx_cols.len() >= 2
    {
        let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
        let outer_indices = &idx_cols[..idx_cols.len() - 1];
        let outer_columns: Vec<Vec<f64>> = outer_indices
            .iter()
            .map(|&idx| {
                let arr = batch
                    .column_by_name(column_names[idx])
                    .expect("index column present");
                let ds: DatasetArray = arr.clone().try_into().expect("valid array");
                ds.as_numeric().expect("numeric index").values().to_vec()
            })
            .collect();

        let mut group_starts = vec![0_usize];
        for row in 1..num_rows {
            #[expect(
                clippy::float_cmp,
                reason = "Index values are stored, not computed; exact comparison is correct"
            )]
            if outer_columns.iter().any(|col| col[row] != col[row - 1]) {
                group_starts.push(row);
            }
        }
        group_starts
    } else {
        // No meaningful grouping: each row is its own "sweep"
        (0..num_rows).collect()
    }
}

/// Find the start row of the last outer-index group.
///
/// Like [`compute_sweep_groups`] but only needs the final boundary, so it
/// avoids allocating the full group list.
///
/// `outer_index_count` specifies how many leading index columns form the
/// "outer" key. When `outer_index_count == 0` the whole batch is the last group
/// (returns 0).
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

    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let outer_indices = &index_columns[..outer_index_count];
    let outer_columns: Vec<Vec<f64>> = outer_indices
        .iter()
        .map(|&idx| {
            let arr = batch
                .column_by_name(column_names[idx])
                .expect("index column present");
            let ds: DatasetArray = arr.clone().try_into().expect("valid array");
            ds.as_numeric().expect("numeric index").values().to_vec()
        })
        .collect();

    let mut last_group_start = 0;
    for row in 1..num_rows {
        #[expect(
            clippy::float_cmp,
            reason = "Index values are stored, not computed; exact comparison is correct"
        )]
        if outer_columns.iter().any(|col| col[row] != col[row - 1]) {
            last_group_start = row;
        }
    }
    last_group_start
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
        test_utils::{numeric_batch, numeric_schema},
        *,
    };

    #[test]
    fn compute_sweep_groups_with_two_indices() {
        let batch = numeric_batch(&[
            ("outer", &[1.0, 1.0, 1.0, 2.0, 2.0]),
            ("inner", &[10.0, 20.0, 30.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["outer", "inner"]);
        let groups = compute_sweep_groups(&batch, &schema, Some(&[0, 1]));
        assert_eq!(groups, vec![0, 3]);
    }

    #[test]
    fn compute_sweep_groups_single_index_falls_back_to_per_row() {
        let batch = numeric_batch(&[("idx", &[1.0, 2.0, 3.0])]);
        let schema = numeric_schema(&["idx"]);
        let groups = compute_sweep_groups(&batch, &schema, Some(&[0]));
        assert_eq!(groups, vec![0, 1, 2]);
    }

    #[test]
    fn compute_sweep_groups_none_falls_back_to_per_row() {
        let batch = numeric_batch(&[("x", &[1.0, 2.0])]);
        let schema = numeric_schema(&["x"]);
        let groups = compute_sweep_groups(&batch, &schema, None);
        assert_eq!(groups, vec![0, 1]);
    }

    #[test]
    fn compute_sweep_groups_empty_batch() {
        let batch = numeric_batch(&[("x", &[])]);
        let schema = numeric_schema(&["x"]);
        let groups = compute_sweep_groups(&batch, &schema, Some(&[0]));
        assert!(groups.is_empty());
    }

    #[test]
    fn last_outer_group_start_finds_boundary() {
        let batch = numeric_batch(&[
            ("outer", &[1.0, 1.0, 2.0, 2.0, 3.0, 3.0]),
            ("inner", &[10.0, 20.0, 10.0, 20.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["outer", "inner"]);
        // outer_index_count = 1 (just "outer"), index_columns = [0, 1]
        assert_eq!(last_outer_group_start(&batch, &schema, &[0, 1], 1), 4);
    }

    #[test]
    fn last_outer_group_start_zero_outer_returns_zero() {
        let batch = numeric_batch(&[("x", &[1.0, 2.0, 3.0])]);
        let schema = numeric_schema(&["x"]);
        assert_eq!(last_outer_group_start(&batch, &schema, &[0], 0), 0);
    }
}

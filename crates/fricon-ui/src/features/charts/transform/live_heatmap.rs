use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::{heatmap::build_heatmap_series, last_outer_group_start};
use crate::features::charts::types::{
    ChartCommonOptions, ChartSnapshot, ComplexViewOption, FlatXYZSeries, HeatmapChartDataOptions,
    HeatmapChartSnapshot, LiveHeatmapOptions, complex_view_label, transform_complex_values,
};

/// Build a live heatmap showing only the latest sweep of data.
///
/// For **trace** data the last row is expanded: the trace's own x-axis becomes
/// the heatmap X, and there is no Y grouping (single row).
///
/// For **scalar** data we need at least two index columns. The most-frequent
/// index (last) becomes the heatmap X-axis, the second-most-frequent index
/// (second-to-last) becomes the Y-axis, and the series values become Z.
/// Only rows belonging to the very last outer-index group are included.
pub(crate) fn build_live_heatmap_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveHeatmapOptions,
) -> Result<ChartSnapshot> {
    let quantity_name = &options.quantity;
    let data_type = *schema
        .columns()
        .get(quantity_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));

    debug!(
        chart_type = "live_heatmap",
        quantity = %quantity_name,
        ?data_type,
        rows = batch.num_rows(),
        "Building live heatmap chart series"
    );

    if is_trace {
        return build_trace_live_heatmap(batch, schema, index_columns, options);
    }

    // Scalar path: need at least two index columns (mfi for X, second-mfi for Y)
    let idx_cols = index_columns
        .filter(|c| c.len() >= 2)
        .context("Live heatmap requires at least two index columns")?;

    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let mfi_idx = *idx_cols.last().context("No index columns")?;
    let second_mfi_idx = idx_cols[idx_cols.len() - 2];
    let mfi_name = column_names[mfi_idx];
    let second_mfi_name = column_names[second_mfi_idx];

    // Crop to the last outer-index group (outermost indices, excluding the two
    // most-frequent ones).
    let outer_count = idx_cols.len().saturating_sub(2);
    let num_rows = batch.num_rows();
    let start = last_outer_group_start(batch, schema, idx_cols, outer_count);

    let cropped = batch.slice(start, num_rows - start);

    // Delegate to the normal heatmap builder with MFI as x, second-MFI as y
    let heatmap_options = HeatmapChartDataOptions {
        quantity: quantity_name.clone(),
        x_column: Some(mfi_name.to_string()),
        y_column: second_mfi_name.to_string(),
        complex_view_single: options.complex_view_single,
        common: ChartCommonOptions::default(),
    };

    build_heatmap_series(&cropped, schema, &heatmap_options)
}

/// Trace live heatmap: expand the last row's trace and plot it.
/// If there is an index column, use it as the Y-axis label per row.
fn build_trace_live_heatmap(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveHeatmapOptions,
) -> Result<ChartSnapshot> {
    let quantity_name = &options.quantity;
    let data_type = *schema
        .columns()
        .get(quantity_name)
        .context("Column not found")?;
    let is_complex = data_type.is_complex();
    let view_option = options
        .complex_view_single
        .unwrap_or(ComplexViewOption::Mag);

    // For traces, fall back to showing just the last row (or last sweep group)
    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let y_column_name = index_columns
        .and_then(|cols| cols.last())
        .map(|&idx| column_names[idx]);
    let y_name = y_column_name.unwrap_or("row").to_string();
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok(ChartSnapshot::Heatmap(HeatmapChartSnapshot {
            x_name: format!("{quantity_name} - X"),
            y_name,
            series: vec![],
        }));
    }

    // Crop to last group if we have outer indices (more than 1 index col)
    let start = if let Some(idx_cols) = index_columns
        && idx_cols.len() >= 2
    {
        let outer_count = idx_cols.len() - 1;
        last_outer_group_start(batch, schema, idx_cols, outer_count)
    } else {
        0
    };

    let series_array: DatasetArray = batch
        .column_by_name(quantity_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;

    let y_values: Option<Vec<f64>> = y_column_name.map(|name| {
        let arr = batch.column_by_name(name).expect("y column present");
        let ds: DatasetArray = arr.clone().try_into().expect("valid array");
        ds.as_numeric().expect("numeric column").values().to_vec()
    });

    let mut values = Vec::new();
    let mut point_count = 0;
    for row in start..num_rows {
        let Some((x_values, trace_values)) = series_array.expand_trace(row)? else {
            continue;
        };
        #[expect(
            clippy::cast_precision_loss,
            reason = "Row index is unlikely to exceed 2^53"
        )]
        let y_val = y_values
            .as_ref()
            .and_then(|v| v.get(row).copied())
            .unwrap_or(row as f64);
        let ds_trace: DatasetArray = trace_values.try_into()?;
        if is_complex {
            let complex_array = ds_trace.as_complex().context("Expected complex array")?;
            let z_values = transform_complex_values(
                complex_array.real().values(),
                complex_array.imag().values(),
                view_option,
            );
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                values.push(x_values[i]);
                values.push(y_val);
                values.push(z_values[i]);
            }
            point_count += len;
        } else {
            let z_values = ds_trace
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                values.push(x_values[i]);
                values.push(y_val);
                values.push(z_values[i]);
            }
            point_count += len;
        }
    }

    let name = if is_complex {
        format!("{quantity_name} ({})", complex_view_label(view_option))
    } else {
        quantity_name.clone()
    };

    let series = vec![FlatXYZSeries::new(name.clone(), name, values, point_count)];

    Ok(ChartSnapshot::Heatmap(HeatmapChartSnapshot {
        x_name: format!("{quantity_name} - X"),
        y_name,
        series,
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{ArrayRef, Float64Array};
    use arrow_schema::{DataType, Field};
    use arrow_select::concat::concat;
    use fricon::{DatasetScalar, ScalarArray, ScalarKind, TraceKind};
    use indexmap::IndexMap;

    use super::*;
    use crate::features::charts::transform::test_utils::{numeric_batch, numeric_schema};

    fn heatmap_snapshot(snapshot: ChartSnapshot) -> HeatmapChartSnapshot {
        match snapshot {
            ChartSnapshot::Heatmap(snapshot) => snapshot,
            other @ ChartSnapshot::Xy(_) => {
                panic!("expected heatmap snapshot, got {other:?}")
            }
        }
    }

    fn xyz_points(series: &FlatXYZSeries) -> Vec<Vec<f64>> {
        series
            .values
            .chunks_exact(3)
            .map(|point| vec![point[0], point[1], point[2]])
            .collect()
    }

    fn trace_batch(
        trace_rows: Vec<Vec<f64>>,
        index_values: &[f64],
    ) -> (RecordBatch, DatasetSchema) {
        let trace_array: ArrayRef = if trace_rows.is_empty() {
            let sample: ArrayRef = DatasetArray::from(DatasetScalar::SimpleTrace(
                ScalarArray::from_iter(Vec::<f64>::new()),
            ))
            .into();
            sample.slice(0, 0)
        } else {
            let row_arrays: Vec<ArrayRef> = trace_rows
                .into_iter()
                .map(|row| {
                    DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(row)))
                        .into()
                })
                .collect();
            let row_refs: Vec<&dyn arrow_array::Array> =
                row_arrays.iter().map(|array| &**array).collect();
            concat(&row_refs).unwrap()
        };
        let index_array: ArrayRef = Arc::new(Float64Array::from(index_values.to_vec()));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("idx", DataType::Float64, false),
            Field::new("trace", trace_array.data_type().clone(), false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![index_array, trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "idx".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        (batch, DatasetSchema::new(columns))
    }

    fn trace_batch_with_two_indices(
        trace_rows: Vec<Vec<f64>>,
        outer_values: &[f64],
        row_values: &[f64],
    ) -> (RecordBatch, DatasetSchema) {
        let trace_array: ArrayRef = if trace_rows.is_empty() {
            let sample: ArrayRef = DatasetArray::from(DatasetScalar::SimpleTrace(
                ScalarArray::from_iter(Vec::<f64>::new()),
            ))
            .into();
            sample.slice(0, 0)
        } else {
            let row_arrays: Vec<ArrayRef> = trace_rows
                .into_iter()
                .map(|row| {
                    DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(row)))
                        .into()
                })
                .collect();
            let row_refs: Vec<&dyn arrow_array::Array> =
                row_arrays.iter().map(|array| &**array).collect();
            concat(&row_refs).unwrap()
        };
        let outer_array: ArrayRef = Arc::new(Float64Array::from(outer_values.to_vec()));
        let row_array: ArrayRef = Arc::new(Float64Array::from(row_values.to_vec()));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("outer", DataType::Float64, false),
            Field::new("row", DataType::Float64, false),
            Field::new("trace", trace_array.data_type().clone(), false),
        ]));
        let batch =
            RecordBatch::try_new(arrow_schema, vec![outer_array, row_array, trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "outer".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "row".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        (batch, DatasetSchema::new(columns))
    }

    /// 9 rows: 3 outer sweeps × 3 inner points.
    /// Index columns: sweep (outer), freq (MFI-2), point (MFI).
    /// Only the last outer sweep should appear.
    #[test]
    fn scalar_shows_only_last_sweep() {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]),
            (
                "freq",
                &[10.0, 10.0, 20.0, 20.0, 10.0, 10.0, 10.0, 10.0, 20.0],
            ),
            ("point", &[1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0]),
            ("val", &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]),
        ]);
        let schema = numeric_schema(&["sweep", "freq", "point", "val"]);
        let options = LiveHeatmapOptions {
            quantity: "val".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };
        // index_columns = [0 (sweep), 1 (freq), 2 (point)]
        // MFI = point (last), second-MFI = freq, outer = sweep
        let res = heatmap_snapshot(
            build_live_heatmap_series(&batch, &schema, Some(&[0, 1, 2]), &options).unwrap(),
        );
        // Only rows from the last outer-sweep (sweep=2, rows 6-8) should be included
        assert_eq!(res.series.len(), 1);
        // The heatmap should contain data from the last 3 rows only
        assert_eq!(res.series[0].point_count, 3);
    }

    #[test]
    fn scalar_keeps_all_rows_from_latest_outer_sweep() {
        let batch = numeric_batch(&[
            ("cycle", &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            ("y", &[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0]),
            ("x", &[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
            (
                "val",
                &[
                    0.0, 1.0, 10.0, 11.0, 100.0, 101.0, 110.0, 111.0, 120.0, 121.0,
                ],
            ),
        ]);
        let schema = numeric_schema(&["cycle", "y", "x", "val"]);
        let options = LiveHeatmapOptions {
            quantity: "val".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };

        let res = heatmap_snapshot(
            build_live_heatmap_series(&batch, &schema, Some(&[0, 1, 2]), &options).unwrap(),
        );

        assert_eq!(res.x_name, "x");
        assert_eq!(res.y_name, "y");
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].point_count, 6);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![
                vec![0.0, 0.0, 100.0],
                vec![1.0, 0.0, 101.0],
                vec![0.0, 1.0, 110.0],
                vec![1.0, 1.0, 111.0],
                vec![0.0, 2.0, 120.0],
                vec![1.0, 2.0, 121.0],
            ]
        );
    }

    #[test]
    fn requires_two_index_columns() {
        let batch = numeric_batch(&[("idx", &[1.0, 2.0, 3.0]), ("val", &[10.0, 20.0, 30.0])]);
        let schema = numeric_schema(&["idx", "val"]);
        let options = LiveHeatmapOptions {
            quantity: "val".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };
        // Only one index column → should error
        let res = build_live_heatmap_series(&batch, &schema, Some(&[0]), &options);
        assert!(res.is_err());
    }

    #[test]
    fn trace_empty_batch_uses_heatmap_contract_defaults() {
        let (batch, schema) = trace_batch(vec![], &[]);
        let options = LiveHeatmapOptions {
            quantity: "trace".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };

        let res = heatmap_snapshot(
            build_live_heatmap_series(&batch, &schema, Some(&[0]), &options).unwrap(),
        );

        assert_eq!(res.y_name, "idx");
        assert!(res.series.is_empty());
    }

    #[test]
    fn trace_without_index_columns_falls_back_to_row_y_axis() {
        let (batch, schema) = trace_batch(vec![vec![1.0, 2.0]], &[0.0]);
        let options = LiveHeatmapOptions {
            quantity: "trace".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };

        let res =
            heatmap_snapshot(build_live_heatmap_series(&batch, &schema, None, &options).unwrap());

        assert_eq!(res.y_name, "row");
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![vec![0.0, 0.0, 1.0], vec![1.0, 0.0, 2.0]]
        );
    }

    #[test]
    fn trace_keeps_all_rows_from_latest_outer_sweep() {
        let (batch, schema) = trace_batch_with_two_indices(
            vec![
                vec![1.0, 2.0],
                vec![3.0, 4.0],
                vec![5.0, 6.0],
                vec![7.0, 8.0],
            ],
            &[0.0, 0.0, 1.0, 1.0],
            &[0.0, 1.0, 0.0, 1.0],
        );
        let options = LiveHeatmapOptions {
            quantity: "trace".to_string(),
            complex_view_single: None,
            known_row_count: None,
        };

        let res = heatmap_snapshot(
            build_live_heatmap_series(&batch, &schema, Some(&[0, 1]), &options).unwrap(),
        );

        assert_eq!(res.x_name, "trace - X");
        assert_eq!(res.y_name, "row");
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].point_count, 4);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![
                vec![0.0, 0.0, 5.0],
                vec![1.0, 0.0, 6.0],
                vec![0.0, 1.0, 7.0],
                vec![1.0, 1.0, 8.0],
            ]
        );
    }
}

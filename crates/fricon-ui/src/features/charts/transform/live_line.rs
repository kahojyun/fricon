use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::{compute_sweep_groups, group_series_id, row_series_id};
use crate::features::charts::types::{
    ChartSnapshot, ComplexViewOption, FlatXYSeries, LineChartSnapshot, LiveLineOptions,
    complex_view_label, transform_complex_values,
};

fn row_axis_value(row: usize) -> Result<f64> {
    let row = u32::try_from(row).context("Row index exceeds supported chart range")?;
    Ok(f64::from(row))
}

/// Build live line series for **trace** columns.
///
/// Each row in the batch is one sweep. We take the last `tail_count` rows,
/// expand each row's trace, and produce one `Series` per row.
fn build_trace_live_series(
    series_name: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    tail_count: usize,
) -> Result<(String, Vec<FlatXYSeries>)> {
    let num_rows = series_array.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let x_name = format!("{series_name} - X");

    let mut series = Vec::new();
    for row in start..num_rows {
        let Some((x_values, y_values_array)) = series_array
            .expand_trace(row)
            .with_context(|| format!("Failed to expand trace row {row}"))?
        else {
            continue;
        };
        if x_values.is_empty() {
            continue;
        }

        let ds_y: DatasetArray = y_values_array.try_into()?;
        if is_complex {
            let complex_array = ds_y.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            for &view in complex_views {
                let y_values = transform_complex_values(reals, imags, view);
                let len = x_values.len().min(y_values.len());
                let mut values = Vec::with_capacity(len * 2);
                for i in 0..len {
                    values.push(x_values[i]);
                    values.push(y_values[i]);
                }
                series.push(FlatXYSeries::new(
                    format!("{}:{}", row_series_id(row), complex_view_label(view)),
                    format!("{series_name} ({})", complex_view_label(view)),
                    values,
                    len,
                ));
            }
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(y_values.len());
            let mut values = Vec::with_capacity(len * 2);
            for i in 0..len {
                values.push(x_values[i]);
                values.push(y_values[i]);
            }
            series.push(FlatXYSeries::new(
                row_series_id(row),
                series_name.to_string(),
                values,
                len,
            ));
        }
    }

    Ok((x_name, series))
}

/// Build live line series for **scalar** columns with index grouping.
///
/// Rows are grouped by outer index columns (all index columns except the
/// most-frequent / last one). The most-frequent index becomes the x-axis.
/// We take the last `tail_count` complete groups.
fn build_scalar_live_series(
    batch: &RecordBatch,
    series_name: &str,
    index_columns: &[usize],
    schema: &DatasetSchema,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    tail_count: usize,
) -> Result<(String, Vec<FlatXYSeries>)> {
    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();

    // Most-frequent index = last index column
    let mfi_idx = *index_columns.last().context("No index columns")?;
    let mfi_name = column_names[mfi_idx];

    // Extract x-axis values (most-frequent index)
    let x_array = batch
        .column_by_name(mfi_name)
        .context("MFI column not found")?;
    let ds_x: DatasetArray = x_array.clone().try_into()?;
    let x_values = ds_x
        .as_numeric()
        .context("MFI column must be numeric")?
        .values();

    // Extract series y-values
    let series_column = batch
        .column_by_name(series_name)
        .context("Series column not found")?;
    let ds_y: DatasetArray = series_column.clone().try_into()?;

    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok((mfi_name.to_string(), vec![]));
    }

    // Compute sweep groups and take the last `tail_count`
    let groups = compute_sweep_groups(batch, schema, Some(index_columns));
    let start_group = groups.len().saturating_sub(tail_count);
    let selected_groups = &groups[start_group..];

    let mut series_list = Vec::new();

    for (group_index, &group_start) in selected_groups.iter().enumerate() {
        let group_end = selected_groups
            .get(group_index + 1)
            .copied()
            .unwrap_or(num_rows);
        if is_complex {
            let complex_array = ds_y.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            for &view in complex_views {
                let y_values = transform_complex_values(
                    &reals[group_start..group_end],
                    &imags[group_start..group_end],
                    view,
                );
                let len = group_end - group_start;
                let mut values = Vec::with_capacity(len * 2);
                for (i, row) in (group_start..group_end).enumerate() {
                    values.push(x_values[row]);
                    values.push(y_values[i]);
                }
                series_list.push(FlatXYSeries::new(
                    format!(
                        "{}:{}",
                        group_series_id(group_start),
                        complex_view_label(view)
                    ),
                    format!("{series_name} ({})", complex_view_label(view)),
                    values,
                    len,
                ));
            }
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = group_end - group_start;
            let mut values = Vec::with_capacity(len * 2);
            for row in group_start..group_end {
                values.push(x_values[row]);
                values.push(y_values[row]);
            }
            series_list.push(FlatXYSeries::new(
                group_series_id(group_start),
                series_name.to_string(),
                values,
                len,
            ));
        }
    }

    Ok((mfi_name.to_string(), series_list))
}

/// Build live line series for scalar data with exactly one index column.
///
/// This remains a single continuous series over the index values rather than
/// splitting each row into its own sweep.
fn build_scalar_single_index_live_series(
    batch: &RecordBatch,
    series_name: &str,
    index_column: usize,
    schema: &DatasetSchema,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    tail_count: usize,
) -> Result<(String, Vec<FlatXYSeries>)> {
    let column_names: Vec<&str> = schema.columns().keys().map(String::as_str).collect();
    let x_name = column_names[index_column];
    let x_array = batch
        .column_by_name(x_name)
        .context("Index column not found")?;
    let ds_x: DatasetArray = x_array.clone().try_into()?;
    let x_values = ds_x
        .as_numeric()
        .context("Index column must be numeric")?
        .values();

    let series_column = batch
        .column_by_name(series_name)
        .context("Series column not found")?;
    let ds_y: DatasetArray = series_column.clone().try_into()?;
    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);

    let series = if is_complex {
        let complex_array = ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        complex_views
            .iter()
            .map(|&view| {
                let y_values = transform_complex_values(
                    &reals[start..num_rows],
                    &imags[start..num_rows],
                    view,
                );
                let len = num_rows - start;
                let mut values = Vec::with_capacity(len * 2);
                for (i, row) in (start..num_rows).enumerate() {
                    values.push(x_values[row]);
                    values.push(y_values[i]);
                }
                FlatXYSeries::new(
                    format!("{series_name}:{}", complex_view_label(view)),
                    format!("{series_name} ({})", complex_view_label(view)),
                    values,
                    len,
                )
            })
            .collect()
    } else {
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = num_rows - start;
        let mut values = Vec::with_capacity(len * 2);
        for row in start..num_rows {
            values.push(x_values[row]);
            values.push(y_values[row]);
        }
        vec![FlatXYSeries::new(
            series_name.to_string(),
            series_name.to_string(),
            values,
            len,
        )]
    };

    Ok((x_name.to_string(), series))
}

/// Build live line series for scalar data **without** index columns.
///
/// Simply shows the last `tail_count` data points as a single series.
fn build_scalar_no_index_live_series(
    batch: &RecordBatch,
    series_name: &str,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    tail_count: usize,
) -> Result<(String, Vec<FlatXYSeries>)> {
    let series_column = batch
        .column_by_name(series_name)
        .context("Series column not found")?;
    let ds_y: DatasetArray = series_column.clone().try_into()?;
    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let x_name = "row".to_string();

    let series = if is_complex {
        let complex_array = ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        complex_views
            .iter()
            .map(|&view| {
                let y_values = transform_complex_values(
                    &reals[start..num_rows],
                    &imags[start..num_rows],
                    view,
                );
                let len = num_rows - start;
                let mut values = Vec::with_capacity(len * 2);
                for (i, row) in (start..num_rows).enumerate() {
                    values.push(row_axis_value(row)?);
                    values.push(y_values[i]);
                }
                Ok(FlatXYSeries::new(
                    format!("{series_name}:{}", complex_view_label(view)),
                    format!("{series_name} ({})", complex_view_label(view)),
                    values,
                    len,
                ))
            })
            .collect::<Result<Vec<_>>>()
    } else {
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = num_rows - start;
        let mut values = Vec::with_capacity(len * 2);
        for row in start..num_rows {
            values.push(row_axis_value(row)?);
            values.push(y_values[row]);
        }
        Ok(vec![FlatXYSeries::new(
            series_name.to_string(),
            series_name.to_string(),
            values,
            len,
        )])
    }?;

    Ok((x_name, series))
}

pub(crate) fn build_live_line_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveLineOptions,
) -> Result<ChartSnapshot> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let tail_count = options.tail_count.max(1);
    let complex_views = if is_complex {
        options
            .complex_views
            .clone()
            .unwrap_or_else(|| vec![ComplexViewOption::Real, ComplexViewOption::Imag])
    } else {
        vec![]
    };

    debug!(
        chart_type = "live_line",
        series = %series_name,
        ?data_type,
        rows = batch.num_rows(),
        tail_count,
        "Building live line chart series"
    );

    let (x_name, series) = if is_trace {
        let series_column = batch
            .column_by_name(series_name)
            .cloned()
            .context("Column not found")?;
        let series_array: DatasetArray = series_column.try_into()?;
        build_trace_live_series(
            series_name,
            &series_array,
            is_complex,
            &complex_views,
            tail_count,
        )?
    } else if let Some(idx_cols) = index_columns
        && idx_cols.len() >= 2
    {
        build_scalar_live_series(
            batch,
            series_name,
            idx_cols,
            schema,
            is_complex,
            &complex_views,
            tail_count,
        )?
    } else if let Some(&index_column) = index_columns.and_then(|idx_cols| idx_cols.first()) {
        build_scalar_single_index_live_series(
            batch,
            series_name,
            index_column,
            schema,
            is_complex,
            &complex_views,
            tail_count,
        )?
    } else {
        build_scalar_no_index_live_series(
            batch,
            series_name,
            is_complex,
            &complex_views,
            tail_count,
        )?
    };

    Ok(ChartSnapshot::Line(LineChartSnapshot { x_name, series }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::charts::transform::test_utils::{numeric_batch, numeric_schema};

    fn line_snapshot(snapshot: ChartSnapshot) -> LineChartSnapshot {
        match snapshot {
            ChartSnapshot::Line(snapshot) => snapshot,
            other => panic!("expected line snapshot, got {other:?}"),
        }
    }

    fn xy_points(series: &FlatXYSeries) -> Vec<Vec<f64>> {
        series
            .values
            .chunks_exact(2)
            .map(|point| vec![point[0], point[1]])
            .collect()
    }

    /// 9 rows, 2 index columns (sweep, freq), series "val".
    /// sweep transitions at rows 0, 3, 6 → 3 groups of 3.
    fn sample_indexed_batch() -> (RecordBatch, DatasetSchema) {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0]),
            (
                "freq",
                &[10.0, 20.0, 30.0, 10.0, 20.0, 30.0, 10.0, 20.0, 30.0],
            ),
            ("val", &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]),
        ]);
        let schema = numeric_schema(&["sweep", "freq", "val"]);
        (batch, schema)
    }

    #[test]
    fn scalar_with_index_groups_by_sweep() {
        let (batch, schema) = sample_indexed_batch();
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_views: None,
            tail_count: 10,
            known_row_count: None,
        };
        let res = line_snapshot(
            build_live_line_series(&batch, &schema, Some(&[0, 1]), &options).unwrap(),
        );

        assert_eq!(res.x_name, "freq"); // MFI = last index column
        assert_eq!(res.series.len(), 3);
        assert_eq!(res.series[0].id, "group:0");
        assert_eq!(res.series[1].id, "group:3");
        assert_eq!(res.series[2].id, "group:6");
        // First sweep: freq=[10,20,30], val=[0.1,0.2,0.3]
        assert_eq!(
            xy_points(&res.series[0]),
            vec![vec![10.0, 0.1], vec![20.0, 0.2], vec![30.0, 0.3]]
        );
    }

    #[test]
    fn scalar_with_index_respects_tail_count() {
        let (batch, schema) = sample_indexed_batch();
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_views: None,
            tail_count: 2,
            known_row_count: None,
        };
        let res = line_snapshot(
            build_live_line_series(&batch, &schema, Some(&[0, 1]), &options).unwrap(),
        );

        assert_eq!(res.series.len(), 2);
        assert_eq!(res.series[0].id, "group:3");
        assert_eq!(res.series[1].id, "group:6");
        // Only last 2 sweeps: sweep=2 and sweep=3
        assert_eq!(
            xy_points(&res.series[1]),
            vec![vec![10.0, 0.7], vec![20.0, 0.8], vec![30.0, 0.9]]
        );
    }

    #[test]
    fn scalar_no_index_shows_last_n_points() {
        let batch = numeric_batch(&[("val", &[1.0, 2.0, 3.0, 4.0, 5.0])]);
        let schema = numeric_schema(&["val"]);
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_views: None,
            tail_count: 3,
            known_row_count: None,
        };
        let res = line_snapshot(build_live_line_series(&batch, &schema, None, &options).unwrap());

        assert_eq!(res.x_name, "row");
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].label, "val");
        assert_eq!(
            xy_points(&res.series[0]),
            vec![vec![2.0, 3.0], vec![3.0, 4.0], vec![4.0, 5.0]]
        );
    }

    #[test]
    fn empty_batch_returns_no_series() {
        let batch = numeric_batch(&[("val", &[])]);
        let schema = numeric_schema(&["val"]);
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_views: None,
            tail_count: 5,
            known_row_count: None,
        };
        let res = line_snapshot(build_live_line_series(&batch, &schema, None, &options).unwrap());
        assert!(res.series.is_empty() || res.series[0].values.is_empty());
    }

    #[test]
    fn scalar_single_index_stays_continuous() {
        let batch = numeric_batch(&[
            ("t", &[0.0, 1.0, 2.0, 3.0]),
            ("val", &[10.0, 11.0, 12.0, 13.0]),
        ]);
        let schema = numeric_schema(&["t", "val"]);
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_views: None,
            tail_count: 3,
            known_row_count: None,
        };
        let res =
            line_snapshot(build_live_line_series(&batch, &schema, Some(&[0]), &options).unwrap());

        assert_eq!(res.x_name, "t");
        assert_eq!(res.series.len(), 1);
        assert_eq!(
            xy_points(&res.series[0]),
            vec![vec![1.0, 11.0], vec![2.0, 12.0], vec![3.0, 13.0]]
        );
    }

    #[test]
    fn complex_line_respects_selected_views() {
        use std::sync::Arc;

        use arrow_array::{Float64Array, RecordBatch, StructArray};
        use arrow_schema::{DataType, Field};
        use fricon::{DatasetDataType, ScalarKind};
        use indexmap::IndexMap;

        let fields = vec![
            Arc::new(Field::new("real", DataType::Float64, false)),
            Arc::new(Field::new("imag", DataType::Float64, false)),
        ];
        let complex_struct = StructArray::try_new(
            fields.into(),
            vec![
                Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0])),
                Arc::new(Float64Array::from(vec![4.0, 5.0, 6.0])),
            ],
            None,
        )
        .unwrap();
        let batch = RecordBatch::try_from_iter(vec![
            ("t", Arc::new(Float64Array::from(vec![0.0, 1.0, 2.0])) as _),
            ("sig", Arc::new(complex_struct) as _),
        ])
        .unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "t".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "sig".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        let schema = DatasetSchema::new(columns);

        let options = LiveLineOptions {
            series: "sig".to_string(),
            complex_views: Some(vec![ComplexViewOption::Real, ComplexViewOption::Mag]),
            tail_count: 3,
            known_row_count: None,
        };
        let res =
            line_snapshot(build_live_line_series(&batch, &schema, Some(&[0]), &options).unwrap());

        assert_eq!(res.series.len(), 2);
        assert_eq!(res.series[0].label, "sig (real)");
        assert_eq!(res.series[1].label, "sig (mag)");
    }
}

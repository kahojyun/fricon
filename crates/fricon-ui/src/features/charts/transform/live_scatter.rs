use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::{compute_sweep_groups, sweep_name};
use crate::features::charts::types::{
    ChartDataResponse, ChartType, LiveScatterOptions, ScatterModeOptions, Series,
};

/// Build live scatter series split into per-sweep groups so the frontend can
/// apply color/opacity differentiation between old and current sweeps.
pub(crate) fn build_live_scatter_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveScatterOptions,
) -> Result<ChartDataResponse> {
    let tail_count = options.tail_count.max(1);
    debug!(
        chart_type = "live_scatter",
        rows = batch.num_rows(),
        tail_count,
        "Building live scatter chart series"
    );

    let (x_name, y_name, series) = match &options.scatter {
        ScatterModeOptions::Complex { series } => {
            build_complex_live(batch, schema, index_columns, series, tail_count)?
        }
        ScatterModeOptions::TraceXy {
            trace_x_column,
            trace_y_column,
        } => build_trace_xy_live(batch, trace_x_column, trace_y_column, tail_count)?,
        ScatterModeOptions::Xy {
            x_column, y_column, ..
        } => build_xy_live(batch, index_columns, schema, x_column, y_column, tail_count)?,
    };

    Ok(ChartDataResponse {
        r#type: ChartType::Scatter,
        x_name,
        y_name: Some(y_name),
        x_categories: None,
        y_categories: None,
        series,
    })
}

/// Complex scatter: real vs imag, split per sweep.
fn build_complex_live(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    series_name: &str,
    tail_count: usize,
) -> Result<(String, String, Vec<Series>)> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));

    if is_trace {
        // Trace: each row is one sweep
        let series_array: DatasetArray = batch
            .column_by_name(series_name)
            .cloned()
            .context("Column not found")?
            .try_into()?;
        let num_rows = series_array.num_rows();
        let start = num_rows.saturating_sub(tail_count);
        let total = num_rows - start;
        let mut result = Vec::new();
        for (age, row) in (start..num_rows).enumerate() {
            let Some((_x_values, trace_values)) = series_array.expand_trace(row)? else {
                continue;
            };
            let ds_trace: DatasetArray = trace_values.try_into()?;
            let complex_array = ds_trace.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            let len = reals.len().min(imags.len());
            let data = (0..len).map(|i| vec![reals[i], imags[i]]).collect();
            result.push(Series {
                name: sweep_name(age, total),
                data,
            });
        }
        Ok((
            format!("{series_name} (real)"),
            format!("{series_name} (imag)"),
            result,
        ))
    } else {
        // Scalar complex: group by outer index
        let series_array: DatasetArray = batch
            .column_by_name(series_name)
            .cloned()
            .context("Column not found")?
            .try_into()?;
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();

        let groups = compute_sweep_groups(batch, schema, index_columns);
        let start_group = groups.len().saturating_sub(tail_count);
        let selected = &groups[start_group..];
        let total = selected.len();
        let num_rows = batch.num_rows();

        let mut result = Vec::new();
        for (age, &group_start) in selected.iter().enumerate() {
            let group_end = selected.get(age + 1).copied().unwrap_or(num_rows);
            let data = (group_start..group_end)
                .map(|i| vec![reals[i], imags[i]])
                .collect();
            result.push(Series {
                name: sweep_name(age, total),
                data,
            });
        }
        Ok((
            format!("{series_name} (real)"),
            format!("{series_name} (imag)"),
            result,
        ))
    }
}

/// Trace X/Y scatter: each row pair is one sweep.
fn build_trace_xy_live(
    batch: &RecordBatch,
    trace_x: &str,
    trace_y: &str,
    tail_count: usize,
) -> Result<(String, String, Vec<Series>)> {
    let x_array: DatasetArray = batch
        .column_by_name(trace_x)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(trace_y)
        .cloned()
        .context("Y not found")?
        .try_into()?;

    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let total = num_rows - start;
    let mut result = Vec::new();

    for (age, row) in (start..num_rows).enumerate() {
        let Some((_x_axis, x_values_array)) = x_array.expand_trace(row)? else {
            continue;
        };
        let Some((_y_axis, y_values_array)) = y_array.expand_trace(row)? else {
            continue;
        };
        let ds_x: DatasetArray = x_values_array.try_into()?;
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let x_values = ds_x.as_numeric().context("X must be numeric")?.values();
        let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();
        let len = x_values.len().min(y_values.len());
        let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
        result.push(Series {
            name: sweep_name(age, total),
            data,
        });
    }

    Ok((trace_x.to_string(), trace_y.to_string(), result))
}

/// X/Y column scatter: group rows by outer index, split per sweep.
fn build_xy_live(
    batch: &RecordBatch,
    index_columns: Option<&[usize]>,
    schema: &DatasetSchema,
    x_column: &str,
    y_column: &str,
    tail_count: usize,
) -> Result<(String, String, Vec<Series>)> {
    let x_array: DatasetArray = batch
        .column_by_name(x_column)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y not found")?
        .try_into()?;
    let x_values = x_array.as_numeric().context("X must be numeric")?.values();
    let y_values = y_array.as_numeric().context("Y must be numeric")?.values();
    let num_rows = batch.num_rows();

    let groups = compute_sweep_groups(batch, schema, index_columns);
    let start_group = groups.len().saturating_sub(tail_count);
    let selected = &groups[start_group..];
    let total = selected.len();

    let mut result = Vec::new();
    for (age, &group_start) in selected.iter().enumerate() {
        let group_end = selected.get(age + 1).copied().unwrap_or(num_rows);
        let data = (group_start..group_end)
            .map(|i| vec![x_values[i], y_values[i]])
            .collect();
        result.push(Series {
            name: sweep_name(age, total),
            data,
        });
    }

    Ok((x_column.to_string(), y_column.to_string(), result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::charts::transform::test_utils::{numeric_batch, numeric_schema};

    /// 9 rows: 3 sweeps of 3 points each.
    fn sample_xy_batch() -> (RecordBatch, DatasetSchema) {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0]),
            ("x", &[0.1, 0.2, 0.3, 0.1, 0.2, 0.3, 0.1, 0.2, 0.3]),
            ("y", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]),
        ]);
        let schema = numeric_schema(&["sweep", "x", "y"]);
        (batch, schema)
    }

    #[test]
    fn xy_splits_per_sweep() {
        let (batch, schema) = sample_xy_batch();
        let options = LiveScatterOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: None,
            },
            tail_count: 10,
        };
        let res = build_live_scatter_series(&batch, &schema, Some(&[0, 1]), &options).unwrap();

        assert_eq!(res.r#type, ChartType::Scatter);
        assert_eq!(res.series.len(), 3);
        assert_eq!(res.series[0].name, "-2");
        assert_eq!(res.series[2].name, "current");
        assert_eq!(
            res.series[2].data,
            vec![vec![0.1, 7.0], vec![0.2, 8.0], vec![0.3, 9.0]]
        );
    }

    #[test]
    fn xy_respects_tail_count() {
        let (batch, schema) = sample_xy_batch();
        let options = LiveScatterOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: None,
            },
            tail_count: 2,
        };
        let res = build_live_scatter_series(&batch, &schema, Some(&[0, 1]), &options).unwrap();

        assert_eq!(res.series.len(), 2);
        assert_eq!(res.series[0].name, "-1");
        assert_eq!(res.series[1].name, "current");
    }

    #[test]
    fn xy_no_index_each_row_is_sweep() {
        let batch = numeric_batch(&[("x", &[1.0, 2.0, 3.0]), ("y", &[10.0, 20.0, 30.0])]);
        let schema = numeric_schema(&["x", "y"]);
        let options = LiveScatterOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: None,
            },
            tail_count: 2,
        };
        let res = build_live_scatter_series(&batch, &schema, None, &options).unwrap();

        // Without 2+ index columns, each row is its own "sweep"
        assert_eq!(res.series.len(), 2);
        assert_eq!(res.series[1].name, "current");
        assert_eq!(res.series[1].data, vec![vec![3.0, 30.0]]);
    }
}

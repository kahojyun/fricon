use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::{compute_sweep_groups, sweep_name};
use crate::features::charts::types::{
    ChartDataResponse, ChartType, ComplexViewOption, LiveLineOptions, Series,
    transform_complex_values,
};

/// Build live line series for **trace** columns.
///
/// Each row in the batch is one sweep. We take the last `tail_count` rows,
/// expand each row's trace, and produce one `Series` per row.
fn build_trace_live_series(
    series_name: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    complex_view: Option<ComplexViewOption>,
    tail_count: usize,
) -> Result<(String, Vec<Series>)> {
    let num_rows = series_array.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let x_name = format!("{series_name} - X");

    let mut series = Vec::new();
    let total = num_rows - start;
    for (age, row) in (start..num_rows).enumerate() {
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
        let name = sweep_name(age, total);

        if is_complex {
            let complex_array = ds_y.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            let view = complex_view.unwrap_or(ComplexViewOption::Mag);
            let y_values = transform_complex_values(reals, imags, view);
            let len = x_values.len().min(y_values.len());
            let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
            series.push(Series { name, data });
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(y_values.len());
            let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
            series.push(Series { name, data });
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
    complex_view: Option<ComplexViewOption>,
    tail_count: usize,
) -> Result<(String, Vec<Series>)> {
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
    let total = selected_groups.len();

    let mut series_list = Vec::new();

    for (age, &group_start) in selected_groups.iter().enumerate() {
        let group_end = selected_groups.get(age + 1).copied().unwrap_or(num_rows);
        let name = sweep_name(age, total);

        if is_complex {
            let complex_array = ds_y.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            let view = complex_view.unwrap_or(ComplexViewOption::Mag);
            let y_values = transform_complex_values(
                &reals[group_start..group_end],
                &imags[group_start..group_end],
                view,
            );
            let data = (group_start..group_end)
                .enumerate()
                .map(|(i, row)| vec![x_values[row], y_values[i]])
                .collect();
            series_list.push(Series { name, data });
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let data = (group_start..group_end)
                .map(|row| vec![x_values[row], y_values[row]])
                .collect();
            series_list.push(Series { name, data });
        }
    }

    Ok((mfi_name.to_string(), series_list))
}

/// Build live line series for scalar data **without** index columns.
///
/// Simply shows the last `tail_count` data points as a single series.
fn build_scalar_no_index_live_series(
    batch: &RecordBatch,
    series_name: &str,
    is_complex: bool,
    complex_view: Option<ComplexViewOption>,
    tail_count: usize,
) -> Result<(String, Vec<Series>)> {
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
        let view = complex_view.unwrap_or(ComplexViewOption::Mag);
        let y_values =
            transform_complex_values(&reals[start..num_rows], &imags[start..num_rows], view);
        #[expect(
            clippy::cast_precision_loss,
            reason = "Row index is unlikely to exceed 2^53"
        )]
        let data = (start..num_rows)
            .enumerate()
            .map(|(i, row)| vec![row as f64, y_values[i]])
            .collect();
        vec![Series {
            name: series_name.to_string(),
            data,
        }]
    } else {
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        #[expect(
            clippy::cast_precision_loss,
            reason = "Row index is unlikely to exceed 2^53"
        )]
        let data = (start..num_rows)
            .map(|row| vec![row as f64, y_values[row]])
            .collect();
        vec![Series {
            name: series_name.to_string(),
            data,
        }]
    };

    Ok((x_name, series))
}

pub(crate) fn build_live_line_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveLineOptions,
) -> Result<ChartDataResponse> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let tail_count = options.tail_count.max(1);

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
            options.complex_view,
            tail_count,
        )?
    } else if let Some(idx_cols) = index_columns
        && !idx_cols.is_empty()
    {
        build_scalar_live_series(
            batch,
            series_name,
            idx_cols,
            schema,
            is_complex,
            options.complex_view,
            tail_count,
        )?
    } else {
        build_scalar_no_index_live_series(
            batch,
            series_name,
            is_complex,
            options.complex_view,
            tail_count,
        )?
    };

    Ok(ChartDataResponse {
        r#type: ChartType::Line,
        x_name,
        y_name: None,
        x_categories: None,
        y_categories: None,
        series,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::charts::transform::test_utils::{numeric_batch, numeric_schema};

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
            complex_view: None,
            tail_count: 10,
        };
        let res = build_live_line_series(&batch, &schema, Some(&[0, 1]), &options).unwrap();

        assert_eq!(res.r#type, ChartType::Line);
        assert_eq!(res.x_name, "freq"); // MFI = last index column
        assert_eq!(res.series.len(), 3);
        assert_eq!(res.series[0].name, "-2");
        assert_eq!(res.series[1].name, "-1");
        assert_eq!(res.series[2].name, "current");
        // First sweep: freq=[10,20,30], val=[0.1,0.2,0.3]
        assert_eq!(
            res.series[0].data,
            vec![vec![10.0, 0.1], vec![20.0, 0.2], vec![30.0, 0.3]]
        );
    }

    #[test]
    fn scalar_with_index_respects_tail_count() {
        let (batch, schema) = sample_indexed_batch();
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_view: None,
            tail_count: 2,
        };
        let res = build_live_line_series(&batch, &schema, Some(&[0, 1]), &options).unwrap();

        assert_eq!(res.series.len(), 2);
        assert_eq!(res.series[0].name, "-1");
        assert_eq!(res.series[1].name, "current");
        // Only last 2 sweeps: sweep=2 and sweep=3
        assert_eq!(
            res.series[1].data,
            vec![vec![10.0, 0.7], vec![20.0, 0.8], vec![30.0, 0.9]]
        );
    }

    #[test]
    fn scalar_no_index_shows_last_n_points() {
        let batch = numeric_batch(&[("val", &[1.0, 2.0, 3.0, 4.0, 5.0])]);
        let schema = numeric_schema(&["val"]);
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_view: None,
            tail_count: 3,
        };
        let res = build_live_line_series(&batch, &schema, None, &options).unwrap();

        assert_eq!(res.x_name, "row");
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].name, "val");
        assert_eq!(
            res.series[0].data,
            vec![vec![2.0, 3.0], vec![3.0, 4.0], vec![4.0, 5.0]]
        );
    }

    #[test]
    fn empty_batch_returns_no_series() {
        let batch = numeric_batch(&[("val", &[])]);
        let schema = numeric_schema(&["val"]);
        let options = LiveLineOptions {
            series: "val".to_string(),
            complex_view: None,
            tail_count: 5,
        };
        let res = build_live_line_series(&batch, &schema, None, &options).unwrap();
        assert!(res.series.is_empty() || res.series[0].data.is_empty());
    }
}

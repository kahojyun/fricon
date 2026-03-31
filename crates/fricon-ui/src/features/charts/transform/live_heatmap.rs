use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::heatmap::build_heatmap_series;
use crate::features::charts::types::{
    ChartCommonOptions, ChartDataResponse, ComplexViewOption, HeatmapChartDataOptions,
    LiveHeatmapOptions, Series, complex_view_label, transform_complex_values,
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
) -> Result<ChartDataResponse> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));

    debug!(
        chart_type = "live_heatmap",
        series = %series_name,
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

    let column_names: Vec<&str> = schema.columns().keys().map(|k| k.as_str()).collect();
    let mfi_idx = *idx_cols.last().context("No index columns")?;
    let second_mfi_idx = idx_cols[idx_cols.len() - 2];
    let mfi_name = column_names[mfi_idx];
    let second_mfi_name = column_names[second_mfi_idx];

    // Crop to the last outer-index group (outermost indices, excluding the two
    // most-frequent ones).
    let outer_indices = &idx_cols[..idx_cols.len().saturating_sub(2)];
    let num_rows = batch.num_rows();
    let start = if !outer_indices.is_empty() && num_rows > 0 {
        let outer_columns: Vec<Vec<f64>> = outer_indices
            .iter()
            .map(|&idx| {
                let arr = batch.column_by_name(column_names[idx]).unwrap();
                let ds: DatasetArray = arr.clone().try_into().unwrap();
                ds.as_numeric().unwrap().values().to_vec()
            })
            .collect();
        let mut last_group_start = 0;
        for row in 1..num_rows {
            if outer_columns.iter().any(|col| col[row] != col[row - 1]) {
                last_group_start = row;
            }
        }
        last_group_start
    } else {
        0
    };

    let cropped = batch.slice(start, num_rows - start);

    // Delegate to the normal heatmap builder with MFI as x, second-MFI as y
    let heatmap_options = HeatmapChartDataOptions {
        series: series_name.clone(),
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
) -> Result<ChartDataResponse> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_complex = data_type.is_complex();
    let view_option = options
        .complex_view_single
        .unwrap_or(ComplexViewOption::Mag);

    // For traces, fall back to showing just the last row (or last sweep group)
    let column_names: Vec<&str> = schema.columns().keys().map(|k| k.as_str()).collect();
    let num_rows = batch.num_rows();
    if num_rows == 0 {
        return Ok(ChartDataResponse {
            r#type: crate::features::charts::types::ChartType::Heatmap,
            x_name: format!("{series_name} - X"),
            y_name: None,
            x_categories: None,
            y_categories: None,
            series: vec![],
        });
    }

    // Determine Y column: use the last index column if available
    let y_column_name = index_columns
        .and_then(|cols| cols.last())
        .map(|&idx| column_names[idx]);

    // Crop to last group if we have outer indices (more than 1 index col)
    let start = if let Some(idx_cols) = index_columns
        && idx_cols.len() >= 2
    {
        let outer_indices = &idx_cols[..idx_cols.len() - 1];
        let outer_columns: Vec<Vec<f64>> = outer_indices
            .iter()
            .map(|&idx| {
                let arr = batch.column_by_name(column_names[idx]).unwrap();
                let ds: DatasetArray = arr.clone().try_into().unwrap();
                ds.as_numeric().unwrap().values().to_vec()
            })
            .collect();
        let mut last_group_start = 0;
        for row in 1..num_rows {
            if outer_columns.iter().any(|col| col[row] != col[row - 1]) {
                last_group_start = row;
            }
        }
        last_group_start
    } else {
        0
    };

    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;

    let y_values: Option<Vec<f64>> = y_column_name.map(|name| {
        let arr = batch.column_by_name(name).unwrap();
        let ds: DatasetArray = arr.clone().try_into().unwrap();
        ds.as_numeric().unwrap().values().to_vec()
    });

    let mut data = Vec::new();
    for row in start..num_rows {
        let Some((x_values, trace_values)) = series_array.expand_trace(row)? else {
            continue;
        };
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
                data.push(vec![x_values[i], y_val, z_values[i]]);
            }
        } else {
            let z_values = ds_trace
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                data.push(vec![x_values[i], y_val, z_values[i]]);
            }
        }
    }

    let name = if is_complex {
        format!("{series_name} ({})", complex_view_label(view_option))
    } else {
        series_name.to_string()
    };

    let mut series = vec![Series { name, data }];
    let (x_categories, y_categories) = super::heatmap::normalize_heatmap_series(&mut series);

    Ok(ChartDataResponse {
        r#type: crate::features::charts::types::ChartType::Heatmap,
        x_name: format!("{series_name} - X"),
        y_name: y_column_name.map(|s| s.to_string()),
        x_categories: Some(x_categories),
        y_categories: Some(y_categories),
        series,
    })
}

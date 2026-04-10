use anyhow::{Context, Result, bail};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::debug;

use super::{
    XYTraceRoles, compute_group_starts, group_ranges, make_group_id_suffix, make_group_label,
    resolve_xy_trace_roles, row_order_for_group, row_series_id,
};
use crate::features::charts::types::{
    ChartSnapshot, ComplexViewOption, FlatXYSeries, LiveXYOptions, XYChartSnapshot, XYPlotMode,
    XYPlotModeOptions, complex_view_label, transform_complex_values,
};

pub(crate) fn build_live_xy_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    row_start: usize,
    options: &LiveXYOptions,
) -> Result<ChartSnapshot> {
    debug!(
        chart_type = "live_xy",
        plot_mode = ?options.plot_mode.plot_mode(),
        rows = batch.num_rows(),
        tail_count = options.tail_count,
        "Building live XY chart series"
    );

    let snapshot = match &options.plot_mode {
        XYPlotModeOptions::QuantityVsSweep {
            quantity,
            complex_views,
        } => build_live_quantity_vs_sweep_snapshot(
            batch,
            schema,
            index_columns,
            row_start,
            options,
            quantity,
            complex_views.as_deref().unwrap_or(&[]),
        )?,
        XYPlotModeOptions::Xy { x_column, y_column } => {
            build_live_xy_snapshot(batch, schema, index_columns, options, x_column, y_column)?
        }
        XYPlotModeOptions::ComplexPlane { quantity } => {
            build_live_complex_plane_snapshot(batch, schema, index_columns, options, quantity)?
        }
    };

    Ok(ChartSnapshot::Xy(snapshot))
}

fn build_live_quantity_vs_sweep_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    row_start: usize,
    options: &LiveXYOptions,
    series_name: &str,
    complex_views: &[ComplexViewOption],
) -> Result<XYChartSnapshot> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let tail_count = options.tail_count.max(1);

    let series = if is_trace {
        build_live_trace_quantity_vs_sweep(
            batch,
            series_name,
            is_complex,
            complex_views,
            tail_count,
        )?
    } else {
        let roles = resolve_xy_trace_roles(
            schema,
            index_columns,
            &options.trace_roles,
            options.draw_style,
        )?;
        let series_column = batch
            .column_by_name(series_name)
            .cloned()
            .context("Series column not found")?;
        let ds_y: DatasetArray = series_column.try_into()?;
        let x_values = roles
            .sweep
            .map(|index| numeric_values(batch, schema, index))
            .transpose()?;
        let ctx = LiveScalarQuantityVsSweepContext {
            batch,
            schema,
            series_name,
            ds_y: &ds_y,
            roles: &roles,
            x_values: x_values.as_deref(),
            tail_count,
            row_start,
        };
        if roles.trace_group.is_empty() {
            build_live_scalar_quantity_vs_sweep_rows(&ctx, is_complex, complex_views)?
        } else {
            build_live_scalar_quantity_vs_sweep_groups(&ctx, is_complex, complex_views)?
        }
    };

    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        resolve_live_quantity_vs_sweep_x_name(schema, index_columns, options)?
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::QuantityVsSweep,
        draw_style: options.draw_style,
        x_name,
        y_name: None,
        series,
    })
}

fn build_live_xy_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveXYOptions,
    x_column: &str,
    y_column: &str,
) -> Result<XYChartSnapshot> {
    let x_type = *schema
        .columns()
        .get(x_column)
        .context("X column not found")?;
    let y_type = *schema
        .columns()
        .get(y_column)
        .context("Y column not found")?;
    let x_is_trace = matches!(x_type, DatasetDataType::Trace(_, _));
    let y_is_trace = matches!(y_type, DatasetDataType::Trace(_, _));
    let tail_count = options.tail_count.max(1);

    let series = match (x_is_trace, y_is_trace) {
        (true, true) => build_live_trace_xy(batch, x_column, y_column, tail_count)?,
        (false, false) => {
            let roles = resolve_xy_trace_roles(
                schema,
                index_columns,
                &options.trace_roles,
                options.draw_style,
            )?;
            build_live_scalar_xy(batch, schema, x_column, y_column, tail_count, &roles)?
        }
        _ => bail!("X/Y plot mode requires both columns to be trace or both to be scalar"),
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::Xy,
        draw_style: options.draw_style,
        x_name: x_column.to_string(),
        y_name: Some(y_column.to_string()),
        series,
    })
}

fn build_live_complex_plane_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveXYOptions,
    series_name: &str,
) -> Result<XYChartSnapshot> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let tail_count = options.tail_count.max(1);

    let series = if is_trace {
        build_live_trace_complex_plane(batch, schema, series_name, tail_count)?
    } else {
        let roles = resolve_xy_trace_roles(
            schema,
            index_columns,
            &options.trace_roles,
            options.draw_style,
        )?;
        build_live_scalar_complex_plane(batch, schema, series_name, tail_count, &roles)?
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::ComplexPlane,
        draw_style: options.draw_style,
        x_name: format!("{series_name} (real)"),
        y_name: Some(format!("{series_name} (imag)")),
        series,
    })
}

fn build_live_trace_quantity_vs_sweep(
    batch: &RecordBatch,
    series_name: &str,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    tail_count: usize,
) -> Result<Vec<FlatXYSeries>> {
    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let view_options = if is_complex {
        if complex_views.is_empty() {
            vec![ComplexViewOption::Real, ComplexViewOption::Imag]
        } else {
            complex_views.to_vec()
        }
    } else {
        vec![]
    };
    let mut result = Vec::new();

    for row in start..num_rows {
        let Some((x_values, y_values_array)) = series_array.expand_trace(row)? else {
            continue;
        };
        let ds_y: DatasetArray = y_values_array.try_into()?;
        if is_complex {
            let complex_array = ds_y.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            for &view in &view_options {
                let y_values = transform_complex_values(reals, imags, view);
                result.push(make_live_trace_series(
                    row,
                    &format!("{series_name}:{}", complex_view_label(view)),
                    &format!("{series_name} ({})", complex_view_label(view)),
                    &x_values,
                    &y_values,
                ));
            }
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values()
                .to_vec();
            result.push(make_live_trace_series(
                row,
                series_name,
                series_name,
                &x_values,
                &y_values,
            ));
        }
    }

    Ok(result)
}

fn build_live_trace_xy(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
    tail_count: usize,
) -> Result<Vec<FlatXYSeries>> {
    let x_array: DatasetArray = batch
        .column_by_name(x_column)
        .cloned()
        .context("X column not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y column not found")?
        .try_into()?;
    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let mut result = Vec::new();

    for row in start..num_rows {
        let Some((_axis, x_values_array)) = x_array.expand_trace(row)? else {
            continue;
        };
        let Some((_axis, y_values_array)) = y_array.expand_trace(row)? else {
            continue;
        };
        let ds_x: DatasetArray = x_values_array.try_into()?;
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let x_values = ds_x
            .as_numeric()
            .context("Trace X must be numeric")?
            .values();
        let y_values = ds_y
            .as_numeric()
            .context("Trace Y must be numeric")?
            .values();
        let len = x_values.len().min(y_values.len());
        let mut values = Vec::with_capacity(len * 2);
        for i in 0..len {
            values.push(x_values[i]);
            values.push(y_values[i]);
        }
        result.push(FlatXYSeries::new(
            row_series_id(row),
            format!("{x_column} vs {y_column}"),
            values,
            len,
        ));
    }

    Ok(result)
}

fn build_live_trace_complex_plane(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
    tail_count: usize,
) -> Result<Vec<FlatXYSeries>> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        bail!("Complex plane live trace plot mode requires a trace quantity");
    }

    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let num_rows = batch.num_rows();
    let start = num_rows.saturating_sub(tail_count);
    let mut result = Vec::new();

    for row in start..num_rows {
        let Some((_axis, trace_values)) = series_array.expand_trace(row)? else {
            continue;
        };
        let ds_trace: DatasetArray = trace_values.try_into()?;
        let complex_array = ds_trace.as_complex().context("Expected complex trace")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        let len = reals.len().min(imags.len());
        let mut values = Vec::with_capacity(len * 2);
        for i in 0..len {
            values.push(reals[i]);
            values.push(imags[i]);
        }
        result.push(FlatXYSeries::new(
            row_series_id(row),
            series_name.to_string(),
            values,
            len,
        ));
    }

    Ok(result)
}

fn build_live_scalar_xy(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    x_column: &str,
    y_column: &str,
    tail_count: usize,
    roles: &XYTraceRoles,
) -> Result<Vec<FlatXYSeries>> {
    let x_array: DatasetArray = batch
        .column_by_name(x_column)
        .cloned()
        .context("X column not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y column not found")?
        .try_into()?;
    let x_values = x_array.as_numeric().context("X must be numeric")?.values();
    let y_values = y_array.as_numeric().context("Y must be numeric")?.values();

    Ok(build_live_grouped_xy(
        batch,
        schema,
        tail_count,
        roles,
        &format!("{x_column} vs {y_column}"),
        |row| (x_values[row], y_values[row]),
    ))
}

fn build_live_scalar_complex_plane(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
    tail_count: usize,
    roles: &XYTraceRoles,
) -> Result<Vec<FlatXYSeries>> {
    let series_column: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let complex_array = series_column
        .as_complex()
        .context("Expected complex array")?;
    let reals = complex_array.real().values();
    let imags = complex_array.imag().values();

    Ok(build_live_grouped_xy(
        batch,
        schema,
        tail_count,
        roles,
        series_name,
        |row| (reals[row], imags[row]),
    ))
}

fn build_live_grouped_xy(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    tail_count: usize,
    roles: &XYTraceRoles,
    base_label: &str,
    point_at: impl Fn(usize) -> (f64, f64),
) -> Vec<FlatXYSeries> {
    if roles.trace_group.is_empty() {
        let num_rows = batch.num_rows();
        let start = num_rows.saturating_sub(tail_count);
        let rows = row_order_for_group(batch, schema, start, num_rows, roles.sweep);
        let mut values = Vec::with_capacity(rows.len() * 2);
        for row in rows.iter().copied() {
            let (x, y) = point_at(row);
            values.push(x);
            values.push(y);
        }
        return vec![FlatXYSeries::new(
            base_label.to_string(),
            base_label.to_string(),
            values,
            rows.len(),
        )];
    }

    let groups = compute_group_starts(batch, schema, &roles.trace_group);
    let ranges = group_ranges(&groups, batch.num_rows());
    let selected = &ranges[ranges.len().saturating_sub(tail_count)..];
    let mut result = Vec::new();

    for &(group_start, group_end) in selected {
        let rows = row_order_for_group(batch, schema, group_start, group_end, roles.sweep);
        let group_label = make_group_label(batch, schema, &roles.trace_group, group_start);
        let mut values = Vec::with_capacity(rows.len() * 2);
        for row in rows.iter().copied() {
            let (x, y) = point_at(row);
            values.push(x);
            values.push(y);
        }
        result.push(FlatXYSeries::new(
            super::group_series_id(group_start),
            semantic_live_label(base_label, group_label.as_deref()),
            values,
            rows.len(),
        ));
    }

    result
}

fn resolve_live_quantity_vs_sweep_x_name(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveXYOptions,
) -> Result<String> {
    let roles = resolve_xy_trace_roles(
        schema,
        index_columns,
        &options.trace_roles,
        options.draw_style,
    )?;
    Ok(match roles.sweep {
        Some(index) => schema
            .columns()
            .get_index(index)
            .map(|(name, _)| name.clone())
            .context("Order index column not found")?,
        None => "row".to_string(),
    })
}

fn numeric_values(batch: &RecordBatch, schema: &DatasetSchema, index: usize) -> Result<Vec<f64>> {
    let name = schema
        .columns()
        .get_index(index)
        .map(|(name, _)| name.clone())
        .context("Column index not found")?;
    let column = batch
        .column_by_name(&name)
        .cloned()
        .context("Column not found")?;
    let ds: DatasetArray = column.try_into()?;
    Ok(ds
        .as_numeric()
        .context("Expected numeric column")?
        .values()
        .to_vec())
}

fn row_axis_value(row: usize) -> Result<f64> {
    let row = u32::try_from(row).context("Row index exceeds supported chart range")?;
    Ok(f64::from(row))
}

fn resolve_quantity_vs_sweep_x(
    x_values: Option<&[f64]>,
    row_start: usize,
    row: usize,
) -> Result<f64> {
    Ok(x_values
        .and_then(|values| values.get(row).copied())
        .unwrap_or(row_axis_value(row_start + row)?))
}

fn make_live_trace_series(
    row: usize,
    id_base: &str,
    label_base: &str,
    x_values: &[f64],
    y_values: &[f64],
) -> FlatXYSeries {
    let len = x_values.len().min(y_values.len());
    let mut values = Vec::with_capacity(len * 2);
    for i in 0..len {
        values.push(x_values[i]);
        values.push(y_values[i]);
    }
    FlatXYSeries::new(
        format!("{}:{id_base}", row_series_id(row)),
        label_base.to_string(),
        values,
        len,
    )
}

fn semantic_live_label(base: &str, group_label: Option<&str>) -> String {
    match group_label {
        Some(group_label) => format!("{base} [{group_label}]"),
        None => base.to_string(),
    }
}

fn resolved_complex_views(complex_views: &[ComplexViewOption]) -> Vec<ComplexViewOption> {
    if complex_views.is_empty() {
        vec![ComplexViewOption::Real, ComplexViewOption::Imag]
    } else {
        complex_views.to_vec()
    }
}

struct LiveScalarQuantityVsSweepContext<'a> {
    batch: &'a RecordBatch,
    schema: &'a DatasetSchema,
    series_name: &'a str,
    ds_y: &'a DatasetArray,
    roles: &'a XYTraceRoles,
    x_values: Option<&'a [f64]>,
    tail_count: usize,
    row_start: usize,
}

fn build_live_scalar_quantity_vs_sweep_rows(
    ctx: &LiveScalarQuantityVsSweepContext<'_>,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
) -> Result<Vec<FlatXYSeries>> {
    let num_rows = ctx.batch.num_rows();
    let start = num_rows.saturating_sub(ctx.tail_count);
    let rows = row_order_for_group(ctx.batch, ctx.schema, start, num_rows, ctx.roles.sweep);

    if is_complex {
        let complex_array = ctx.ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        return resolved_complex_views(complex_views)
            .into_iter()
            .map(|view| {
                let transformed = transform_complex_values(reals, imags, view);
                let mut values = Vec::with_capacity(rows.len() * 2);
                for row in rows.iter().copied() {
                    values.push(resolve_quantity_vs_sweep_x(
                        ctx.x_values,
                        ctx.row_start,
                        row,
                    )?);
                    values.push(transformed[row]);
                }
                Ok(FlatXYSeries::new(
                    format!("{}:{}", ctx.series_name, complex_view_label(view)),
                    format!("{} ({})", ctx.series_name, complex_view_label(view)),
                    values,
                    rows.len(),
                ))
            })
            .collect();
    }

    let y_values = ctx
        .ds_y
        .as_numeric()
        .context("Expected numeric array")?
        .values();
    let mut values = Vec::with_capacity(rows.len() * 2);
    for row in rows.iter().copied() {
        values.push(resolve_quantity_vs_sweep_x(
            ctx.x_values,
            ctx.row_start,
            row,
        )?);
        values.push(y_values[row]);
    }
    Ok(vec![FlatXYSeries::new(
        ctx.series_name.to_string(),
        ctx.series_name.to_string(),
        values,
        rows.len(),
    )])
}

fn build_live_scalar_quantity_vs_sweep_groups(
    ctx: &LiveScalarQuantityVsSweepContext<'_>,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
) -> Result<Vec<FlatXYSeries>> {
    let groups = compute_group_starts(ctx.batch, ctx.schema, &ctx.roles.trace_group);
    let ranges = group_ranges(&groups, ctx.batch.num_rows());
    let selected = &ranges[ranges.len().saturating_sub(ctx.tail_count)..];

    if is_complex {
        let complex_array = ctx.ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        let views = resolved_complex_views(complex_views);
        let mut result = Vec::new();

        for &(group_start, group_end) in selected {
            let rows = row_order_for_group(
                ctx.batch,
                ctx.schema,
                group_start,
                group_end,
                ctx.roles.sweep,
            );
            let group_label =
                make_group_label(ctx.batch, ctx.schema, &ctx.roles.trace_group, group_start);
            let group_id =
                make_group_id_suffix(ctx.batch, ctx.schema, &ctx.roles.trace_group, group_start);
            for &view in &views {
                let transformed = transform_complex_values(reals, imags, view);
                let mut values = Vec::with_capacity(rows.len() * 2);
                for row in rows.iter().copied() {
                    values.push(resolve_quantity_vs_sweep_x(
                        ctx.x_values,
                        ctx.row_start,
                        row,
                    )?);
                    values.push(transformed[row]);
                }
                result.push(FlatXYSeries::new(
                    format!(
                        "{}:{}:{}",
                        super::group_series_id(group_start),
                        group_id.clone().unwrap_or_default(),
                        complex_view_label(view)
                    ),
                    semantic_live_label(
                        &format!("{} ({})", ctx.series_name, complex_view_label(view)),
                        group_label.as_deref(),
                    ),
                    values,
                    rows.len(),
                ));
            }
        }

        return Ok(result);
    }

    let y_values = ctx
        .ds_y
        .as_numeric()
        .context("Expected numeric array")?
        .values();
    let mut result = Vec::new();
    for &(group_start, group_end) in selected {
        let rows = row_order_for_group(
            ctx.batch,
            ctx.schema,
            group_start,
            group_end,
            ctx.roles.sweep,
        );
        let group_label =
            make_group_label(ctx.batch, ctx.schema, &ctx.roles.trace_group, group_start);
        let mut values = Vec::with_capacity(rows.len() * 2);
        for row in rows.iter().copied() {
            values.push(resolve_quantity_vs_sweep_x(
                ctx.x_values,
                ctx.row_start,
                row,
            )?);
            values.push(y_values[row]);
        }
        result.push(FlatXYSeries::new(
            super::group_series_id(group_start),
            semantic_live_label(ctx.series_name, group_label.as_deref()),
            values,
            rows.len(),
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::charts::{
        transform::test_utils::{numeric_batch, numeric_schema},
        types::{LiveChartDataOptions, XYDrawStyle, XYPlotModeOptions, XYTraceRoleOptions},
    };

    fn xy_snapshot(snapshot: ChartSnapshot) -> XYChartSnapshot {
        match snapshot {
            ChartSnapshot::Xy(snapshot) => snapshot,
            other @ ChartSnapshot::Heatmap(_) => {
                panic!("expected XY snapshot, got {other:?}")
            }
        }
    }

    #[test]
    fn live_scalar_xy_without_grouping_appends_to_single_series_shape() {
        let batch = numeric_batch(&[
            ("t", &[0.0, 1.0, 2.0, 3.0]),
            ("x", &[0.0, 1.0, 2.0, 3.0]),
            ("y", &[10.0, 11.0, 12.0, 13.0]),
        ]);
        let schema = numeric_schema(&["t", "x", "y"]);

        let snapshot = xy_snapshot(
            build_live_xy_series(
                &batch,
                &schema,
                Some(&[0]),
                0,
                match &LiveChartDataOptions::Xy(LiveXYOptions {
                    draw_style: XYDrawStyle::Line,
                    tail_count: 2,
                    known_row_count: None,
                    plot_mode: XYPlotModeOptions::Xy {
                        x_column: "x".to_string(),
                        y_column: "y".to_string(),
                    },
                    trace_roles: XYTraceRoleOptions::default(),
                }) {
                    LiveChartDataOptions::Xy(options) => options,
                    LiveChartDataOptions::Heatmap(_) => unreachable!(),
                },
            )
            .unwrap(),
        );

        assert_eq!(snapshot.series.len(), 1);
        assert_eq!(snapshot.series[0].point_count, 2);
    }

    #[test]
    fn live_quantity_vs_sweep_without_sweep_column_keeps_absolute_row_axis() {
        let batch = numeric_batch(&[("value", &[10.0, 11.0, 12.0])]);
        let schema = numeric_schema(&["value"]);

        let snapshot = xy_snapshot(
            build_live_xy_series(
                &batch,
                &schema,
                None,
                5,
                match &LiveChartDataOptions::Xy(LiveXYOptions {
                    draw_style: XYDrawStyle::Line,
                    tail_count: 3,
                    known_row_count: None,
                    plot_mode: XYPlotModeOptions::QuantityVsSweep {
                        quantity: "value".to_string(),
                        complex_views: None,
                    },
                    trace_roles: XYTraceRoleOptions::default(),
                }) {
                    LiveChartDataOptions::Xy(options) => options,
                    LiveChartDataOptions::Heatmap(_) => unreachable!(),
                },
            )
            .unwrap(),
        );

        assert_eq!(snapshot.x_name, "row");
        assert_eq!(
            snapshot.series[0].values,
            vec![5.0, 10.0, 6.0, 11.0, 7.0, 12.0]
        );
    }
}

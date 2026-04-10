use anyhow::{Context, Result, bail};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};

use super::{
    XYTraceRoles, compute_group_starts, format_numeric_value, group_ranges, make_group_id_suffix,
    make_group_label, resolve_xy_trace_roles, row_order_for_group, row_series_id,
};
use crate::features::charts::types::{
    ChartSnapshot, ComplexViewOption, FlatXYSeries, XYChartDataOptions, XYChartSnapshot,
    XYDrawStyle, XYPlotMode, XYPlotModeOptions, XYTraceRoleOptions, complex_view_label,
    transform_complex_values,
};

pub(crate) fn build_xy_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &XYChartDataOptions,
) -> Result<ChartSnapshot> {
    let snapshot = match &options.plot_mode {
        XYPlotModeOptions::QuantityVsSweep {
            quantity,
            complex_views,
        } => build_quantity_vs_sweep_snapshot(
            batch,
            schema,
            index_columns,
            options.draw_style,
            quantity,
            complex_views.as_deref().unwrap_or(&[]),
            &options.trace_roles,
        )?,
        XYPlotModeOptions::Xy { x_column, y_column } => build_xy_snapshot(
            batch,
            schema,
            index_columns,
            options.draw_style,
            x_column,
            y_column,
            &options.trace_roles,
        )?,
        XYPlotModeOptions::ComplexPlane { quantity } => build_complex_plane_snapshot(
            batch,
            schema,
            index_columns,
            options.draw_style,
            quantity,
            &options.trace_roles,
        )?,
    };

    Ok(ChartSnapshot::Xy(snapshot))
}

fn build_quantity_vs_sweep_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    draw_style: XYDrawStyle,
    series_name: &str,
    complex_views: &[ComplexViewOption],
    trace_roles: &XYTraceRoleOptions,
) -> Result<XYChartSnapshot> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();

    let series = if is_trace {
        build_trace_quantity_vs_sweep_series(batch, series_name, is_complex, complex_views)?
    } else {
        let roles = resolve_xy_trace_roles(schema, index_columns, trace_roles, draw_style)?;
        build_scalar_quantity_vs_sweep_series(
            batch,
            schema,
            series_name,
            is_complex,
            complex_views,
            &roles,
        )?
    };

    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        resolve_quantity_vs_sweep_x_name(schema, index_columns, trace_roles, draw_style)?
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::QuantityVsSweep,
        draw_style,
        x_name,
        y_name: None,
        series,
    })
}

fn build_xy_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    draw_style: XYDrawStyle,
    x_column: &str,
    y_column: &str,
    trace_roles: &XYTraceRoleOptions,
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

    let series = match (x_is_trace, y_is_trace) {
        (true, true) => build_trace_xy_series(batch, x_column, y_column)?,
        (false, false) => {
            let roles = resolve_xy_trace_roles(schema, index_columns, trace_roles, draw_style)?;
            build_scalar_xy_series(batch, schema, x_column, y_column, &roles)?
        }
        _ => bail!("X/Y plot mode requires both columns to be trace or both to be scalar"),
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::Xy,
        draw_style,
        x_name: x_column.to_string(),
        y_name: Some(y_column.to_string()),
        series,
    })
}

fn build_complex_plane_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    draw_style: XYDrawStyle,
    series_name: &str,
    trace_roles: &XYTraceRoleOptions,
) -> Result<XYChartSnapshot> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));

    let series = if is_trace {
        build_trace_complex_plane_series(batch, schema, series_name)?
    } else {
        let roles = resolve_xy_trace_roles(schema, index_columns, trace_roles, draw_style)?;
        build_scalar_complex_plane_series(batch, schema, series_name, &roles)?
    };

    Ok(XYChartSnapshot {
        plot_mode: XYPlotMode::ComplexPlane,
        draw_style,
        x_name: format!("{series_name} (real)"),
        y_name: Some(format!("{series_name} (imag)")),
        series,
    })
}

fn build_trace_quantity_vs_sweep_series(
    batch: &RecordBatch,
    series_name: &str,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
) -> Result<Vec<FlatXYSeries>> {
    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let num_rows = batch.num_rows();
    let view_options = resolved_complex_views(is_complex, complex_views);
    let mut result = Vec::new();

    for row in 0..num_rows {
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
            for &view in &view_options {
                let y_values = transform_complex_values(reals, imags, view);
                result.push(make_trace_series(
                    row,
                    num_rows,
                    &format!("{series_name}:{}", complex_view_label(view)),
                    &format!("{series_name} ({})", complex_view_label(view)),
                    &x_values,
                    &y_values,
                )?);
            }
        } else {
            let y_values = ds_y
                .as_numeric()
                .context("Expected numeric array")?
                .values()
                .to_vec();
            result.push(make_trace_series(
                row,
                num_rows,
                series_name,
                series_name,
                &x_values,
                &y_values,
            )?);
        }
    }

    Ok(result)
}

fn build_trace_xy_series(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
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
    let mut result = Vec::new();
    for row in 0..num_rows {
        let Some((_trace_axis, x_values_array)) = x_array.expand_trace(row)? else {
            continue;
        };
        let Some((_trace_axis, y_values_array)) = y_array.expand_trace(row)? else {
            continue;
        };
        let ds_x: DatasetArray = x_values_array.try_into()?;
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let x_values = ds_x
            .as_numeric()
            .context("X trace must be numeric")?
            .values();
        let y_values = ds_y
            .as_numeric()
            .context("Y trace must be numeric")?
            .values();
        let len = x_values.len().min(y_values.len());
        let mut values = Vec::with_capacity(len * 2);
        for i in 0..len {
            values.push(x_values[i]);
            values.push(y_values[i]);
        }
        result.push(FlatXYSeries::new(
            row_series_id(row),
            row_label(&format!("{x_column} vs {y_column}"), row, num_rows)?,
            values,
            len,
        ));
    }
    Ok(result)
}

fn build_trace_complex_plane_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
) -> Result<Vec<FlatXYSeries>> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        bail!("Complex plane trace plot mode requires a trace quantity");
    }

    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let num_rows = batch.num_rows();
    let mut result = Vec::new();

    for row in 0..num_rows {
        let Some((_trace_axis, trace_values)) = series_array.expand_trace(row)? else {
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
            row_label(series_name, row, num_rows)?,
            values,
            len,
        ));
    }

    Ok(result)
}

fn build_scalar_quantity_vs_sweep_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
    is_complex: bool,
    complex_views: &[ComplexViewOption],
    roles: &XYTraceRoles,
) -> Result<Vec<FlatXYSeries>> {
    let series_column = batch
        .column_by_name(series_name)
        .cloned()
        .context("Series column not found")?;
    let ds_y: DatasetArray = series_column.try_into()?;
    let groups = compute_group_starts(batch, schema, &roles.trace_group);
    let ranges = group_ranges(&groups, batch.num_rows());
    let view_options = resolved_complex_views(is_complex, complex_views);
    let x_values = roles
        .sweep
        .map(|index| numeric_column_values(batch, schema, index))
        .transpose()?;

    let mut result = Vec::new();
    if is_complex {
        let complex_array = ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        for (group_start, group_end) in ranges {
            let row_order = row_order_for_group(batch, schema, group_start, group_end, roles.sweep);
            let group_label = make_group_label(batch, schema, &roles.trace_group, group_start);
            let group_id = make_group_id_suffix(batch, schema, &roles.trace_group, group_start);
            for &view in &view_options {
                let mut values = Vec::with_capacity(row_order.len() * 2);
                let transformed = transform_complex_values(reals, imags, view);
                for (position, row) in row_order.iter().copied().enumerate() {
                    values.push(resolve_scalar_quantity_vs_sweep_x(
                        x_values.as_deref(),
                        position,
                        row,
                    )?);
                    values.push(transformed[row]);
                }
                result.push(FlatXYSeries::new(
                    semantic_series_id(
                        &format!("{series_name}:{}", complex_view_label(view)),
                        group_id.as_deref(),
                    ),
                    semantic_series_label(
                        &format!("{series_name} ({})", complex_view_label(view)),
                        group_label.as_deref(),
                    ),
                    values,
                    row_order.len(),
                ));
            }
        }
    } else {
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        for (group_start, group_end) in ranges {
            let row_order = row_order_for_group(batch, schema, group_start, group_end, roles.sweep);
            let group_label = make_group_label(batch, schema, &roles.trace_group, group_start);
            let group_id = make_group_id_suffix(batch, schema, &roles.trace_group, group_start);
            let mut values = Vec::with_capacity(row_order.len() * 2);
            for (position, row) in row_order.iter().copied().enumerate() {
                values.push(resolve_scalar_quantity_vs_sweep_x(
                    x_values.as_deref(),
                    position,
                    row,
                )?);
                values.push(y_values[row]);
            }
            result.push(FlatXYSeries::new(
                semantic_series_id(series_name, group_id.as_deref()),
                semantic_series_label(series_name, group_label.as_deref()),
                values,
                row_order.len(),
            ));
        }
    }

    Ok(result)
}

fn build_scalar_xy_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    x_column: &str,
    y_column: &str,
    roles: &XYTraceRoles,
) -> Result<Vec<FlatXYSeries>> {
    let x_values = batch
        .column_by_name(x_column)
        .cloned()
        .context("X column not found")?;
    let y_values = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y column not found")?;
    let ds_x: DatasetArray = x_values.try_into()?;
    let ds_y: DatasetArray = y_values.try_into()?;
    let x_numeric = ds_x.as_numeric().context("X must be numeric")?.values();
    let y_numeric = ds_y.as_numeric().context("Y must be numeric")?.values();

    Ok(build_grouped_xy_series(
        batch,
        schema,
        &format!("{x_column} vs {y_column}"),
        roles,
        |row| (x_numeric[row], y_numeric[row]),
    ))
}

fn build_scalar_complex_plane_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
    roles: &XYTraceRoles,
) -> Result<Vec<FlatXYSeries>> {
    let series_column = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?;
    let ds_series: DatasetArray = series_column.try_into()?;
    let complex_array = ds_series.as_complex().context("Expected complex array")?;
    let reals = complex_array.real().values();
    let imags = complex_array.imag().values();

    Ok(build_grouped_xy_series(
        batch,
        schema,
        series_name,
        roles,
        |row| (reals[row], imags[row]),
    ))
}

fn build_grouped_xy_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    base_label: &str,
    roles: &XYTraceRoles,
    point_at: impl Fn(usize) -> (f64, f64),
) -> Vec<FlatXYSeries> {
    let groups = compute_group_starts(batch, schema, &roles.trace_group);
    let ranges = group_ranges(&groups, batch.num_rows());
    let mut result = Vec::new();

    for (group_start, group_end) in ranges {
        let row_order = row_order_for_group(batch, schema, group_start, group_end, roles.sweep);
        let group_label = make_group_label(batch, schema, &roles.trace_group, group_start);
        let group_id = make_group_id_suffix(batch, schema, &roles.trace_group, group_start);
        let mut values = Vec::with_capacity(row_order.len() * 2);
        for row in row_order.iter().copied() {
            let (x, y) = point_at(row);
            values.push(x);
            values.push(y);
        }
        result.push(FlatXYSeries::new(
            semantic_series_id(base_label, group_id.as_deref()),
            semantic_series_label(base_label, group_label.as_deref()),
            values,
            row_order.len(),
        ));
    }

    result
}

fn resolve_quantity_vs_sweep_x_name(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    trace_roles: &XYTraceRoleOptions,
    draw_style: XYDrawStyle,
) -> Result<String> {
    let roles = resolve_xy_trace_roles(schema, index_columns, trace_roles, draw_style)?;
    Ok(match roles.sweep {
        Some(index) => schema
            .columns()
            .get_index(index)
            .map(|(name, _)| name.clone())
            .context("Order index column not found")?,
        None => "row".to_string(),
    })
}

fn resolved_complex_views(
    is_complex: bool,
    complex_views: &[ComplexViewOption],
) -> Vec<ComplexViewOption> {
    if !is_complex {
        return vec![];
    }
    if complex_views.is_empty() {
        vec![ComplexViewOption::Real, ComplexViewOption::Imag]
    } else {
        complex_views.to_vec()
    }
}

fn numeric_column_values(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index: usize,
) -> Result<Vec<f64>> {
    let name = schema
        .columns()
        .get_index(index)
        .map(|(name, _)| name.clone())
        .context("Column index not found")?;
    let arr = batch
        .column_by_name(&name)
        .cloned()
        .context("Column not found")?;
    let ds: DatasetArray = arr.try_into()?;
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

fn resolve_scalar_quantity_vs_sweep_x(
    x_values: Option<&[f64]>,
    position: usize,
    row: usize,
) -> Result<f64> {
    Ok(x_values
        .and_then(|values| values.get(row).copied())
        .unwrap_or(row_axis_value(position)?))
}

fn make_trace_series(
    row: usize,
    total_rows: usize,
    id_base: &str,
    label_base: &str,
    x_values: &[f64],
    y_values: &[f64],
) -> Result<FlatXYSeries> {
    let len = x_values.len().min(y_values.len());
    let mut values = Vec::with_capacity(len * 2);
    for i in 0..len {
        values.push(x_values[i]);
        values.push(y_values[i]);
    }
    Ok(FlatXYSeries::new(
        format!("{}:{}", row_series_id(row), id_base),
        row_label(label_base, row, total_rows)?,
        values,
        len,
    ))
}

fn semantic_series_id(base: &str, group_suffix: Option<&str>) -> String {
    match group_suffix {
        Some(group_suffix) => format!("{}:{}", base.replace(' ', "_"), group_suffix),
        None => base.replace(' ', "_"),
    }
}

fn semantic_series_label(base: &str, group_label: Option<&str>) -> String {
    match group_label {
        Some(group_label) => format!("{base} [{group_label}]"),
        None => base.to_string(),
    }
}

fn row_label(base: &str, row: usize, total_rows: usize) -> Result<String> {
    if total_rows <= 1 {
        Ok(base.to_string())
    } else {
        Ok(format!(
            "{base} [row {}]",
            format_numeric_value(row_axis_value(row)?)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::charts::{
        transform::test_utils::{numeric_batch, numeric_schema},
        types::{
            ChartCommonOptions, XYChartDataOptions, XYDrawStyle, XYPlotModeOptions,
            XYTraceRoleOptions,
        },
    };

    fn xy_snapshot(snapshot: ChartSnapshot) -> XYChartSnapshot {
        match snapshot {
            ChartSnapshot::Xy(snapshot) => snapshot,
            other @ ChartSnapshot::Heatmap(_) => {
                panic!("expected XY snapshot, got {other:?}")
            }
        }
    }

    fn xy_points(series: &FlatXYSeries) -> Vec<Vec<f64>> {
        series
            .values
            .chunks_exact(2)
            .map(|point| vec![point[0], point[1]])
            .collect()
    }

    #[test]
    fn builds_scalar_quantity_vs_sweep_from_sweep_index() {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 2.0, 2.0]),
            ("t", &[0.0, 1.0, 0.0, 1.0]),
            ("y", &[10.0, 11.0, 20.0, 21.0]),
        ]);
        let schema = numeric_schema(&["sweep", "t", "y"]);

        let snapshot = xy_snapshot(
            build_xy_series(
                &batch,
                &schema,
                Some(&[0, 1]),
                &XYChartDataOptions {
                    draw_style: XYDrawStyle::Line,
                    plot_mode: XYPlotModeOptions::QuantityVsSweep {
                        quantity: "y".to_string(),
                        complex_views: None,
                    },
                    trace_roles: XYTraceRoleOptions {
                        trace_group_index_columns: Some(vec!["sweep".to_string()]),
                        sweep_index_column: Some("t".to_string()),
                    },
                    common: ChartCommonOptions::default(),
                },
            )
            .unwrap(),
        );

        assert_eq!(snapshot.plot_mode, XYPlotMode::QuantityVsSweep);
        assert_eq!(snapshot.x_name, "t");
        assert_eq!(snapshot.series.len(), 2);
        assert_eq!(
            xy_points(&snapshot.series[0]),
            vec![vec![0.0, 10.0], vec![1.0, 11.0]]
        );
    }

    #[test]
    fn builds_scalar_xy_grouped_series() {
        let batch = numeric_batch(&[
            ("outer", &[1.0, 1.0, 2.0, 2.0]),
            ("inner", &[0.0, 1.0, 0.0, 1.0]),
            ("x", &[0.0, 1.0, 0.0, 1.0]),
            ("y", &[10.0, 11.0, 20.0, 21.0]),
        ]);
        let schema = numeric_schema(&["outer", "inner", "x", "y"]);

        let snapshot = xy_snapshot(
            build_xy_series(
                &batch,
                &schema,
                Some(&[0, 1]),
                &XYChartDataOptions {
                    draw_style: XYDrawStyle::Points,
                    plot_mode: XYPlotModeOptions::Xy {
                        x_column: "x".to_string(),
                        y_column: "y".to_string(),
                    },
                    trace_roles: XYTraceRoleOptions {
                        trace_group_index_columns: Some(vec!["outer".to_string()]),
                        sweep_index_column: None,
                    },
                    common: ChartCommonOptions::default(),
                },
            )
            .unwrap(),
        );

        assert_eq!(snapshot.series.len(), 2);
        assert_eq!(
            xy_points(&snapshot.series[1]),
            vec![vec![0.0, 20.0], vec![1.0, 21.0]]
        );
    }

    #[test]
    fn builds_complex_plane_from_scalar_complex_series() {
        let batch = numeric_batch(&[("sweep", &[1.0, 1.0, 2.0]), ("value", &[1.0, 2.0, 3.0])]);
        let schema = numeric_schema(&["sweep", "value"]);

        let result = build_xy_series(
            &batch,
            &schema,
            Some(&[0]),
            &XYChartDataOptions {
                draw_style: XYDrawStyle::Points,
                plot_mode: XYPlotModeOptions::ComplexPlane {
                    quantity: "value".to_string(),
                },
                trace_roles: XYTraceRoleOptions::default(),
                common: ChartCommonOptions::default(),
            },
        );

        assert!(result.is_err());
    }
}

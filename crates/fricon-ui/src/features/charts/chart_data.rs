use anyhow::Context;
use arrow_array::RecordBatch;
use fricon::{DatasetDataType, DatasetSchema};
use tracing::{debug, error, instrument};

use super::{
    filter_table::build_semantic_filters, semantic_source, types::DatasetChartDataOptions,
};
use crate::{
    desktop_runtime::session::WorkspaceSession,
    features::charts::{
        transform::{
            SemanticRoleColumns, build_heatmap_series, build_live_heatmap_series,
            build_live_xy_series, build_xy_series, compute_group_starts,
            mapping::{build_chart_selected_columns, build_live_chart_selected_columns},
            resolve_xy_trace_roles,
        },
        types::{
            ChartCommonOptions, ChartSnapshot, FlatSeries, FlatXYSeries, HeatmapChartDataOptions,
            HeatmapChartSnapshot, LiveChartAppendOperation, LiveChartDataOptions,
            LiveChartDataResponse, LiveHeatmapOptions, LiveXYOptions, XYChartDataOptions,
            XYPlotModeOptions, XYTraceRoleOptions,
        },
    },
};

fn recent_group_starts_in_scan_batch(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    row_indices: &[usize],
    range_start: usize,
) -> Vec<usize> {
    compute_group_starts(batch, schema, grouping_index_columns)
        .into_iter()
        .filter_map(|offset| row_indices.get(offset).copied())
        .filter(|&row| row >= range_start)
        .collect()
}

fn resolve_group_tail_start_in_scan_batch(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    row_indices: &[usize],
    range_start: usize,
    required_groups: usize,
) -> Option<usize> {
    let starts = recent_group_starts_in_scan_batch(
        batch,
        schema,
        grouping_index_columns,
        row_indices,
        range_start,
    );
    (starts.len() >= required_groups).then(|| starts[starts.len() - required_groups])
}

async fn prepare_live_range(
    session: &WorkspaceSession,
    id: i32,
    start: usize,
    end: usize,
    selected_columns: Option<&[usize]>,
) -> anyhow::Result<semantic_source::PreparedChartData> {
    let common = ChartCommonOptions {
        start: Some(start),
        end: Some(end),
        index_filters: None,
        exclude_columns: None,
    };
    semantic_source::prepare_chart_data(session, id, &common, &[], selected_columns).await
}

async fn resolve_group_tail_start(
    session: &WorkspaceSession,
    id: i32,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    total_rows: usize,
    required_groups: usize,
) -> anyhow::Result<usize> {
    if total_rows == 0 || required_groups == 0 {
        return Ok(0);
    }

    let grouping_column_names = column_names(schema, grouping_index_columns)?;
    let mut window_rows = required_groups.max(1);
    loop {
        let range_start = total_rows.saturating_sub(window_rows);
        let scan_start = range_start.saturating_sub(1);
        let prepared = prepare_live_range(
            session,
            id,
            scan_start,
            total_rows,
            Some(grouping_index_columns),
        )
        .await?;
        let projected_grouping_index_columns =
            column_indices(&prepared.schema, &grouping_column_names)?;

        if let Some(start) = resolve_group_tail_start_in_scan_batch(
            &prepared.batch,
            &prepared.schema,
            &projected_grouping_index_columns,
            &prepared.row_indices,
            range_start,
            required_groups,
        ) {
            return Ok(start);
        }

        if range_start == 0 {
            return Ok(0);
        }

        window_rows = window_rows.saturating_mul(2).min(total_rows);
    }
}

fn column_names(schema: &DatasetSchema, columns: &[usize]) -> anyhow::Result<Vec<String>> {
    columns
        .iter()
        .map(|&index| {
            schema
                .columns()
                .get_index(index)
                .map(|(name, _)| name.clone())
                .with_context(|| format!("Column index '{index}' not found"))
        })
        .collect()
}

fn column_indices(schema: &DatasetSchema, columns: &[String]) -> anyhow::Result<Vec<usize>> {
    columns
        .iter()
        .map(|name| {
            schema
                .columns()
                .get_full(name)
                .map(|(index, _, _)| index)
                .with_context(|| format!("Column '{name}' not found"))
        })
        .collect()
}

fn plot_mode_is_trace(
    schema: &DatasetSchema,
    plot_mode: &XYPlotModeOptions,
) -> anyhow::Result<bool> {
    match plot_mode {
        XYPlotModeOptions::QuantityVsSweep { quantity, .. }
        | XYPlotModeOptions::ComplexPlane { quantity } => Ok(matches!(
            schema.columns().get(quantity).context("Column not found")?,
            DatasetDataType::Trace(_, _)
        )),
        XYPlotModeOptions::Xy { x_column, y_column } => {
            let x_type = *schema
                .columns()
                .get(x_column)
                .context("X column not found")?;
            let y_type = *schema
                .columns()
                .get(y_column)
                .context("Y column not found")?;
            Ok(matches!(x_type, DatasetDataType::Trace(_, _))
                && matches!(y_type, DatasetDataType::Trace(_, _)))
        }
    }
}

async fn resolve_live_row_start(
    session: &WorkspaceSession,
    id: i32,
    total_rows: usize,
    options: &LiveChartDataOptions,
) -> anyhow::Result<usize> {
    if total_rows == 0 {
        return Ok(0);
    }

    let prepared = prepare_live_range(session, id, 0, 0, None).await?;
    let schema = &prepared.schema;
    let index_columns = prepared.index_columns.as_deref();
    match options {
        LiveChartDataOptions::Xy(opts) => {
            let tail_count = opts.tail_count.max(1);
            if plot_mode_is_trace(schema, &opts.plot_mode)? {
                return Ok(total_rows.saturating_sub(tail_count));
            }

            let roles = resolve_xy_trace_roles(
                schema,
                SemanticRoleColumns::new(index_columns, prepared.group_columns.as_deref()),
                &opts.trace_roles,
                opts.draw_style,
            )?;
            if roles.trace_group.is_empty() {
                Ok(total_rows.saturating_sub(tail_count))
            } else {
                resolve_group_tail_start(
                    session,
                    id,
                    schema,
                    &roles.trace_group,
                    total_rows,
                    tail_count,
                )
                .await
            }
        }
        LiveChartDataOptions::Heatmap(opts) => {
            let data_type = *schema
                .columns()
                .get(&opts.quantity)
                .context("Column not found")?;
            if matches!(data_type, DatasetDataType::Trace(_, _)) {
                match index_columns {
                    Some(idx_cols) if idx_cols.len() >= 2 => {
                        resolve_group_tail_start(
                            session,
                            id,
                            schema,
                            &idx_cols[..idx_cols.len() - 1],
                            total_rows,
                            1,
                        )
                        .await
                    }
                    _ => Ok(total_rows.saturating_sub(1)),
                }
            } else if let Some(idx_cols) = index_columns {
                if idx_cols.len() >= 3 {
                    resolve_group_tail_start(
                        session,
                        id,
                        schema,
                        &idx_cols[..idx_cols.len() - 2],
                        total_rows,
                        1,
                    )
                    .await
                } else {
                    Ok(0)
                }
            } else {
                Ok(0)
            }
        }
    }
}

#[instrument(level = "debug", skip(session, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    session: &WorkspaceSession,
    id: i32,
    options: &DatasetChartDataOptions,
) -> anyhow::Result<ChartSnapshot> {
    let common = options.common();
    let filters = if let Some(indices) = common.index_filters.as_deref() {
        build_semantic_filters(session, id, common.exclude_columns.clone(), indices)
            .await
            .context("Failed to build semantic index filters")?
    } else {
        Vec::new()
    };
    let metadata = prepare_live_range(session, id, 0, 0, None).await?;
    let resolved_options = resolve_dataset_chart_options(&metadata, options);
    let selected_columns = build_chart_selected_columns(
        &metadata.schema,
        metadata.index_columns.as_deref(),
        metadata.group_columns.as_deref(),
        &resolved_options,
    )?;
    let prepared =
        semantic_source::prepare_chart_data(session, id, common, &filters, Some(&selected_columns))
            .await?;
    let chart_type = options.view_name();
    debug!(
        dataset_id = id,
        chart_type,
        rows = prepared.batch.num_rows(),
        cols = prepared.batch.num_columns(),
        "Building dataset chart data"
    );

    let result = match &resolved_options {
        DatasetChartDataOptions::Xy(options) => build_xy_series(
            &prepared.batch,
            &prepared.schema,
            prepared.index_columns.as_deref(),
            prepared.group_columns.as_deref(),
            options,
        ),
        DatasetChartDataOptions::Heatmap(options) => {
            build_heatmap_series(&prepared.batch, &prepared.schema, options)
        }
    };
    if let Err(err) = &result {
        error!(
            dataset_id = id,
            chart_type,
            rows = prepared.batch.num_rows(),
            cols = prepared.batch.num_columns(),
            error = %err,
            "Failed to build dataset chart data"
        );
    }
    result
}

#[instrument(level = "debug", skip(session, options), fields(dataset_id = id))]
pub(crate) async fn dataset_live_chart_data(
    session: &WorkspaceSession,
    id: i32,
    options: &LiveChartDataOptions,
) -> anyhow::Result<LiveChartDataResponse> {
    let dataset = session.dataset(id).await?;
    let total_rows = dataset.num_rows();
    let metadata = prepare_live_range(session, id, 0, 0, None).await?;
    let resolved_options = resolve_live_chart_options(&metadata, options);
    let start = resolve_live_row_start(session, id, total_rows, &resolved_options).await?;
    let selected_columns = build_live_chart_selected_columns(
        &metadata.schema,
        metadata.index_columns.as_deref(),
        metadata.group_columns.as_deref(),
        &resolved_options,
    )?;
    let prepared =
        prepare_live_range(session, id, start, total_rows, Some(&selected_columns)).await?;
    debug!(
        dataset_id = id,
        start,
        end = total_rows,
        rows = prepared.batch.num_rows(),
        cols = prepared.batch.num_columns(),
        "Building live chart data"
    );

    let snapshot = build_live_snapshot(
        &prepared.batch,
        &prepared.schema,
        prepared.index_columns.as_deref(),
        prepared.group_columns.as_deref(),
        start,
        &resolved_options,
    );
    if let Err(err) = &snapshot {
        error!(
            dataset_id = id,
            rows = prepared.batch.num_rows(),
            error = %err,
            "Failed to build live chart data"
        );
    }
    let snapshot = snapshot?;

    let Some(known_row_count) = options.known_row_count() else {
        return Ok(LiveChartDataResponse::Reset {
            row_count: total_rows,
            snapshot,
        });
    };

    if known_row_count == 0 || known_row_count > total_rows {
        return Ok(LiveChartDataResponse::Reset {
            row_count: total_rows,
            snapshot,
        });
    }

    if known_row_count == total_rows {
        return Ok(LiveChartDataResponse::Append {
            row_count: total_rows,
            ops: vec![],
        });
    }

    let previous_start =
        resolve_live_row_start(session, id, known_row_count, &resolved_options).await?;
    if previous_start != start {
        return Ok(LiveChartDataResponse::Reset {
            row_count: total_rows,
            snapshot,
        });
    }

    let previous_prepared = prepare_live_range(
        session,
        id,
        previous_start,
        known_row_count,
        Some(&selected_columns),
    )
    .await?;
    let previous_snapshot = build_live_snapshot(
        &previous_prepared.batch,
        &previous_prepared.schema,
        previous_prepared.index_columns.as_deref(),
        previous_prepared.group_columns.as_deref(),
        previous_start,
        &resolved_options,
    )?;

    let Some(ops) = diff_live_snapshots(&previous_snapshot, &snapshot) else {
        return Ok(LiveChartDataResponse::Reset {
            row_count: total_rows,
            snapshot,
        });
    };

    Ok(LiveChartDataResponse::Append {
        row_count: total_rows,
        ops,
    })
}

fn build_live_snapshot(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    group_columns: Option<&[usize]>,
    row_start: usize,
    options: &LiveChartDataOptions,
) -> anyhow::Result<ChartSnapshot> {
    match options {
        LiveChartDataOptions::Xy(opts) => {
            build_live_xy_series(batch, schema, index_columns, group_columns, row_start, opts)
        }
        LiveChartDataOptions::Heatmap(opts) => {
            build_live_heatmap_series(batch, schema, index_columns, opts)
        }
    }
}

fn resolve_dataset_chart_options(
    metadata: &semantic_source::PreparedChartData,
    options: &DatasetChartDataOptions,
) -> DatasetChartDataOptions {
    match options {
        DatasetChartDataOptions::Xy(options) => {
            DatasetChartDataOptions::Xy(resolve_xy_chart_options(metadata, options))
        }
        DatasetChartDataOptions::Heatmap(options) => {
            DatasetChartDataOptions::Heatmap(HeatmapChartDataOptions {
                quantity: metadata.resolve_column_name(&options.quantity),
                x_column: options
                    .x_column
                    .as_deref()
                    .map(|column| metadata.resolve_column_name(column)),
                y_column: metadata.resolve_column_name(&options.y_column),
                complex_view_single: options.complex_view_single,
                common: options.common.clone(),
            })
        }
    }
}

fn resolve_live_chart_options(
    metadata: &semantic_source::PreparedChartData,
    options: &LiveChartDataOptions,
) -> LiveChartDataOptions {
    match options {
        LiveChartDataOptions::Xy(options) => LiveChartDataOptions::Xy(LiveXYOptions {
            draw_style: options.draw_style,
            tail_count: options.tail_count,
            known_row_count: options.known_row_count,
            plot_mode: resolve_xy_plot_mode_options(metadata, &options.plot_mode),
            trace_roles: resolve_trace_roles(metadata, &options.trace_roles),
        }),
        LiveChartDataOptions::Heatmap(options) => {
            LiveChartDataOptions::Heatmap(LiveHeatmapOptions {
                quantity: metadata.resolve_column_name(&options.quantity),
                complex_view_single: options.complex_view_single,
                known_row_count: options.known_row_count,
            })
        }
    }
}

fn resolve_xy_chart_options(
    metadata: &semantic_source::PreparedChartData,
    options: &XYChartDataOptions,
) -> XYChartDataOptions {
    XYChartDataOptions {
        draw_style: options.draw_style,
        plot_mode: resolve_xy_plot_mode_options(metadata, &options.plot_mode),
        trace_roles: resolve_trace_roles(metadata, &options.trace_roles),
        common: options.common.clone(),
    }
}

fn resolve_xy_plot_mode_options(
    metadata: &semantic_source::PreparedChartData,
    options: &XYPlotModeOptions,
) -> XYPlotModeOptions {
    match options {
        XYPlotModeOptions::QuantityVsSweep {
            quantity,
            complex_views,
        } => XYPlotModeOptions::QuantityVsSweep {
            quantity: metadata.resolve_column_name(quantity),
            complex_views: complex_views.clone(),
        },
        XYPlotModeOptions::Xy { x_column, y_column } => XYPlotModeOptions::Xy {
            x_column: metadata.resolve_column_name(x_column),
            y_column: metadata.resolve_column_name(y_column),
        },
        XYPlotModeOptions::ComplexPlane { quantity } => XYPlotModeOptions::ComplexPlane {
            quantity: metadata.resolve_column_name(quantity),
        },
    }
}

fn resolve_trace_roles(
    metadata: &semantic_source::PreparedChartData,
    options: &XYTraceRoleOptions,
) -> XYTraceRoleOptions {
    XYTraceRoleOptions {
        trace_group_index_columns: options.trace_group_index_columns.as_ref().map(|columns| {
            columns
                .iter()
                .map(|column| metadata.resolve_column_name(column))
                .collect()
        }),
        sweep_index_column: options
            .sweep_index_column
            .as_deref()
            .map(|column| metadata.resolve_column_name(column)),
    }
}

fn diff_live_snapshots(
    previous: &ChartSnapshot,
    current: &ChartSnapshot,
) -> Option<Vec<LiveChartAppendOperation>> {
    match (previous, current) {
        (ChartSnapshot::Xy(previous), ChartSnapshot::Xy(current))
            if previous.plot_mode == current.plot_mode
                && previous.draw_style == current.draw_style
                && previous.x_name == current.x_name
                && previous.y_name == current.y_name =>
        {
            diff_xy_series(&previous.series, &current.series)
        }
        (ChartSnapshot::Heatmap(previous), ChartSnapshot::Heatmap(current)) => {
            diff_heatmap(previous, current)
        }
        _ => None,
    }
}

fn diff_xy_series(
    previous: &[FlatXYSeries],
    current: &[FlatXYSeries],
) -> Option<Vec<LiveChartAppendOperation>> {
    if previous.len() > current.len() {
        return None;
    }

    let mut ops = Vec::new();
    for (previous_series, current_series) in previous.iter().zip(current) {
        if previous_series.id != current_series.id || previous_series.label != current_series.label
        {
            return None;
        }
        if !current_series.values.starts_with(&previous_series.values) {
            return None;
        }
        if previous_series.point_count > current_series.point_count {
            return None;
        }
        let appended_values = current_series.values[previous_series.values.len()..].to_vec();
        let appended_points = current_series.point_count - previous_series.point_count;
        if appended_points > 0 {
            ops.push(LiveChartAppendOperation::AppendPoints {
                series_id: current_series.id.clone(),
                values: appended_values,
                point_count: appended_points,
            });
        }
    }

    for series in &current[previous.len()..] {
        ops.push(LiveChartAppendOperation::AppendSeries {
            series: FlatSeries::Xy(series.clone()),
        });
    }

    Some(ops)
}

fn diff_heatmap(
    previous: &HeatmapChartSnapshot,
    current: &HeatmapChartSnapshot,
) -> Option<Vec<LiveChartAppendOperation>> {
    if previous.series.len() > current.series.len() {
        return None;
    }

    let mut ops = Vec::new();

    for (previous_series, current_series) in previous.series.iter().zip(&current.series) {
        if previous_series.id != current_series.id || previous_series.label != current_series.label
        {
            return None;
        }
        if !current_series.values.starts_with(&previous_series.values) {
            return None;
        }
        if previous_series.point_count > current_series.point_count {
            return None;
        }
        let appended_values = current_series.values[previous_series.values.len()..].to_vec();
        let appended_points = current_series.point_count - previous_series.point_count;
        if appended_points > 0 {
            ops.push(LiveChartAppendOperation::AppendPoints {
                series_id: current_series.id.clone(),
                values: appended_values,
                point_count: appended_points,
            });
        }
    }

    for series in &current.series[previous.series.len()..] {
        ops.push(LiveChartAppendOperation::AppendSeries {
            series: FlatSeries::Xyz(series.clone()),
        });
    }

    Some(ops)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fricon::{
        AppManager, Client, DatasetDataType, DatasetRow, DatasetScalar, DatasetSchema, ScalarKind,
        WorkspaceRoot,
    };
    use indexmap::IndexMap;
    use tempfile::TempDir;

    use super::{
        dataset_live_chart_data as load_live_chart_data, diff_heatmap, diff_live_snapshots,
        diff_xy_series, recent_group_starts_in_scan_batch, resolve_dataset_chart_options,
        resolve_group_tail_start_in_scan_batch,
    };
    use crate::{
        desktop_runtime::session::WorkspaceSession,
        features::charts::{
            semantic_source,
            transform::test_utils::{numeric_batch, numeric_schema},
            types::{
                ChartCommonOptions, ChartSnapshot, DatasetChartDataOptions, FlatSeries,
                FlatXYSeries, FlatXYZSeries, HeatmapChartSnapshot, LiveChartAppendOperation,
                LiveChartDataOptions, LiveChartDataResponse, LiveHeatmapOptions,
                XYChartDataOptions, XYChartSnapshot, XYDrawStyle, XYPlotMode, XYPlotModeOptions,
                XYTraceRoleOptions,
            },
        },
    };

    #[test]
    fn resolve_group_tail_start_handles_scan_prefix_row() {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 2.0, 2.0, 3.0, 3.0]),
            ("freq", &[10.0, 20.0, 10.0, 20.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["sweep", "freq"]);
        let row_indices = (4..10).collect::<Vec<_>>();

        let starts = recent_group_starts_in_scan_batch(&batch, &schema, &[0], &row_indices, 5);
        assert_eq!(starts, vec![6, 8]);
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], &row_indices, 5, 2),
            Some(6)
        );
    }

    #[test]
    fn resolve_group_tail_start_returns_none_without_enough_groups() {
        let batch = numeric_batch(&[
            ("sweep", &[2.0, 2.0, 3.0, 3.0]),
            ("freq", &[10.0, 20.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["sweep", "freq"]);
        let row_indices = (5..9).collect::<Vec<_>>();

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], &row_indices, 6, 2),
            None
        );
    }

    #[test]
    fn resolve_group_tail_start_uses_original_rows_after_projection() {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 2.0, 2.0, 3.0]),
            ("freq", &[10.0, 10.0, 20.0, 10.0]),
        ]);
        let schema = numeric_schema(&["sweep", "freq"]);
        let row_indices = vec![4, 6, 7, 9];

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], &row_indices, 5, 2),
            Some(6)
        );
    }

    #[test]
    fn resolve_group_tail_start_for_scalar_live_heatmap_uses_outer_sweep_key() {
        let batch = numeric_batch(&[
            ("cycle", &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            ("y", &[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0]),
            ("x", &[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
        ]);
        let schema = numeric_schema(&["cycle", "y", "x"]);
        let row_indices = (0..10).collect::<Vec<_>>();

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], &row_indices, 0, 1),
            Some(4)
        );
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], &row_indices, 0, 1),
            Some(8)
        );
    }

    #[test]
    fn resolve_group_tail_start_for_trace_live_heatmap_uses_outer_sweep_key() {
        let batch = numeric_batch(&[
            ("outer", &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            ("row", &[0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        ]);
        let schema = numeric_schema(&["outer", "row"]);
        let row_indices = (0..6).collect::<Vec<_>>();

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], &row_indices, 0, 1),
            Some(3)
        );
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], &row_indices, 0, 1),
            Some(5)
        );
    }

    #[test]
    fn resolve_dataset_chart_options_uses_projection_semantic_names() {
        let schema = DatasetSchema::new(IndexMap::from([
            (
                "signal".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
            (
                "logicalIndex:gate".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
            (
                "column:logicalIndex:physical".to_string(),
                DatasetDataType::Scalar(ScalarKind::Numeric),
            ),
        ]));
        let metadata = semantic_source::prepared_chart_data_for_test(
            schema,
            HashMap::from([
                ("column:signal".to_string(), "signal".to_string()),
                (
                    "logicalIndex:gate".to_string(),
                    "logicalIndex:gate".to_string(),
                ),
                (
                    "column:logicalIndex:physical".to_string(),
                    "column:logicalIndex:physical".to_string(),
                ),
            ]),
            Some(vec![1]),
            Some(vec![1, 2]),
        );
        let options = DatasetChartDataOptions::Xy(XYChartDataOptions {
            draw_style: XYDrawStyle::Line,
            plot_mode: XYPlotModeOptions::QuantityVsSweep {
                quantity: "column:signal".to_string(),
                complex_views: None,
            },
            trace_roles: XYTraceRoleOptions {
                trace_group_index_columns: Some(vec!["column:logicalIndex:physical".to_string()]),
                sweep_index_column: Some("logicalIndex:gate".to_string()),
            },
            common: ChartCommonOptions::default(),
        });

        let DatasetChartDataOptions::Xy(resolved) =
            resolve_dataset_chart_options(&metadata, &options)
        else {
            panic!("expected XY options");
        };

        let XYPlotModeOptions::QuantityVsSweep { quantity, .. } = resolved.plot_mode else {
            panic!("expected quantity vs sweep options");
        };
        assert_eq!(quantity, "signal");
        assert_eq!(
            resolved.trace_roles.trace_group_index_columns,
            Some(vec!["column:logicalIndex:physical".to_string()])
        );
        assert_eq!(
            resolved.trace_roles.sweep_index_column,
            Some("logicalIndex:gate".to_string())
        );
    }

    #[test]
    fn diff_xy_series_emits_append_points_and_new_series() {
        let previous = vec![xy_series("signal", "signal", &[0.0, 1.0])];
        let current = vec![
            xy_series("signal", "signal", &[0.0, 1.0, 1.0, 2.0]),
            xy_series("signal:imag", "signal (imag)", &[0.0, 3.0, 1.0, 4.0]),
        ];

        let ops = diff_xy_series(&previous, &current).expect("expected append ops");

        assert_eq!(
            ops,
            vec![
                LiveChartAppendOperation::AppendPoints {
                    series_id: "signal".to_string(),
                    values: vec![1.0, 2.0],
                    point_count: 1,
                },
                LiveChartAppendOperation::AppendSeries {
                    series: FlatSeries::Xy(xy_series(
                        "signal:imag",
                        "signal (imag)",
                        &[0.0, 3.0, 1.0, 4.0],
                    )),
                },
            ]
        );
    }

    #[test]
    fn diff_heatmap_emits_point_appends() {
        let previous = HeatmapChartSnapshot {
            x_name: "x".to_string(),
            y_name: "y".to_string(),
            series: vec![xyz_series("heat", "heat", &[0.0, 0.0, 1.0])],
        };
        let current = HeatmapChartSnapshot {
            x_name: "x".to_string(),
            y_name: "y".to_string(),
            series: vec![xyz_series("heat", "heat", &[0.0, 0.0, 1.0, 1.0, 2.0, 5.0])],
        };

        let ops = diff_heatmap(&previous, &current).expect("expected append ops");

        assert_eq!(
            ops,
            vec![LiveChartAppendOperation::AppendPoints {
                series_id: "heat".to_string(),
                values: vec![1.0, 2.0, 5.0],
                point_count: 1,
            },]
        );
    }

    #[test]
    fn diff_live_snapshots_resets_when_xy_metadata_changes() {
        let previous = ChartSnapshot::Xy(XYChartSnapshot {
            plot_mode: XYPlotMode::QuantityVsSweep,
            draw_style: XYDrawStyle::Line,
            x_name: "t".to_string(),
            y_name: None,
            series: vec![xy_series("signal", "signal", &[0.0, 1.0])],
        });
        let current = ChartSnapshot::Xy(XYChartSnapshot {
            plot_mode: XYPlotMode::QuantityVsSweep,
            draw_style: XYDrawStyle::Points,
            x_name: "t".to_string(),
            y_name: None,
            series: vec![xy_series("signal", "signal", &[0.0, 1.0, 1.0, 2.0])],
        });

        assert_eq!(diff_live_snapshots(&previous, &current), None);
    }

    fn xy_series(id: &str, label: &str, values: &[f64]) -> FlatXYSeries {
        FlatXYSeries::new(id, label, values.to_vec(), values.len() / 2)
    }

    fn xyz_series(id: &str, label: &str, values: &[f64]) -> FlatXYZSeries {
        FlatXYZSeries::new(id, label, values.to_vec(), values.len() / 3)
    }

    async fn create_live_chart_test_session(
        rows: Vec<DatasetRow>,
    ) -> anyhow::Result<(TempDir, AppManager, WorkspaceSession, i32)> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;

        let app_manager =
            AppManager::new_with_path(temp_dir.path())?.start(&tokio::runtime::Handle::current())?;
        let client = Client::connect(temp_dir.path()).await?;

        let schema = rows[0].to_schema();
        let mut writer = client
            .create_dataset(
                "live-heatmap-test".to_string(),
                String::new(),
                vec![],
                schema,
                Vec::new(),
                None,
                false,
            )
            .await?;
        for row in rows {
            writer.write(row).await?;
        }
        let dataset = writer.finish().await?;
        let session = WorkspaceSession::new(app_manager.handle().clone());
        Ok((temp_dir, app_manager, session, dataset.id()))
    }

    fn scalar_heatmap_rows() -> Vec<DatasetRow> {
        vec![
            scalar_row(0.0, 0.0, 0.0, 0.0),
            scalar_row(0.0, 0.0, 1.0, 1.0),
            scalar_row(0.0, 1.0, 0.0, 10.0),
            scalar_row(0.0, 1.0, 1.0, 11.0),
            scalar_row(1.0, 0.0, 0.0, 100.0),
            scalar_row(1.0, 0.0, 1.0, 101.0),
            scalar_row(1.0, 1.0, 0.0, 110.0),
            scalar_row(1.0, 1.0, 1.0, 111.0),
            scalar_row(1.0, 2.0, 0.0, 120.0),
            scalar_row(1.0, 2.0, 1.0, 121.0),
        ]
    }

    fn scalar_row(cycle: f64, y: f64, x: f64, val: f64) -> DatasetRow {
        DatasetRow(IndexMap::from([
            ("cycle".to_string(), DatasetScalar::Numeric(cycle)),
            ("y".to_string(), DatasetScalar::Numeric(y)),
            ("x".to_string(), DatasetScalar::Numeric(x)),
            ("val".to_string(), DatasetScalar::Numeric(val)),
        ]))
    }

    fn trace_heatmap_rows() -> Vec<DatasetRow> {
        vec![
            trace_row(0.0, 0.0, &[1.0, 2.0]),
            trace_row(0.0, 1.0, &[3.0, 4.0]),
            trace_row(1.0, 0.0, &[5.0, 6.0]),
            trace_row(1.0, 1.0, &[7.0, 8.0]),
        ]
    }

    fn trace_row(outer: f64, row: f64, values: &[f64]) -> DatasetRow {
        DatasetRow(IndexMap::from([
            ("outer".to_string(), DatasetScalar::Numeric(outer)),
            ("row".to_string(), DatasetScalar::Numeric(row)),
            (
                "trace".to_string(),
                DatasetScalar::SimpleTrace(values.iter().copied().collect()),
            ),
        ]))
    }

    fn heatmap_snapshot_from_live_response(
        response: LiveChartDataResponse,
    ) -> HeatmapChartSnapshot {
        match response {
            LiveChartDataResponse::Reset { snapshot, .. } => match snapshot {
                ChartSnapshot::Heatmap(snapshot) => snapshot,
                other @ ChartSnapshot::Xy(_) => panic!("expected heatmap snapshot, got {other:?}"),
            },
            LiveChartDataResponse::Append { .. } => {
                panic!("expected reset response for initial live request")
            }
        }
    }

    #[tokio::test]
    async fn live_scalar_heatmap_keeps_latest_outer_sweep_end_to_end() -> anyhow::Result<()> {
        let (_temp_dir, app_manager, session, dataset_id) =
            create_live_chart_test_session(scalar_heatmap_rows()).await?;

        let response = load_live_chart_data(
            &session,
            dataset_id,
            &LiveChartDataOptions::Heatmap(LiveHeatmapOptions {
                quantity: "val".to_string(),
                complex_view_single: None,
                known_row_count: None,
            }),
        )
        .await?;
        let snapshot = heatmap_snapshot_from_live_response(response);

        assert_eq!(snapshot.x_name, "x");
        assert_eq!(snapshot.y_name, "y");
        assert_eq!(snapshot.series.len(), 1);
        assert_eq!(snapshot.series[0].point_count, 6);

        app_manager.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn live_trace_heatmap_keeps_latest_outer_sweep_end_to_end() -> anyhow::Result<()> {
        let (_temp_dir, app_manager, session, dataset_id) =
            create_live_chart_test_session(trace_heatmap_rows()).await?;

        let response = load_live_chart_data(
            &session,
            dataset_id,
            &LiveChartDataOptions::Heatmap(LiveHeatmapOptions {
                quantity: "trace".to_string(),
                complex_view_single: None,
                known_row_count: None,
            }),
        )
        .await?;
        let snapshot = heatmap_snapshot_from_live_response(response);

        assert_eq!(snapshot.x_name, "trace - X");
        assert_eq!(snapshot.y_name, "row");
        assert_eq!(snapshot.series.len(), 1);
        assert_eq!(snapshot.series[0].point_count, 4);

        app_manager.shutdown().await;
        Ok(())
    }
}

use anyhow::Context;
use arrow_array::RecordBatch;
use fricon::DatasetSchema;
use tracing::{debug, error, instrument};

use super::{
    filter_table::build_semantic_filters, semantic_source, types::DatasetChartDataOptions,
};
#[cfg(test)]
use crate::features::charts::transform::compute_group_starts;
use crate::{
    desktop_runtime::session::WorkspaceSession,
    features::charts::{
        transform::{
            build_heatmap_series, build_live_heatmap_series, build_live_xy_series, build_xy_series,
        },
        types::{
            ChartCommonOptions, ChartSnapshot, FlatSeries, FlatXYSeries, HeatmapChartDataOptions,
            HeatmapChartSnapshot, LiveChartAppendOperation, LiveChartDataOptions,
            LiveChartDataResponse, LiveHeatmapOptions, LiveXYOptions, XYChartDataOptions,
            XYPlotModeOptions, XYTraceRoleOptions,
        },
    },
};

#[cfg(test)]
fn recent_group_starts_in_scan_batch(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    scan_start: usize,
    range_start: usize,
) -> Vec<usize> {
    compute_group_starts(batch, schema, grouping_index_columns)
        .into_iter()
        .map(|offset| scan_start + offset)
        .filter(|&row| row >= range_start)
        .collect()
}

#[cfg(test)]
fn resolve_group_tail_start_in_scan_batch(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    scan_start: usize,
    range_start: usize,
    required_groups: usize,
) -> Option<usize> {
    let starts = recent_group_starts_in_scan_batch(
        batch,
        schema,
        grouping_index_columns,
        scan_start,
        range_start,
    );
    (starts.len() >= required_groups).then(|| starts[starts.len() - required_groups])
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
    let prepared = semantic_source::prepare_chart_data(session, id, common, &filters).await?;
    let resolved_options = resolve_dataset_chart_options(options);
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
    let resolved_options = resolve_live_chart_options(options);
    let current_common = ChartCommonOptions {
        start: Some(0),
        end: Some(total_rows),
        index_filters: None,
        exclude_columns: None,
    };
    let prepared = semantic_source::prepare_chart_data(session, id, &current_common, &[]).await?;
    debug!(
        dataset_id = id,
        start = 0,
        end = total_rows,
        rows = prepared.batch.num_rows(),
        cols = prepared.batch.num_columns(),
        "Building live chart data"
    );

    let snapshot = build_live_snapshot(
        &prepared.batch,
        &prepared.schema,
        prepared.index_columns.as_deref(),
        0,
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

    let previous_common = ChartCommonOptions {
        start: Some(0),
        end: Some(known_row_count),
        index_filters: None,
        exclude_columns: None,
    };
    let previous_prepared =
        semantic_source::prepare_chart_data(session, id, &previous_common, &[]).await?;
    let previous_snapshot = build_live_snapshot(
        &previous_prepared.batch,
        &previous_prepared.schema,
        previous_prepared.index_columns.as_deref(),
        0,
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
    row_start: usize,
    options: &LiveChartDataOptions,
) -> anyhow::Result<ChartSnapshot> {
    match options {
        LiveChartDataOptions::Xy(opts) => {
            build_live_xy_series(batch, schema, index_columns, row_start, opts)
        }
        LiveChartDataOptions::Heatmap(opts) => {
            build_live_heatmap_series(batch, schema, index_columns, opts)
        }
    }
}

fn resolve_dataset_chart_options(options: &DatasetChartDataOptions) -> DatasetChartDataOptions {
    match options {
        DatasetChartDataOptions::Xy(options) => {
            DatasetChartDataOptions::Xy(resolve_xy_chart_options(options))
        }
        DatasetChartDataOptions::Heatmap(options) => {
            DatasetChartDataOptions::Heatmap(HeatmapChartDataOptions {
                quantity: semantic_source::resolve_column_name(&options.quantity),
                x_column: options
                    .x_column
                    .as_deref()
                    .map(semantic_source::resolve_axis_column_name),
                y_column: semantic_source::resolve_axis_column_name(&options.y_column),
                complex_view_single: options.complex_view_single,
                common: options.common.clone(),
            })
        }
    }
}

fn resolve_live_chart_options(options: &LiveChartDataOptions) -> LiveChartDataOptions {
    match options {
        LiveChartDataOptions::Xy(options) => LiveChartDataOptions::Xy(LiveXYOptions {
            draw_style: options.draw_style,
            tail_count: options.tail_count,
            known_row_count: options.known_row_count,
            plot_mode: resolve_xy_plot_mode_options(&options.plot_mode),
            trace_roles: resolve_trace_roles(&options.trace_roles),
        }),
        LiveChartDataOptions::Heatmap(options) => {
            LiveChartDataOptions::Heatmap(LiveHeatmapOptions {
                quantity: semantic_source::resolve_column_name(&options.quantity),
                complex_view_single: options.complex_view_single,
                known_row_count: options.known_row_count,
            })
        }
    }
}

fn resolve_xy_chart_options(options: &XYChartDataOptions) -> XYChartDataOptions {
    XYChartDataOptions {
        draw_style: options.draw_style,
        plot_mode: resolve_xy_plot_mode_options(&options.plot_mode),
        trace_roles: resolve_trace_roles(&options.trace_roles),
        common: options.common.clone(),
    }
}

fn resolve_xy_plot_mode_options(options: &XYPlotModeOptions) -> XYPlotModeOptions {
    match options {
        XYPlotModeOptions::QuantityVsSweep {
            quantity,
            complex_views,
        } => XYPlotModeOptions::QuantityVsSweep {
            quantity: semantic_source::resolve_column_name(quantity),
            complex_views: complex_views.clone(),
        },
        XYPlotModeOptions::Xy { x_column, y_column } => XYPlotModeOptions::Xy {
            x_column: semantic_source::resolve_column_name(x_column),
            y_column: semantic_source::resolve_column_name(y_column),
        },
        XYPlotModeOptions::ComplexPlane { quantity } => XYPlotModeOptions::ComplexPlane {
            quantity: semantic_source::resolve_column_name(quantity),
        },
    }
}

fn resolve_trace_roles(options: &XYTraceRoleOptions) -> XYTraceRoleOptions {
    XYTraceRoleOptions {
        trace_group_index_columns: options.trace_group_index_columns.as_ref().map(|columns| {
            columns
                .iter()
                .map(|column| semantic_source::resolve_axis_column_name(column))
                .collect()
        }),
        sweep_index_column: options
            .sweep_index_column
            .as_deref()
            .map(semantic_source::resolve_axis_column_name),
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
    use fricon::{AppManager, Client, DatasetRow, DatasetScalar, WorkspaceRoot};
    use indexmap::IndexMap;
    use tempfile::TempDir;

    use super::{
        dataset_live_chart_data as load_live_chart_data, diff_heatmap, diff_live_snapshots,
        diff_xy_series, recent_group_starts_in_scan_batch, resolve_group_tail_start_in_scan_batch,
    };
    use crate::{
        desktop_runtime::session::WorkspaceSession,
        features::charts::{
            transform::test_utils::{numeric_batch, numeric_schema},
            types::{
                ChartSnapshot, FlatSeries, FlatXYSeries, FlatXYZSeries, HeatmapChartSnapshot,
                LiveChartAppendOperation, LiveChartDataOptions, LiveChartDataResponse,
                LiveHeatmapOptions, XYChartSnapshot, XYDrawStyle, XYPlotMode,
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

        let starts = recent_group_starts_in_scan_batch(&batch, &schema, &[0], 4, 5);
        assert_eq!(starts, vec![6, 8]);
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], 4, 5, 2),
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

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], 5, 6, 2),
            None
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

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], 0, 0, 1),
            Some(4)
        );
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], 0, 0, 1),
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

        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0], 0, 0, 1),
            Some(3)
        );
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], 0, 0, 1),
            Some(5)
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

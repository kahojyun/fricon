use std::ops::Bound;

use anyhow::Context;
use arrow_array::RecordBatch;
use arrow_select::concat::concat_batches;
use fricon::{DatasetDataType, DatasetReader, DatasetSchema, SelectOptions};
use tracing::{debug, error, instrument};

use super::{filter_table::build_filter_batch, types::DatasetChartDataOptions};
use crate::{
    desktop_runtime::session::WorkspaceSession,
    features::charts::{
        transform::{
            build_heatmap_series, build_line_series, build_live_heatmap_series,
            build_live_line_series, build_live_scatter_series, build_scatter_series,
            compute_sweep_groups,
            mapping::{build_chart_selected_columns, build_live_chart_selected_columns},
        },
        types::{ChartDataResponse, LiveChartDataOptions, ScatterModeOptions},
    },
};

fn select_live_range(
    dataset: &DatasetReader,
    start: usize,
    end: usize,
    selected_columns: Vec<usize>,
) -> anyhow::Result<RecordBatch> {
    let (output_schema, batches) = dataset
        .select_data(&SelectOptions {
            start: Bound::Included(start),
            end: Bound::Excluded(end),
            index_filters: None,
            selected_columns: Some(selected_columns),
        })
        .context("Failed to select live data")?;

    concat_batches(&output_schema, &batches).context("Failed to concat batches")
}

fn recent_group_starts_in_scan_batch(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    scan_start: usize,
    range_start: usize,
) -> Vec<usize> {
    compute_sweep_groups(batch, schema, Some(grouping_index_columns))
        .into_iter()
        .map(|offset| scan_start + offset)
        .filter(|&row| row >= range_start)
        .collect()
}

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

fn resolve_group_tail_start(
    dataset: &DatasetReader,
    schema: &DatasetSchema,
    grouping_index_columns: &[usize],
    total_rows: usize,
    required_groups: usize,
) -> anyhow::Result<usize> {
    if total_rows == 0 || required_groups == 0 {
        return Ok(0);
    }

    let mut window_rows = required_groups.max(1);
    loop {
        let range_start = total_rows.saturating_sub(window_rows);
        let scan_start = range_start.saturating_sub(1);
        let batch = select_live_range(
            dataset,
            scan_start,
            total_rows,
            grouping_index_columns.to_vec(),
        )?;

        if let Some(start) = resolve_group_tail_start_in_scan_batch(
            &batch,
            schema,
            grouping_index_columns,
            scan_start,
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

fn resolve_live_row_start(
    dataset: &DatasetReader,
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveChartDataOptions,
) -> anyhow::Result<usize> {
    let total_rows = dataset.num_rows();
    if total_rows == 0 {
        return Ok(0);
    }

    match options {
        LiveChartDataOptions::Line(opts) => {
            let data_type = *schema
                .columns()
                .get(&opts.series)
                .context("Column not found")?;
            let tail_count = opts.tail_count.max(1);
            if matches!(data_type, DatasetDataType::Trace(_, _)) {
                return Ok(total_rows.saturating_sub(tail_count));
            }

            match index_columns {
                Some(idx_cols) if idx_cols.len() >= 2 => {
                    resolve_group_tail_start(dataset, schema, idx_cols, total_rows, tail_count)
                }
                _ => Ok(total_rows.saturating_sub(tail_count)),
            }
        }
        LiveChartDataOptions::Heatmap(opts) => {
            let data_type = *schema
                .columns()
                .get(&opts.series)
                .context("Column not found")?;
            if matches!(data_type, DatasetDataType::Trace(_, _)) {
                match index_columns {
                    Some(idx_cols) if idx_cols.len() >= 2 => {
                        resolve_group_tail_start(dataset, schema, idx_cols, total_rows, 1)
                    }
                    _ => Ok(total_rows.saturating_sub(1)),
                }
            } else if let Some(idx_cols) = index_columns {
                if idx_cols.len() >= 3 {
                    resolve_group_tail_start(
                        dataset,
                        schema,
                        &idx_cols[..idx_cols.len() - 1],
                        total_rows,
                        1,
                    )
                } else {
                    Ok(0)
                }
            } else {
                Ok(0)
            }
        }
        LiveChartDataOptions::Scatter(opts) => {
            let tail_count = opts.tail_count.max(1);
            match &opts.scatter {
                ScatterModeOptions::Complex { series } => {
                    let data_type = *schema.columns().get(series).context("Column not found")?;
                    if matches!(data_type, DatasetDataType::Trace(_, _)) {
                        Ok(total_rows.saturating_sub(tail_count))
                    } else if let Some(idx_cols) = index_columns {
                        if idx_cols.len() >= 2 {
                            resolve_group_tail_start(
                                dataset, schema, idx_cols, total_rows, tail_count,
                            )
                        } else {
                            Ok(total_rows.saturating_sub(tail_count))
                        }
                    } else {
                        Ok(total_rows.saturating_sub(tail_count))
                    }
                }
                ScatterModeOptions::TraceXy { .. } => Ok(total_rows.saturating_sub(tail_count)),
                ScatterModeOptions::Xy { .. } => {
                    if let Some(idx_cols) = index_columns {
                        if idx_cols.len() >= 2 {
                            resolve_group_tail_start(
                                dataset, schema, idx_cols, total_rows, tail_count,
                            )
                        } else {
                            Ok(total_rows.saturating_sub(tail_count))
                        }
                    } else {
                        Ok(total_rows.saturating_sub(tail_count))
                    }
                }
            }
        }
    }
}

#[instrument(level = "debug", skip(session, options), fields(dataset_id = id))]
pub(crate) async fn dataset_chart_data(
    session: &WorkspaceSession,
    id: i32,
    options: &DatasetChartDataOptions,
) -> anyhow::Result<ChartDataResponse> {
    let dataset = session.dataset(id).await?;
    let schema = dataset.schema();
    let common = options.common();
    let start = common.start.map_or(Bound::Unbounded, Bound::Included);
    let end = common.end.map_or(Bound::Unbounded, Bound::Excluded);
    let index_filters = if let Some(indices) = common.index_filters.clone() {
        build_filter_batch(
            session,
            id,
            common.exclude_columns.clone(),
            &indices,
            dataset.arrow_schema().clone(),
        )
        .await
        .context("Failed to build index filters")?
    } else {
        None
    };

    let selected_columns = build_chart_selected_columns(schema, options)?;
    let chart_type = options.chart_type_name();
    debug!(
        dataset_id = id,
        chart_type,
        ?selected_columns,
        "Selecting chart source data"
    );
    let (output_schema, batches) = dataset
        .select_data(&SelectOptions {
            start,
            end,
            index_filters,
            selected_columns: Some(selected_columns),
        })
        .inspect_err(|err| {
            error!(
                dataset_id = id,
                chart_type,
                error = %err,
                "Failed to select chart source data"
            );
        })
        .context("Failed to select data.")?;

    let batch = concat_batches(&output_schema, &batches).inspect_err(|err| {
        error!(
            dataset_id = id,
            chart_type,
            error = %err,
            "Failed to concat chart batches"
        );
    })?;
    debug!(
        dataset_id = id,
        chart_type,
        rows = batch.num_rows(),
        cols = batch.num_columns(),
        "Building dataset chart data"
    );

    let result = match options {
        DatasetChartDataOptions::Line(options) => build_line_series(&batch, schema, options),
        DatasetChartDataOptions::Heatmap(options) => build_heatmap_series(&batch, schema, options),
        DatasetChartDataOptions::Scatter(options) => build_scatter_series(&batch, schema, options),
    };
    if let Err(err) = &result {
        error!(
            dataset_id = id,
            chart_type,
            rows = batch.num_rows(),
            cols = batch.num_columns(),
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
) -> anyhow::Result<ChartDataResponse> {
    let dataset = session.dataset(id).await?;
    let schema = dataset.schema();
    let index_columns = dataset.index_columns();
    let total_rows = dataset.num_rows();
    let start = resolve_live_row_start(&dataset, schema, index_columns.as_deref(), options)?;
    let selected_columns =
        build_live_chart_selected_columns(schema, index_columns.as_deref(), options)?;

    let batch = select_live_range(&dataset, start, total_rows, selected_columns)?;
    debug!(
        dataset_id = id,
        start,
        end = total_rows,
        rows = batch.num_rows(),
        cols = batch.num_columns(),
        "Building live chart data"
    );

    let result = match options {
        LiveChartDataOptions::Line(opts) => {
            build_live_line_series(&batch, schema, index_columns.as_deref(), opts)
        }
        LiveChartDataOptions::Heatmap(opts) => {
            build_live_heatmap_series(&batch, schema, index_columns.as_deref(), opts)
        }
        LiveChartDataOptions::Scatter(opts) => {
            build_live_scatter_series(&batch, schema, index_columns.as_deref(), opts)
        }
    };
    if let Err(err) = &result {
        error!(
            dataset_id = id,
            rows = batch.num_rows(),
            error = %err,
            "Failed to build live chart data"
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{recent_group_starts_in_scan_batch, resolve_group_tail_start_in_scan_batch};
    use crate::features::charts::transform::test_utils::{numeric_batch, numeric_schema};

    #[test]
    fn resolve_group_tail_start_handles_scan_prefix_row() {
        let batch = numeric_batch(&[
            ("sweep", &[1.0, 1.0, 2.0, 2.0, 3.0, 3.0]),
            ("freq", &[10.0, 20.0, 10.0, 20.0, 10.0, 20.0]),
        ]);
        let schema = numeric_schema(&["sweep", "freq"]);

        let starts = recent_group_starts_in_scan_batch(&batch, &schema, &[0, 1], 4, 5);
        assert_eq!(starts, vec![6, 8]);
        assert_eq!(
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], 4, 5, 2),
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
            resolve_group_tail_start_in_scan_batch(&batch, &schema, &[0, 1], 5, 6, 2),
            None
        );
    }
}

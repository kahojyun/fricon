use std::ops::Bound;

use anyhow::Context;
use arrow_array::RecordBatch;
use arrow_select::concat::concat_batches;
use fricon::SelectOptions;
use tracing::{debug, error, instrument};

use super::{filter_table::build_filter_batch, types::DatasetChartDataOptions};
use crate::{
    desktop_runtime::session::WorkspaceSession,
    features::charts::transform::{
        ChartDataResponse, build_heatmap_series, build_line_series, build_scatter_series,
        mapping::build_chart_selected_columns,
    },
};

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
        .map_err(|err| {
            error!(
                dataset_id = id,
                chart_type,
                error = %err,
                "Failed to select chart source data"
            );
            err
        })
        .context("Failed to select data.")?;

    let batch = if batches.is_empty() {
        RecordBatch::new_empty(output_schema)
    } else {
        concat_batches(&output_schema, &batches)
            .map_err(|err| {
                error!(
                    dataset_id = id,
                    chart_type,
                    error = %err,
                    "Failed to concat chart batches"
                );
                err
            })
            .context("Failed to concat batches")?
    };
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

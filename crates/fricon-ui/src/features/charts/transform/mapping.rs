use anyhow::Context;
use fricon::{DatasetDataType, DatasetSchema};

use crate::features::charts::types::{
    DatasetChartDataOptions, HeatmapChartDataOptions, LineChartDataOptions,
    ScatterChartDataOptions, ScatterModeOptions,
};

fn column_index(schema: &DatasetSchema, name: &str) -> anyhow::Result<usize> {
    let (idx, _, _) = schema
        .columns()
        .get_full(name)
        .with_context(|| format!("Column '{name}' not found"))?;
    Ok(idx)
}

fn push_column(columns: &mut Vec<usize>, index: usize) {
    if !columns.contains(&index) {
        columns.push(index);
    }
}

fn resolve_series_column(
    schema: &DatasetSchema,
    series_name: &str,
) -> anyhow::Result<(usize, DatasetDataType)> {
    let series_index = column_index(schema, series_name)?;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    Ok((series_index, data_type))
}

fn build_line_selected_columns(
    schema: &DatasetSchema,
    options: &LineChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    let (series_index, data_type) = resolve_series_column(schema, &options.series)?;
    push_column(&mut selected, series_index);
    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        let x_name = options
            .x_column
            .as_ref()
            .context("Line chart requires x column")?;
        let x_index = column_index(schema, x_name)?;
        push_column(&mut selected, x_index);
    }
    Ok(selected)
}

fn build_heatmap_selected_columns(
    schema: &DatasetSchema,
    options: &HeatmapChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    let (series_index, data_type) = resolve_series_column(schema, &options.series)?;
    push_column(&mut selected, series_index);

    let y_index = column_index(schema, &options.y_column)?;
    push_column(&mut selected, y_index);

    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        let x_name = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        let x_index = column_index(schema, x_name)?;
        push_column(&mut selected, x_index);
    }

    Ok(selected)
}

fn build_scatter_selected_columns(
    schema: &DatasetSchema,
    options: &ScatterChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    match &options.scatter {
        ScatterModeOptions::Complex { series } => {
            push_column(&mut selected, column_index(schema, series)?);
        }
        ScatterModeOptions::TraceXy {
            trace_x_column,
            trace_y_column,
        } => {
            push_column(&mut selected, column_index(schema, trace_x_column)?);
            push_column(&mut selected, column_index(schema, trace_y_column)?);
        }
        ScatterModeOptions::Xy {
            x_column,
            y_column,
            bin_column,
        } => {
            push_column(&mut selected, column_index(schema, x_column)?);
            push_column(&mut selected, column_index(schema, y_column)?);
            if let Some(bin_name) = bin_column.as_ref() {
                push_column(&mut selected, column_index(schema, bin_name)?);
            }
        }
    }
    Ok(selected)
}

pub(crate) fn build_chart_selected_columns(
    schema: &DatasetSchema,
    options: &DatasetChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    match options {
        DatasetChartDataOptions::Line(options) => build_line_selected_columns(schema, options),
        DatasetChartDataOptions::Heatmap(options) => {
            build_heatmap_selected_columns(schema, options)
        }
        DatasetChartDataOptions::Scatter(options) => {
            build_scatter_selected_columns(schema, options)
        }
    }
}

#[cfg(test)]
mod tests {
    use fricon::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind};
    use indexmap::IndexMap;

    use super::build_chart_selected_columns;
    use crate::features::charts::types::{
        ChartCommonOptions, DatasetChartDataOptions, HeatmapChartDataOptions, LineChartDataOptions,
        ScatterChartDataOptions, ScatterModeOptions,
    };

    fn numeric_schema() -> DatasetSchema {
        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "z".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        DatasetSchema::new(columns)
    }

    fn mixed_scatter_schema() -> DatasetSchema {
        let mut columns = IndexMap::new();
        columns.insert(
            "complex_scalar".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        columns.insert(
            "complex_trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Complex),
        );
        columns.insert(
            "trace_x".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        columns.insert(
            "trace_y".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        DatasetSchema::new(columns)
    }

    #[test]
    fn build_chart_selected_columns_line() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Line(LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: None,
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![1, 0]);
    }

    #[test]
    fn build_chart_selected_columns_heatmap() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Heatmap(HeatmapChartDataOptions {
            series: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![2, 1, 0]);
    }

    #[test]
    fn build_chart_selected_columns_scatter_xy() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Scatter(ScatterChartDataOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: Some("z".to_string()),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![0, 1, 2]);
    }

    #[test]
    fn build_chart_selected_columns_scatter_complex_scalar() {
        let schema = mixed_scatter_schema();
        let options = DatasetChartDataOptions::Scatter(ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "complex_scalar".to_string(),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![0]);
    }

    #[test]
    fn build_chart_selected_columns_scatter_complex_trace() {
        let schema = mixed_scatter_schema();
        let options = DatasetChartDataOptions::Scatter(ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "complex_trace".to_string(),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![1]);
    }

    #[test]
    fn build_chart_selected_columns_scatter_trace_xy() {
        let schema = mixed_scatter_schema();
        let options = DatasetChartDataOptions::Scatter(ScatterChartDataOptions {
            scatter: ScatterModeOptions::TraceXy {
                trace_x_column: "trace_x".to_string(),
                trace_y_column: "trace_y".to_string(),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, &options).unwrap();
        assert_eq!(selected, vec![2, 3]);
    }
}

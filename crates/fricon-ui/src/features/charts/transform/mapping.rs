use anyhow::Context;
use fricon::{DatasetDataType, DatasetSchema};

use super::resolve_xy_trace_roles;
use crate::features::charts::types::{
    ChartCommonOptions, DatasetChartDataOptions, HeatmapChartDataOptions, LiveChartDataOptions,
    LiveHeatmapOptions, LiveXYOptions, XYChartDataOptions, XYPlotModeOptions,
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

fn push_columns(columns: &mut Vec<usize>, indices: &[usize]) {
    for &index in indices {
        push_column(columns, index);
    }
}

fn build_heatmap_selected_columns(
    schema: &DatasetSchema,
    options: &HeatmapChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    let quantity_index = column_index(schema, &options.quantity)?;
    let data_type = *schema
        .columns()
        .get(&options.quantity)
        .context("Column not found")?;
    push_column(&mut selected, quantity_index);

    let y_index = column_index(schema, &options.y_column)?;
    push_column(&mut selected, y_index);

    if !matches!(data_type, DatasetDataType::Trace(_, _)) {
        let x_name = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        push_column(&mut selected, column_index(schema, x_name)?);
    }

    Ok(selected)
}

fn build_xy_selected_columns(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &XYChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    match &options.plot_mode {
        XYPlotModeOptions::QuantityVsSweep { quantity, .. } => {
            let quantity_index = column_index(schema, quantity)?;
            let data_type = *schema.columns().get(quantity).context("Column not found")?;
            push_column(&mut selected, quantity_index);
            if !matches!(data_type, DatasetDataType::Trace(_, _)) {
                let roles = resolve_xy_trace_roles(
                    schema,
                    index_columns,
                    &options.trace_roles,
                    options.draw_style,
                )?;
                push_columns(&mut selected, &roles.trace_group);
                if let Some(sweep) = roles.sweep {
                    push_column(&mut selected, sweep);
                }
            }
        }
        XYPlotModeOptions::Xy { x_column, y_column } => {
            push_column(&mut selected, column_index(schema, x_column)?);
            push_column(&mut selected, column_index(schema, y_column)?);
            let x_type = *schema
                .columns()
                .get(x_column)
                .context("X column not found")?;
            let y_type = *schema
                .columns()
                .get(y_column)
                .context("Y column not found")?;
            if !matches!(x_type, DatasetDataType::Trace(_, _))
                && !matches!(y_type, DatasetDataType::Trace(_, _))
            {
                let roles = resolve_xy_trace_roles(
                    schema,
                    index_columns,
                    &options.trace_roles,
                    options.draw_style,
                )?;
                push_columns(&mut selected, &roles.trace_group);
                if let Some(sweep) = roles.sweep {
                    push_column(&mut selected, sweep);
                }
            }
        }
        XYPlotModeOptions::ComplexPlane { quantity } => {
            push_column(&mut selected, column_index(schema, quantity)?);
            let data_type = *schema.columns().get(quantity).context("Column not found")?;
            if !matches!(data_type, DatasetDataType::Trace(_, _)) {
                let roles = resolve_xy_trace_roles(
                    schema,
                    index_columns,
                    &options.trace_roles,
                    options.draw_style,
                )?;
                push_columns(&mut selected, &roles.trace_group);
                if let Some(sweep) = roles.sweep {
                    push_column(&mut selected, sweep);
                }
            }
        }
    }
    Ok(selected)
}

pub(crate) fn build_chart_selected_columns(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &DatasetChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    match options {
        DatasetChartDataOptions::Xy(options) => {
            build_xy_selected_columns(schema, index_columns, options)
        }
        DatasetChartDataOptions::Heatmap(options) => {
            build_heatmap_selected_columns(schema, options)
        }
    }
}

fn build_live_heatmap_selected_columns(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveHeatmapOptions,
) -> anyhow::Result<Vec<usize>> {
    let mut selected = Vec::new();
    push_column(&mut selected, column_index(schema, &options.quantity)?);
    if let Some(idx_cols) = index_columns {
        push_columns(&mut selected, idx_cols);
    }
    Ok(selected)
}

fn build_live_xy_selected_columns(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveXYOptions,
) -> anyhow::Result<Vec<usize>> {
    build_xy_selected_columns(
        schema,
        index_columns,
        &XYChartDataOptions {
            draw_style: options.draw_style,
            plot_mode: options.plot_mode.clone(),
            trace_roles: options.trace_roles.clone(),
            common: ChartCommonOptions::default(),
        },
    )
}

pub(crate) fn build_live_chart_selected_columns(
    schema: &DatasetSchema,
    index_columns: Option<&[usize]>,
    options: &LiveChartDataOptions,
) -> anyhow::Result<Vec<usize>> {
    match options {
        LiveChartDataOptions::Xy(options) => {
            build_live_xy_selected_columns(schema, index_columns, options)
        }
        LiveChartDataOptions::Heatmap(options) => {
            build_live_heatmap_selected_columns(schema, index_columns, options)
        }
    }
}

#[cfg(test)]
mod tests {
    use fricon::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind};
    use indexmap::IndexMap;

    use super::{
        super::test_utils::numeric_schema as make_numeric_schema, build_chart_selected_columns,
        build_live_chart_selected_columns,
    };
    use crate::features::charts::types::{
        ChartCommonOptions, DatasetChartDataOptions, HeatmapChartDataOptions, LiveChartDataOptions,
        LiveHeatmapOptions, LiveXYOptions, XYChartDataOptions, XYDrawStyle, XYPlotModeOptions,
        XYTraceRoleOptions,
    };

    fn numeric_schema() -> DatasetSchema {
        make_numeric_schema(&["outer", "inner", "x", "y"])
    }

    fn mixed_schema() -> DatasetSchema {
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
    fn build_chart_selected_columns_for_xy_with_roles() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Xy(XYChartDataOptions {
            draw_style: XYDrawStyle::Line,
            plot_mode: XYPlotModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
            },
            trace_roles: XYTraceRoleOptions {
                trace_group_index_columns: Some(vec!["outer".to_string()]),
                sweep_index_column: Some("inner".to_string()),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, Some(&[0, 1]), &options).unwrap();
        assert_eq!(selected, vec![2, 3, 0, 1]);
    }

    #[test]
    fn build_chart_selected_columns_for_complex_plane_trace_skips_trace_roles() {
        let schema = mixed_schema();
        let options = DatasetChartDataOptions::Xy(XYChartDataOptions {
            draw_style: XYDrawStyle::Points,
            plot_mode: XYPlotModeOptions::ComplexPlane {
                quantity: "complex_trace".to_string(),
            },
            trace_roles: XYTraceRoleOptions {
                trace_group_index_columns: Some(vec!["outer".to_string()]),
                sweep_index_column: Some("inner".to_string()),
            },
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, Some(&[0, 1]), &options).unwrap();
        assert_eq!(selected, vec![1]);
    }

    #[test]
    fn build_live_chart_selected_columns_for_heatmap_includes_indices() {
        let schema = numeric_schema();
        let options = LiveChartDataOptions::Heatmap(LiveHeatmapOptions {
            quantity: "y".to_string(),
            complex_view_single: None,
            known_row_count: None,
        });

        let selected = build_live_chart_selected_columns(&schema, Some(&[0, 1]), &options).unwrap();
        assert_eq!(selected, vec![3, 0, 1]);
    }

    #[test]
    fn build_live_chart_selected_columns_for_xy_includes_default_order_index() {
        let schema = numeric_schema();
        let options = LiveChartDataOptions::Xy(LiveXYOptions {
            draw_style: XYDrawStyle::Line,
            tail_count: 5,
            known_row_count: None,
            plot_mode: XYPlotModeOptions::QuantityVsSweep {
                quantity: "y".to_string(),
                complex_views: None,
            },
            trace_roles: XYTraceRoleOptions::default(),
        });

        let selected = build_live_chart_selected_columns(&schema, Some(&[0, 1]), &options).unwrap();
        assert_eq!(selected, vec![3, 1]);
    }

    #[test]
    fn build_chart_selected_columns_heatmap() {
        let schema = numeric_schema();
        let options = DatasetChartDataOptions::Heatmap(HeatmapChartDataOptions {
            quantity: "y".to_string(),
            x_column: Some("outer".to_string()),
            y_column: "inner".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        });

        let selected = build_chart_selected_columns(&schema, Some(&[0, 1]), &options).unwrap();
        assert_eq!(selected, vec![3, 1, 0]);
    }
}

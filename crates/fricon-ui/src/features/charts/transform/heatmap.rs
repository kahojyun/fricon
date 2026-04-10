use std::collections::HashMap;

use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};

use crate::features::charts::types::{
    ChartSnapshot, ComplexViewOption, FlatXYZSeries, HeatmapChartDataOptions, HeatmapChartSnapshot,
    complex_view_label, transform_complex_values,
};

pub(crate) fn build_heatmap_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &HeatmapChartDataOptions,
) -> Result<ChartSnapshot> {
    let quantity_name = &options.quantity;
    let y_column = &options.y_column;
    let data_type = *schema
        .columns()
        .get(quantity_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let x_name = if is_trace {
        format!("{quantity_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_array: DatasetArray = batch
        .column_by_name(quantity_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let view_option = options
        .complex_view_single
        .unwrap_or(ComplexViewOption::Mag);

    let mut series = if is_trace {
        process_trace_heatmap(
            batch,
            quantity_name,
            y_column,
            &series_array,
            is_complex,
            view_option,
        )?
    } else {
        let x_column = options
            .x_column
            .as_ref()
            .context("Heatmap chart requires x column")?;
        process_scalar_heatmap(
            batch,
            quantity_name,
            x_column,
            y_column,
            &series_array,
            is_complex,
            view_option,
        )?
    };

    let (x_categories, y_categories) = normalize_heatmap_series(&mut series);

    Ok(ChartSnapshot::Heatmap(HeatmapChartSnapshot {
        x_name,
        y_name: y_column.clone(),
        x_categories,
        y_categories,
        series,
    }))
}

pub(crate) fn normalize_heatmap_series(series: &mut [FlatXYZSeries]) -> (Vec<f64>, Vec<f64>) {
    fn f64_key(value: f64) -> u64 {
        if value == 0.0 { 0_u64 } else { value.to_bits() }
    }

    let mut x_categories: Vec<f64> = Vec::new();
    let mut y_categories: Vec<f64> = Vec::new();
    let mut x_index_by_value: HashMap<u64, usize> = HashMap::new();
    let mut y_index_by_value: HashMap<u64, usize> = HashMap::new();

    for item in series.iter_mut() {
        for point in item.values.chunks_exact_mut(3) {
            let x_value = point[0];
            let y_value = point[1];

            let x_index = if let Some(index) = x_index_by_value.get(&f64_key(x_value)) {
                *index
            } else {
                let index = x_categories.len();
                x_categories.push(x_value);
                x_index_by_value.insert(f64_key(x_value), index);
                index
            };

            let y_index = if let Some(index) = y_index_by_value.get(&f64_key(y_value)) {
                *index
            } else {
                let index = y_categories.len();
                y_categories.push(y_value);
                y_index_by_value.insert(f64_key(y_value), index);
                index
            };

            #[expect(
                clippy::cast_precision_loss,
                reason = "Heatmap category indices are bounded by dataset size and safe for \
                          plotting"
            )]
            {
                point[0] = x_index as f64;
                point[1] = y_index as f64;
            }
        }
    }

    (x_categories, y_categories)
}

fn process_trace_heatmap(
    batch: &RecordBatch,
    series_name: &str,
    y_column: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    view_option: ComplexViewOption,
) -> Result<Vec<FlatXYZSeries>> {
    let y_array = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y column not found")?;
    let ds_y: DatasetArray = y_array.try_into()?;
    let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();
    let mut values = Vec::new();
    let mut point_count = 0;
    for row in 0..batch.num_rows() {
        let Some((x_values, trace_values)) = series_array.expand_trace(row)? else {
            continue;
        };
        let y_value = *y_values.get(row).unwrap_or(&0.0);
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
                values.push(x_values[i]);
                values.push(y_value);
                values.push(z_values[i]);
            }
            point_count += len;
        } else {
            let z_values = ds_trace
                .as_numeric()
                .context("Expected numeric array")?
                .values();
            let len = x_values.len().min(z_values.len());
            for i in 0..len {
                values.push(x_values[i]);
                values.push(y_value);
                values.push(z_values[i]);
            }
            point_count += len;
        }
    }
    let name = if is_complex {
        format!("{series_name} ({})", complex_view_label(view_option))
    } else {
        series_name.to_string()
    };
    Ok(vec![FlatXYZSeries::new(
        name.clone(),
        name,
        values,
        point_count,
    )])
}

fn process_scalar_heatmap(
    batch: &RecordBatch,
    series_name: &str,
    x_column: &str,
    y_column: &str,
    series_array: &DatasetArray,
    is_complex: bool,
    view_option: ComplexViewOption,
) -> Result<Vec<FlatXYZSeries>> {
    let x_array = batch
        .column_by_name(x_column)
        .cloned()
        .context("X not found")?;
    let y_array = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y not found")?;
    let ds_x: DatasetArray = x_array.try_into()?;
    let ds_y: DatasetArray = y_array.try_into()?;
    let x_values = ds_x.as_numeric().context("X must be numeric")?.values();
    let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();

    let (values, point_count) = if is_complex {
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let z_values = transform_complex_values(
            complex_array.real().values(),
            complex_array.imag().values(),
            view_option,
        );
        let len = x_values.len().min(y_values.len()).min(z_values.len());
        let mut values = Vec::with_capacity(len * 3);
        for i in 0..len {
            values.push(x_values[i]);
            values.push(y_values[i]);
            values.push(z_values[i]);
        }
        (values, len)
    } else {
        let z_values = series_array
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = x_values.len().min(y_values.len()).min(z_values.len());
        let mut values = Vec::with_capacity(len * 3);
        for i in 0..len {
            values.push(x_values[i]);
            values.push(y_values[i]);
            values.push(z_values[i]);
        }
        (values, len)
    };
    let name = if is_complex {
        format!("{series_name} ({})", complex_view_label(view_option))
    } else {
        series_name.to_string()
    };
    Ok(vec![FlatXYZSeries::new(
        name.clone(),
        name,
        values,
        point_count,
    )])
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{ArrayRef, Float64Array};
    use arrow_schema::{DataType, Field};
    use fricon::{
        DatasetArray, DatasetDataType, DatasetScalar, DatasetSchema, ScalarArray, ScalarKind,
        TraceKind,
    };
    use indexmap::IndexMap;

    use super::*;
    use crate::features::charts::{
        transform::test_utils::{numeric_batch, numeric_schema},
        types::ChartCommonOptions,
    };

    fn heatmap_snapshot(snapshot: ChartSnapshot) -> HeatmapChartSnapshot {
        match snapshot {
            ChartSnapshot::Heatmap(snapshot) => snapshot,
            other @ ChartSnapshot::Xy(_) => {
                panic!("expected heatmap snapshot, got {other:?}")
            }
        }
    }

    fn xyz_points(series: &FlatXYZSeries) -> Vec<Vec<f64>> {
        series
            .values
            .chunks_exact(3)
            .map(|point| vec![point[0], point[1], point[2]])
            .collect()
    }

    #[test]
    fn test_build_heatmap_series_numeric() {
        let batch = numeric_batch(&[
            ("x", &[1.0, 2.0]),
            ("y", &[10.0, 10.0]),
            ("z", &[100.0, 200.0]),
        ]);
        let schema = numeric_schema(&["x", "y", "z"]);

        let options = HeatmapChartDataOptions {
            quantity: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = heatmap_snapshot(build_heatmap_series(&batch, &schema, &options).unwrap());
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.x_categories, vec![1.0, 2.0]);
        assert_eq!(res.y_categories, vec![10.0]);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![vec![0.0, 0.0, 100.0], vec![1.0, 0.0, 200.0]]
        );
    }

    #[test]
    fn test_build_heatmap_series_maps_1_based_indexes() {
        let batch = numeric_batch(&[
            ("x", &[1.0, 2.0, 1.0]),
            ("y", &[1.0, 1.0, 2.0]),
            ("z", &[10.0, 20.0, 30.0]),
        ]);
        let schema = numeric_schema(&["x", "y", "z"]);

        let options = HeatmapChartDataOptions {
            quantity: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = heatmap_snapshot(build_heatmap_series(&batch, &schema, &options).unwrap());
        assert_eq!(res.x_categories, vec![1.0, 2.0]);
        assert_eq!(res.y_categories, vec![1.0, 2.0]);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![
                vec![0.0, 0.0, 10.0],
                vec![1.0, 0.0, 20.0],
                vec![0.0, 1.0, 30.0]
            ]
        );
    }

    #[test]
    fn test_build_heatmap_series_maps_non_contiguous_indexes() {
        let batch = numeric_batch(&[
            ("x", &[10.0, 20.0, 40.0]),
            ("y", &[5.0, 5.0, 9.0]),
            ("z", &[1.0, 2.0, 3.0]),
        ]);
        let schema = numeric_schema(&["x", "y", "z"]);

        let options = HeatmapChartDataOptions {
            quantity: "z".to_string(),
            x_column: Some("x".to_string()),
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = heatmap_snapshot(build_heatmap_series(&batch, &schema, &options).unwrap());
        assert_eq!(res.x_categories, vec![10.0, 20.0, 40.0]);
        assert_eq!(res.y_categories, vec![5.0, 9.0]);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![
                vec![0.0, 0.0, 1.0],
                vec![1.0, 0.0, 2.0],
                vec![2.0, 1.0, 3.0]
            ]
        );
    }

    #[test]
    fn test_build_heatmap_series_trace_uses_same_category_semantics() {
        let y_vals = vec![100.0];
        let trace = DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![1.0, 2.0, 3.0]));
        let trace_array: ArrayRef = DatasetArray::from(trace).into();
        let y_array = Arc::new(Float64Array::from(y_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("y", DataType::Float64, false),
            Field::new("trace", trace_array.data_type().clone(), false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![y_array, trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = HeatmapChartDataOptions {
            quantity: "trace".to_string(),
            x_column: None,
            y_column: "y".to_string(),
            complex_view_single: None,
            common: ChartCommonOptions::default(),
        };

        let res = heatmap_snapshot(build_heatmap_series(&batch, &schema, &options).unwrap());
        assert_eq!(res.x_categories, vec![0.0, 1.0, 2.0]);
        assert_eq!(res.y_categories, vec![100.0]);
        assert_eq!(
            xyz_points(&res.series[0]),
            vec![
                vec![0.0, 0.0, 1.0],
                vec![1.0, 0.0, 2.0],
                vec![2.0, 0.0, 3.0]
            ]
        );
    }
}

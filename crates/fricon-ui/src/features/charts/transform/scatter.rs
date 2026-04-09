use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};

use crate::features::charts::types::{
    ChartSnapshot, FlatXYSeries, ScatterChartDataOptions, ScatterChartSnapshot, ScatterModeOptions,
};

pub(crate) fn build_scatter_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ScatterChartDataOptions,
) -> Result<ChartSnapshot> {
    let (x_name, y_name, series) = match &options.scatter {
        ScatterModeOptions::Complex { series } => process_complex_scatter(batch, schema, series)?,
        ScatterModeOptions::TraceXy {
            trace_x_column,
            trace_y_column,
        } => process_trace_xy_scatter(batch, trace_x_column, trace_y_column)?,
        ScatterModeOptions::Xy {
            x_column, y_column, ..
        } => process_xy_scatter(batch, x_column, y_column)?,
    };

    Ok(ChartSnapshot::Scatter(ScatterChartSnapshot {
        x_name,
        y_name,
        series,
    }))
}

fn process_complex_scatter(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
) -> Result<(String, String, Vec<FlatXYSeries>)> {
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let series_array: DatasetArray = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?
        .try_into()?;
    let mut values = Vec::new();
    let mut point_count = 0;
    if is_trace {
        for row in 0..batch.num_rows() {
            let Some((_x_values, trace_values)) = series_array.expand_trace(row)? else {
                continue;
            };
            let ds_trace: DatasetArray = trace_values.try_into()?;
            let complex_array = ds_trace.as_complex().context("Expected complex array")?;
            let reals = complex_array.real().values();
            let imags = complex_array.imag().values();
            let len = reals.len().min(imags.len());
            for i in 0..len {
                values.push(reals[i]);
                values.push(imags[i]);
            }
            point_count += len;
        }
    } else {
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        let len = reals.len().min(imags.len());
        for i in 0..len {
            values.push(reals[i]);
            values.push(imags[i]);
        }
        point_count = len;
    }
    Ok((
        format!("{series_name} (real)"),
        format!("{series_name} (imag)"),
        vec![FlatXYSeries::new(
            series_name.to_string(),
            series_name.to_string(),
            values,
            point_count,
        )],
    ))
}

fn process_trace_xy_scatter(
    batch: &RecordBatch,
    trace_x: &str,
    trace_y: &str,
) -> Result<(String, String, Vec<FlatXYSeries>)> {
    let x_array: DatasetArray = batch
        .column_by_name(trace_x)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(trace_y)
        .cloned()
        .context("Y not found")?
        .try_into()?;

    let mut values = Vec::new();
    let mut point_count = 0;
    for row in 0..batch.num_rows() {
        let Some((_x_axis, x_values_array)) = x_array.expand_trace(row)? else {
            continue;
        };
        let Some((_y_axis, y_values_array)) = y_array.expand_trace(row)? else {
            continue;
        };
        let ds_x: DatasetArray = x_values_array.try_into()?;
        let ds_y: DatasetArray = y_values_array.try_into()?;
        let x_values = ds_x.as_numeric().context("X must be numeric")?.values();
        let y_values = ds_y.as_numeric().context("Y must be numeric")?.values();
        let len = x_values.len().min(y_values.len());
        for i in 0..len {
            values.push(x_values[i]);
            values.push(y_values[i]);
        }
        point_count += len;
    }
    let series_name = format!("{trace_x} vs {trace_y}");
    Ok((
        trace_x.to_string(),
        trace_y.to_string(),
        vec![FlatXYSeries::new(
            series_name.clone(),
            series_name,
            values,
            point_count,
        )],
    ))
}

fn process_xy_scatter(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
) -> Result<(String, String, Vec<FlatXYSeries>)> {
    let x_array: DatasetArray = batch
        .column_by_name(x_column)
        .cloned()
        .context("X not found")?
        .try_into()?;
    let y_array: DatasetArray = batch
        .column_by_name(y_column)
        .cloned()
        .context("Y not found")?
        .try_into()?;
    let x_values = x_array.as_numeric().context("X must be numeric")?.values();
    let y_values = y_array.as_numeric().context("Y must be numeric")?.values();
    let len = x_values.len().min(y_values.len());
    let mut values = Vec::with_capacity(len * 2);
    for i in 0..len {
        values.push(x_values[i]);
        values.push(y_values[i]);
    }
    let series_name = format!("{x_column} vs {y_column}");
    Ok((
        x_column.to_string(),
        y_column.to_string(),
        vec![FlatXYSeries::new(
            series_name.clone(),
            series_name,
            values,
            len,
        )],
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Array, ArrayRef, Float64Array, StructArray};
    use arrow_schema::{DataType, Field};
    use fricon::{
        DatasetArray, DatasetDataType, DatasetScalar, DatasetSchema, ScalarArray, ScalarKind,
        TraceKind,
    };
    use indexmap::IndexMap;
    use num::complex::Complex64;

    use super::*;
    use crate::features::charts::{
        transform::test_utils::{numeric_batch, numeric_schema},
        types::{ChartCommonOptions, DatasetChartDataOptions},
    };

    fn scatter_snapshot(snapshot: ChartSnapshot) -> ScatterChartSnapshot {
        match snapshot {
            ChartSnapshot::Scatter(snapshot) => snapshot,
            other => panic!("expected scatter snapshot, got {other:?}"),
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
    fn test_build_scatter_series_complex_scalar_and_trace() {
        let scalar_complex_column = Arc::new(
            StructArray::try_new(
                vec![
                    Arc::new(Field::new("real", DataType::Float64, false)),
                    Arc::new(Field::new("imag", DataType::Float64, false)),
                ]
                .into(),
                vec![
                    Arc::new(Float64Array::from(vec![1.0, 2.0])),
                    Arc::new(Float64Array::from(vec![-1.0, -2.0])),
                ],
                None,
            )
            .unwrap(),
        );
        let scalar_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "c",
            scalar_complex_column.data_type().clone(),
            false,
        )]));
        let scalar_batch =
            RecordBatch::try_new(scalar_schema, vec![scalar_complex_column]).unwrap();

        let mut scalar_columns = IndexMap::new();
        scalar_columns.insert(
            "c".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        let scalar_dataset_schema = DatasetSchema::new(scalar_columns);
        let scalar_options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "c".to_string(),
            },
            common: ChartCommonOptions::default(),
        };
        let scalar_res = scatter_snapshot(
            build_scatter_series(&scalar_batch, &scalar_dataset_schema, &scalar_options).unwrap(),
        );
        assert_eq!(
            xy_points(&scalar_res.series[0]),
            vec![vec![1.0, -1.0], vec![2.0, -2.0]]
        );

        let trace_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                Complex64::new(3.0, 4.0),
                Complex64::new(5.0, 6.0),
            ])))
            .into();
        let trace_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "t",
            trace_array.data_type().clone(),
            false,
        )]));
        let trace_batch = RecordBatch::try_new(trace_schema, vec![trace_array]).unwrap();

        let mut trace_columns = IndexMap::new();
        trace_columns.insert(
            "t".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Complex),
        );
        let trace_dataset_schema = DatasetSchema::new(trace_columns);
        let trace_options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Complex {
                series: "t".to_string(),
            },
            common: ChartCommonOptions::default(),
        };
        let trace_res = scatter_snapshot(
            build_scatter_series(&trace_batch, &trace_dataset_schema, &trace_options).unwrap(),
        );
        assert_eq!(
            xy_points(&trace_res.series[0]),
            vec![vec![3.0, 4.0], vec![5.0, 6.0]]
        );
    }

    #[test]
    fn test_build_scatter_series_trace_xy_truncates_to_shorter_trace() {
        let x_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                1.0, 2.0, 3.0,
            ])))
            .into();
        let y_array: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                10.0, 20.0,
            ])))
            .into();
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("tx", x_array.data_type().clone(), false),
            Field::new("ty", y_array.data_type().clone(), false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![x_array, y_array]).unwrap();

        let options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::TraceXy {
                trace_x_column: "tx".to_string(),
                trace_y_column: "ty".to_string(),
            },
            common: ChartCommonOptions::default(),
        };

        let mut columns = IndexMap::new();
        columns.insert(
            "tx".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        columns.insert(
            "ty".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let res = scatter_snapshot(build_scatter_series(&batch, &schema, &options).unwrap());
        assert_eq!(
            xy_points(&res.series[0]),
            vec![vec![1.0, 10.0], vec![2.0, 20.0]]
        );
    }

    #[test]
    fn test_build_scatter_series_xy() {
        let batch = numeric_batch(&[("x", &[1.0, 2.0]), ("y", &[10.0, 20.0])]);
        let schema = numeric_schema(&["x", "y"]);

        let options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
            },
            common: ChartCommonOptions::default(),
        };

        let res = scatter_snapshot(build_scatter_series(&batch, &schema, &options).unwrap());
        assert_eq!(res.series.len(), 1);
        assert_eq!(
            xy_points(&res.series[0]),
            vec![vec![1.0, 10.0], vec![2.0, 20.0]]
        );
    }

    #[test]
    fn test_deserialize_scatter_xy_missing_required_field_fails() {
        let input = serde_json::json!({
            "chartType": "scatter",
            "scatter": {
                "mode": "xy",
                "xColumn": "x"
            }
        });
        let parsed: std::result::Result<DatasetChartDataOptions, _> = serde_json::from_value(input);
        assert!(parsed.is_err());
    }
}

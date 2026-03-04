use anyhow::{Context, Result};
use arrow_array::RecordBatch;
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};

use super::{ChartDataResponse, ChartType, ScatterChartDataOptions, ScatterModeOptions, Series};

pub(crate) fn build_scatter_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &ScatterChartDataOptions,
) -> Result<ChartDataResponse> {
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

    Ok(ChartDataResponse {
        r#type: ChartType::Scatter,
        x_name,
        y_name: Some(y_name),
        x_categories: None,
        y_categories: None,
        series,
    })
}

fn process_complex_scatter(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    series_name: &str,
) -> Result<(String, String, Vec<Series>)> {
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
    let mut data = Vec::new();
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
                data.push(vec![reals[i], imags[i]]);
            }
        }
    } else {
        let complex_array = series_array
            .as_complex()
            .context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();
        let len = reals.len().min(imags.len());
        for i in 0..len {
            data.push(vec![reals[i], imags[i]]);
        }
    }
    Ok((
        format!("{series_name} (real)"),
        format!("{series_name} (imag)"),
        vec![Series {
            name: series_name.to_string(),
            data,
        }],
    ))
}

fn process_trace_xy_scatter(
    batch: &RecordBatch,
    trace_x: &str,
    trace_y: &str,
) -> Result<(String, String, Vec<Series>)> {
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

    let mut data = Vec::new();
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
            data.push(vec![x_values[i], y_values[i]]);
        }
    }
    let series_name = format!("{trace_x} vs {trace_y}");
    Ok((
        trace_x.to_string(),
        trace_y.to_string(),
        vec![Series {
            name: series_name,
            data,
        }],
    ))
}

fn process_xy_scatter(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
) -> Result<(String, String, Vec<Series>)> {
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
    let data = (0..len)
        .map(|i| vec![x_values[i], y_values[i]])
        .collect::<Vec<_>>();
    let series_name = format!("{x_column} vs {y_column}");
    Ok((
        x_column.to_string(),
        y_column.to_string(),
        vec![Series {
            name: series_name,
            data,
        }],
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Array, ArrayRef, Float64Array, RecordBatch, StructArray};
    use arrow_schema::{DataType, Field};
    use fricon::{
        DatasetArray, DatasetDataType, DatasetScalar, DatasetSchema, ScalarArray, ScalarKind,
        TraceKind,
    };
    use indexmap::IndexMap;
    use num::complex::Complex64;

    use super::*;
    use crate::models::chart::{ChartCommonOptions, DatasetChartDataOptions};

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
        let scalar_res =
            build_scatter_series(&scalar_batch, &scalar_dataset_schema, &scalar_options).unwrap();
        assert_eq!(
            scalar_res.series[0].data,
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
        let trace_res =
            build_scatter_series(&trace_batch, &trace_dataset_schema, &trace_options).unwrap();
        assert_eq!(
            trace_res.series[0].data,
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

        let res = build_scatter_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series[0].data, vec![vec![1.0, 10.0], vec![2.0, 20.0]]);
    }

    #[test]
    fn test_build_scatter_series_xy() {
        let x_vals = vec![1.0, 2.0];
        let y_vals = vec![10.0, 20.0];
        let array_x = Arc::new(Float64Array::from(x_vals));
        let array_y = Arc::new(Float64Array::from(y_vals));
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(arrow_schema, vec![array_x, array_y]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = ScatterChartDataOptions {
            scatter: ScatterModeOptions::Xy {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                bin_column: None,
            },
            common: ChartCommonOptions::default(),
        };

        let res = build_scatter_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].data, vec![vec![1.0, 10.0], vec![2.0, 20.0]]);
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

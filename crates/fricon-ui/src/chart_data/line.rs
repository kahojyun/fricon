use anyhow::{Context, Result};
use arrow_array::{ArrayRef, RecordBatch};
use fricon::{DatasetArray, DatasetDataType, DatasetSchema};
use tracing::{debug, warn};

use super::{
    ChartDataResponse, ChartType, ComplexViewOption, LineChartDataOptions, Series,
    complex_view_label, transform_complex_values,
};

fn empty_line_response(x_name: String) -> ChartDataResponse {
    ChartDataResponse {
        r#type: ChartType::Line,
        x_name,
        y_name: None,
        x_categories: None,
        y_categories: None,
        series: vec![],
    }
}

fn resolve_trace_line_values(
    series_array: &DatasetArray,
    series_name: &str,
) -> Result<Option<(usize, Vec<f64>, ArrayRef)>> {
    let row = 0;
    if let Some((x_values, y_values_array)) = series_array
        .expand_trace(row)
        .with_context(|| format!("Failed to expand trace row {row} for series '{series_name}'"))?
    {
        if x_values.is_empty() {
            debug!(row, series = %series_name, "Trace row is empty");
            return Ok(None);
        }
        return Ok(Some((row, x_values, y_values_array)));
    }
    Ok(None)
}

fn resolve_scalar_line_values(
    batch: &RecordBatch,
    options: &LineChartDataOptions,
    series_column: ArrayRef,
) -> Result<(Vec<f64>, ArrayRef)> {
    let x_column = options
        .x_column
        .as_ref()
        .context("Line chart requires x column")?;
    let x_array = batch
        .column_by_name(x_column)
        .cloned()
        .context("X column not found")?;
    let ds_x: DatasetArray = x_array.try_into()?;
    let x_values = ds_x
        .as_numeric()
        .context("X must be numeric")?
        .values()
        .to_vec();
    Ok((x_values, series_column))
}

fn convert_line_y_array(
    y_values_array: ArrayRef,
    series_name: &str,
    trace_row: Option<usize>,
) -> Result<DatasetArray> {
    let y_arrow_type = y_values_array.data_type().clone();
    y_values_array.try_into().with_context(|| {
        let trace_row_desc = if trace_row.is_none() {
            "scalar"
        } else {
            "trace"
        };
        format!(
            "Failed to convert {trace_row_desc} Y array for series '{series_name}' (source row: \
             {trace_row:?}, Arrow type: {y_arrow_type:?})"
        )
    })
}

pub(crate) fn build_line_series(
    batch: &RecordBatch,
    schema: &DatasetSchema,
    options: &LineChartDataOptions,
) -> Result<ChartDataResponse> {
    let series_name = &options.series;
    let data_type = *schema
        .columns()
        .get(series_name)
        .context("Column not found")?;
    let is_trace = matches!(data_type, DatasetDataType::Trace(_, _));
    let is_complex = data_type.is_complex();
    let x_name = if is_trace {
        format!("{series_name} - X")
    } else {
        options.x_column.clone().unwrap_or_else(|| "X".to_string())
    };

    let series_column = batch
        .column_by_name(series_name)
        .cloned()
        .context("Column not found")?;
    debug!(
        chart_type = "line",
        series = %series_name,
        ?data_type,
        arrow_type = ?series_column.data_type(),
        rows = batch.num_rows(),
        "Building line chart series"
    );

    let series_array: DatasetArray = series_column.clone().try_into().with_context(|| {
        format!(
            "Failed to convert series column '{series_name}' to DatasetArray (Arrow type: {:?}, \
             schema type: {:?})",
            series_column.data_type(),
            data_type
        )
    })?;

    let (trace_row, x_values, y_values_array) = if is_trace {
        let Some((row, x_values, y_values_array)) =
            resolve_trace_line_values(&series_array, series_name)?
        else {
            warn!(
                chart_type = "line",
                series = %series_name,
                rows = batch.num_rows(),
                "First trace row is empty or null for line chart"
            );
            return Ok(empty_line_response(x_name));
        };
        (Some(row), x_values, y_values_array)
    } else {
        let (x_values, y_values_array) =
            resolve_scalar_line_values(batch, options, series_column.clone())?;
        (None, x_values, y_values_array)
    };

    let ds_y = convert_line_y_array(y_values_array, series_name, trace_row)?;

    let series = if is_complex {
        let complex_array = ds_y.as_complex().context("Expected complex array")?;
        let reals = complex_array.real().values();
        let imags = complex_array.imag().values();

        let view_options = options
            .complex_views
            .clone()
            .unwrap_or_else(|| vec![ComplexViewOption::Real, ComplexViewOption::Imag]);
        view_options
            .into_iter()
            .map(|option| {
                let y_values = transform_complex_values(reals, imags, option);
                let len = x_values.len().min(y_values.len());
                let data = (0..len).map(|i| vec![x_values[i], y_values[i]]).collect();
                Series {
                    name: format!("{series_name} ({})", complex_view_label(option)),
                    data,
                }
            })
            .collect()
    } else {
        let y_values = ds_y
            .as_numeric()
            .context("Expected numeric array")?
            .values();
        let len = x_values.len().min(y_values.len());
        vec![Series {
            name: series_name.clone(),
            data: (0..len).map(|i| vec![x_values[i], y_values[i]]).collect(),
        }]
    };

    Ok(ChartDataResponse {
        r#type: ChartType::Line,
        x_name,
        y_name: None,
        x_categories: None,
        y_categories: None,
        series,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Array, ArrayRef, Float64Array, RecordBatch, StructArray, new_empty_array};
    use arrow_schema::{DataType, Field};
    use arrow_select::concat::concat;
    use fricon::{
        DatasetArray, DatasetDataType, DatasetScalar, DatasetSchema, ScalarArray, ScalarKind,
        TraceKind,
    };
    use indexmap::IndexMap;

    use super::*;
    use crate::chart_data::ChartCommonOptions;

    #[test]
    fn test_build_line_series_numeric() {
        let x_vals = vec![1.0, 2.0, 3.0];
        let y_vals = vec![10.0, 20.0, 30.0];
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

        let options = LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert_eq!(res.series[0].name, "y");
        assert_eq!(
            res.series[0].data,
            vec![vec![1.0, 10.0], vec![2.0, 20.0], vec![3.0, 30.0]]
        );
    }

    #[test]
    fn test_build_line_series_complex() {
        let real_vals = vec![1.0, 2.0];
        let imag_vals = vec![3.0, 4.0];
        let real_array = Arc::new(Float64Array::from(real_vals));
        let imag_array = Arc::new(Float64Array::from(imag_vals));

        let fields = vec![
            Arc::new(Field::new("real", DataType::Float64, false)),
            Arc::new(Field::new("imag", DataType::Float64, false)),
        ];
        let complex_struct =
            StructArray::try_new(fields.into(), vec![real_array, imag_array], None).unwrap();

        let x_vals = vec![0.1, 0.2];
        let x_array = Arc::new(Float64Array::from(x_vals));

        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", complex_struct.data_type().clone(), false),
        ]));
        let batch =
            RecordBatch::try_new(arrow_schema, vec![x_array, Arc::new(complex_struct)]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "x".to_string(),
            DatasetDataType::Scalar(ScalarKind::Numeric),
        );
        columns.insert(
            "y".to_string(),
            DatasetDataType::Scalar(ScalarKind::Complex),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "y".to_string(),
            x_column: Some("x".to_string()),
            complex_views: Some(vec![ComplexViewOption::Mag]),
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert_eq!(res.series.len(), 1);
        assert!(res.series[0].name.contains("mag"));
        assert!((res.series[0].data[0][1] - 3.1622).abs() < 1e-4);
    }

    #[test]
    fn test_build_line_series_trace_empty_batch_returns_empty_series() {
        let trace_data_type = TraceKind::Simple
            .to_data_type(Arc::new(Field::new_list_field(DataType::Float64, false)));
        let trace_array = new_empty_array(&trace_data_type);
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "trace",
            trace_data_type,
            false,
        )]));
        let batch = RecordBatch::try_new(arrow_schema, vec![trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert!(res.series.is_empty());
    }

    #[test]
    fn test_build_line_series_trace_empty_trace_returns_empty_series() {
        let trace_array: ArrayRef = DatasetArray::from(DatasetScalar::SimpleTrace(
            ScalarArray::from_iter(Vec::<f64>::new()),
        ))
        .into();
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "trace",
            trace_array.data_type().clone(),
            false,
        )]));
        let batch = RecordBatch::try_new(arrow_schema, vec![trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert!(res.series.is_empty());
    }

    #[test]
    fn test_build_line_series_trace_does_not_fallback_to_next_row() {
        let empty_trace: ArrayRef = DatasetArray::from(DatasetScalar::SimpleTrace(
            ScalarArray::from_iter(Vec::<f64>::new()),
        ))
        .into();
        let non_empty_trace: ArrayRef =
            DatasetArray::from(DatasetScalar::SimpleTrace(ScalarArray::from_iter(vec![
                1.0, 2.0, 3.0,
            ])))
            .into();
        let trace_array = concat(&[&*empty_trace, &*non_empty_trace]).unwrap();
        let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![Field::new(
            "trace",
            trace_array.data_type().clone(),
            false,
        )]));
        let batch = RecordBatch::try_new(arrow_schema, vec![trace_array]).unwrap();

        let mut columns = IndexMap::new();
        columns.insert(
            "trace".to_string(),
            DatasetDataType::Trace(TraceKind::Simple, ScalarKind::Numeric),
        );
        let schema = DatasetSchema::new(columns);

        let options = LineChartDataOptions {
            series: "trace".to_string(),
            x_column: None,
            complex_views: None,
            common: ChartCommonOptions::default(),
        };

        let res = build_line_series(&batch, &schema, &options).unwrap();
        assert!(res.series.is_empty());
    }
}

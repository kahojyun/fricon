use anyhow::{Result, ensure};
use arrow::{
    array::{Array, Float64Array, StructArray},
    datatypes::{DataType, Field},
    pyarrow::PyArrowType,
};
use pyo3::prelude::*;

use crate::conversion::{extract_float_array, extract_scalar_array, wrap_as_list_array};

/// 1-D list of values with optional x-axis values.
#[pyclass(module = "fricon._core")]
pub struct Trace {
    array: StructArray,
}

#[pymethods]
impl Trace {
    /// Create a new trace with variable x steps.
    ///
    /// Parameters:
    ///     xs: List of x-axis values.
    ///     ys: List of y-axis values.
    ///
    /// Returns:
    ///     A variable-step trace.
    #[staticmethod]
    pub fn variable_step(xs: &Bound<'_, PyAny>, ys: &Bound<'_, PyAny>) -> Result<Self> {
        let xs = extract_float_array(xs)?;
        let ys = extract_scalar_array(ys)?;
        ensure!(
            xs.len() == ys.len(),
            "Length of `xs` and `ys` should be equal."
        );
        let xs_list = wrap_as_list_array(std::sync::Arc::new(xs));
        let ys_list = wrap_as_list_array(ys);
        let fields = vec![
            Field::new("xs", xs_list.data_type().clone(), false),
            Field::new("ys", ys_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![std::sync::Arc::new(xs_list), std::sync::Arc::new(ys_list)],
            None,
        );
        Ok(Self { array })
    }

    /// Create a new trace with fixed x steps.
    ///
    /// Parameters:
    ///     x0: Starting x-axis value.
    ///     dx: Step size of x-axis values.
    ///     ys: List of y-axis values.
    ///
    /// Returns:
    ///     A fixed-step trace.
    #[staticmethod]
    pub fn fixed_step(x0: f64, dx: f64, ys: &Bound<'_, PyAny>) -> Result<Self> {
        let x0 = Float64Array::new_scalar(x0).into_inner();
        let dx = Float64Array::new_scalar(dx).into_inner();
        let ys = extract_scalar_array(ys)?;
        let ys_list = wrap_as_list_array(ys);
        let fields = vec![
            Field::new("x0", DataType::Float64, false),
            Field::new("dx", DataType::Float64, false),
            Field::new("ys", ys_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![
                std::sync::Arc::new(x0),
                std::sync::Arc::new(dx),
                std::sync::Arc::new(ys_list),
            ],
            None,
        );
        Ok(Self { array })
    }

    /// Arrow data type of the trace.
    #[getter]
    pub fn data_type(&self) -> PyArrowType<DataType> {
        PyArrowType(self.array.data_type().clone())
    }

    /// Convert to an arrow array.
    ///
    /// Returns:
    ///     Arrow array.
    pub fn to_arrow_array(&self) -> PyArrowType<arrow::array::ArrayData> {
        PyArrowType(self.array.to_data())
    }
}

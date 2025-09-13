use anyhow::{Result, bail, ensure};
use arrow::{
    array::{
        Array, ArrayRef, BooleanArray, Float64Array, Int64Array, ListArray, StringBuilder,
        StructArray, make_array,
    },
    buffer::OffsetBuffer,
    datatypes::{DataType, Field},
    pyarrow::PyArrowType,
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyFloat, PyInt, PySequence, PyString},
};

use crate::conversion::extract_float_array;
use fricon::FriconTypeExt;

// Minimal helpers retained locally after legacy removal.
fn create_item_field_from_array(array: &ArrayRef) -> Field {
    let dt = array.data_type();
    if dt.is_complex() {
        // preserve complex extension metadata
        fricon::ComplexType::field("item", false)
    } else {
        Field::new("item", dt.clone(), false)
    }
}

fn wrap_as_list_array_with_field(array: ArrayRef, item_field: Field) -> ListArray {
    let list_field = Field::new(
        "list",
        DataType::List(std::sync::Arc::new(item_field)),
        false,
    );
    ListArray::new(
        std::sync::Arc::new(list_field),
        OffsetBuffer::from_lengths([array.len()]),
        array,
        None,
    )
}

fn wrap_as_list_array(array: ArrayRef) -> ListArray {
    let item_field = create_item_field_from_array(&array);
    wrap_as_list_array_with_field(array, item_field)
}

fn extract_scalar_array(values: &Bound<'_, PyAny>) -> Result<ArrayRef> {
    if let Ok(PyArrowType(data)) = values.extract::<PyArrowType<arrow::array::ArrayData>>() {
        let arr = make_array(data);
        return match arr.data_type() {
            DataType::Boolean | DataType::Int64 | DataType::Float64 | DataType::Utf8 => Ok(arr),
            t @ DataType::Struct(_) if t.is_complex() => Ok(arr),
            _ => bail!("Unsupported arrow array type for ys."),
        };
    }
    if let Ok(seq) = values.downcast::<PySequence>() {
        ensure!(seq.len()? > 0, "Cannot build trace from empty sequence.");
        let first = seq.get_item(0)?;
        if first.is_instance_of::<PyBool>() {
            let mut b = BooleanArray::builder(seq.len()?);
            for v in seq.try_iter()? {
                b.append_value(v?.extract::<bool>()?);
            }
            return Ok(std::sync::Arc::new(b.finish()));
        } else if first.is_instance_of::<PyInt>() {
            let mut b = Int64Array::builder(seq.len()?);
            for v in seq.try_iter()? {
                b.append_value(v?.extract::<i64>()?);
            }
            return Ok(std::sync::Arc::new(b.finish()));
        } else if first.is_instance_of::<PyFloat>() {
            let mut b = Float64Array::builder(seq.len()?);
            for v in seq.try_iter()? {
                b.append_value(v?.extract::<f64>()?);
            }
            return Ok(std::sync::Arc::new(b.finish()));
        } else if first.is_instance_of::<PyString>() {
            let mut b = StringBuilder::new();
            for v in seq.try_iter()? {
                b.append_value(v?.extract::<String>()?);
            }
            return Ok(std::sync::Arc::new(b.finish()));
        }
        bail!("Unsupported sequence element type for ys.");
    }
    let py_type = values.get_type();
    bail!("Cannot convert {py_type} to scalar array.");
}

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
        let ys_item_field = create_item_field_from_array(&ys);
        let ys_list = wrap_as_list_array_with_field(ys.clone(), ys_item_field);
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
        let ys_item_field = create_item_field_from_array(&ys);
        let ys_list = wrap_as_list_array_with_field(ys.clone(), ys_item_field);
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

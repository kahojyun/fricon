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
use fricon::{FriconTypeExt, ScalarKind, TraceType, dataset_schema::DatasetDataType};

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
    dtype: DatasetDataType, // Always a Trace variant
    array: ArrayRef,
}

fn infer_scalar_kind(arr: &ArrayRef) -> Result<ScalarKind> {
    let dt = arr.data_type();
    if dt.is_complex() {
        Ok(ScalarKind::Complex128)
    } else if matches!(dt, DataType::Float64) {
        Ok(ScalarKind::Float64)
    } else {
        bail!("Unsupported scalar kind for trace: {dt}");
    }
}

#[pymethods]
impl Trace {
    /// Create a simple list trace (y-only values, implicit integer x starting at 0).
    ///
    /// Parameters:
    ///     ys: Sequence or Arrow array of scalar values.
    ///
    /// Returns:
    ///     A simple list trace.
    #[staticmethod]
    pub fn simple_list(ys: &Bound<'_, PyAny>) -> Result<Self> {
        let ys = extract_scalar_array(ys)?;
        let item_type = infer_scalar_kind(&ys)?;
        // Layout: single value list (length 1) containing all y values
        let list = wrap_as_list_array_with_field(ys.clone(), create_item_field_from_array(&ys));
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::SimpleList,
                y: item_type,
            },
            array: std::sync::Arc::new(list),
        })
    }

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
        let x_list = wrap_as_list_array(std::sync::Arc::new(xs));
        let y_item_field = create_item_field_from_array(&ys);
        let y_list = wrap_as_list_array_with_field(ys.clone(), y_item_field);
        // Must match TraceType::VariableStep storage: Struct { x: List<f64>, y: List<item> }
        let fields = vec![
            Field::new("x", x_list.data_type().clone(), false),
            Field::new("y", y_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![std::sync::Arc::new(x_list), std::sync::Arc::new(y_list)],
            None,
        );
        let item_type = infer_scalar_kind(&ys)?;
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::VariableStep,
                y: item_type,
            },
            array: std::sync::Arc::new(array),
        })
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
        let y_item_field = create_item_field_from_array(&ys);
        let y_list = wrap_as_list_array_with_field(ys.clone(), y_item_field);
        // Must match TraceType::FixedStep storage: Struct { x0: f64, step: f64, y: List<item> }
        let fields = vec![
            Field::new("x0", DataType::Float64, false),
            Field::new("step", DataType::Float64, false),
            Field::new("y", y_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![
                std::sync::Arc::new(x0),
                std::sync::Arc::new(dx),
                std::sync::Arc::new(y_list),
            ],
            None,
        );
        let item_type = infer_scalar_kind(&ys)?;
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::FixedStep,
                y: item_type,
            },
            array: std::sync::Arc::new(array),
        })
    }
}

// Internal (Rust-only) helpers
impl Trace {
    pub(crate) fn dataset_dtype(&self) -> &DatasetDataType {
        &self.dtype
    }

    pub(crate) fn array(&self) -> &ArrayRef {
        &self.array
    }
}

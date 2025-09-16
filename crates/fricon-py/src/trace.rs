use std::sync::Arc;

use anyhow::{Result, bail, ensure};
use arrow::{
    array::{Array, ArrayRef, Float64Array, ListArray, StructArray, make_array},
    buffer::OffsetBuffer,
    datatypes::{DataType, Field, FieldRef},
    pyarrow::PyArrowType,
};
use num::complex::Complex64;
use numpy::{PyArray1, PyArrayMethods};
use pyo3::{
    prelude::*,
    types::{PyComplex, PyFloat, PySequence},
};

use crate::conversion::extract_float_array;
use fricon::{FriconTypeExt, ScalarKind, TraceType, dataset_schema::DatasetDataType};

fn create_item_field_from_array(array: &ArrayRef) -> Field {
    let dt = array.data_type();
    if dt.is_complex() {
        // preserve complex extension metadata
        fricon::ComplexType::field("item", false)
    } else {
        Field::new("item", dt.clone(), false)
    }
}

fn wrap_as_list_array_with_field(array: ArrayRef, item_field: FieldRef) -> ListArray {
    // Build List(Field<item>) data type implicitly via ListArray constructor which takes the item field.
    ListArray::new(
        item_field,
        OffsetBuffer::from_lengths([array.len()]),
        array,
        None,
    )
}

fn wrap_as_list_array(array: ArrayRef) -> ListArray {
    let item_field = create_item_field_from_array(&array);
    wrap_as_list_array_with_field(array, Arc::new(item_field))
}

fn extract_scalar_array(values: &Bound<'_, PyAny>) -> Result<ArrayRef> {
    if let Ok(PyArrowType(data)) = values.extract::<PyArrowType<arrow::array::ArrayData>>() {
        let arr = make_array(data);
        return match arr.data_type() {
            DataType::Float64 => Ok(arr),
            t @ DataType::Struct(_) if t.is_complex() => Ok(arr),
            _ => bail!("Unsupported arrow array type for ys."),
        };
    }
    if let Ok(array_obj) = values.downcast::<PyArray1<Complex64>>() {
        let arr = array_obj.readonly();
        let view = arr.as_array();
        ensure!(!view.is_empty(), "Cannot build trace from empty array.");
        let mut real: Vec<f64> = Vec::with_capacity(view.len());
        let mut imag: Vec<f64> = Vec::with_capacity(view.len());
        for &c in view {
            real.push(c.re);
            imag.push(c.im);
        }
        let real_arr = Float64Array::from(real);
        let imag_arr = Float64Array::from(imag);
        let fields = vec![
            Field::new("real", DataType::Float64, false),
            Field::new("imag", DataType::Float64, false),
        ];
        let struct_arr = StructArray::new(
            fields.into(),
            vec![Arc::new(real_arr), Arc::new(imag_arr)],
            None,
        );
        return Ok(Arc::new(struct_arr));
    }
    if let Ok(array_obj) = values.downcast::<PyArray1<f64>>() {
        let arr = array_obj.readonly();
        let view = arr.as_array();
        ensure!(!view.is_empty(), "Cannot build trace from empty array.");
        let float_arr = Float64Array::from_iter_values(view.iter().copied());
        return Ok(Arc::new(float_arr));
    }
    if let Ok(seq) = values.downcast::<PySequence>() {
        ensure!(seq.len()? > 0, "Cannot build trace from empty sequence.");
        let first = seq.get_item(0)?;
        if first.is_instance_of::<PyFloat>() {
            let mut b = Float64Array::builder(seq.len()?);
            for v in seq.try_iter()? {
                b.append_value(v?.extract::<f64>()?);
            }
            return Ok(Arc::new(b.finish()));
        }
        if first.is_instance_of::<PyComplex>() {
            // Build complex struct array: StructArray{real: Float64Array, imag: Float64Array}
            let len = seq.len()?;
            let mut real: Vec<f64> = Vec::with_capacity(len);
            let mut imag: Vec<f64> = Vec::with_capacity(len);
            for v in seq.try_iter()? {
                let c: Complex64 = v?.extract()?;
                real.push(c.re);
                imag.push(c.im);
            }
            let real_arr = Float64Array::from(real);
            let imag_arr = Float64Array::from(imag);
            let fields = vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ];
            let struct_arr = StructArray::new(
                fields.into(),
                vec![Arc::new(real_arr), Arc::new(imag_arr)],
                None,
            );
            return Ok(Arc::new(struct_arr));
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
    ///     y: Sequence or Arrow array of scalar values.
    ///
    /// Returns:
    ///     A simple list trace.
    #[staticmethod]
    pub fn simple_list(y: &Bound<'_, PyAny>) -> Result<Self> {
        let y = extract_scalar_array(y)?;
        let item_type = infer_scalar_kind(&y)?;
        let list = wrap_as_list_array(y);
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::SimpleList,
                y: item_type,
            },
            array: Arc::new(list),
        })
    }

    /// Create a new trace with variable x steps.
    ///
    /// Parameters:
    ///     x: List of x-axis values.
    ///     y: List of y-axis values.
    ///
    /// Returns:
    ///     A variable-step trace.
    #[staticmethod]
    pub fn variable_step(x: &Bound<'_, PyAny>, y: &Bound<'_, PyAny>) -> Result<Self> {
        let x = extract_float_array(x)?;
        let y = extract_scalar_array(y)?;
        ensure!(x.len() == y.len(), "Length of `x` and `y` should be equal.");
        let item_type = infer_scalar_kind(&y)?;
        let x_list = wrap_as_list_array(Arc::new(x));
        let y_list = wrap_as_list_array(y);
        let fields = vec![
            Field::new("x", x_list.data_type().clone(), false),
            Field::new("y", y_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![Arc::new(x_list), Arc::new(y_list)],
            None,
        );
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::VariableStep,
                y: item_type,
            },
            array: Arc::new(array),
        })
    }

    /// Create a new trace with fixed x steps.
    ///
    /// Parameters:
    ///     x0: Starting x-axis value.
    ///     step: Step size of x-axis values.
    ///     y: List of y-axis values.
    ///
    /// Returns:
    ///     A fixed-step trace.
    #[staticmethod]
    pub fn fixed_step(x0: f64, step: f64, y: &Bound<'_, PyAny>) -> Result<Self> {
        let x0 = Float64Array::new_scalar(x0).into_inner();
        let step = Float64Array::new_scalar(step).into_inner();
        let y = extract_scalar_array(y)?;
        let item_type = infer_scalar_kind(&y)?;
        let y_list = wrap_as_list_array(y);
        let fields = vec![
            Field::new("x0", DataType::Float64, false),
            Field::new("step", DataType::Float64, false),
            Field::new("y", y_list.data_type().clone(), false),
        ];
        let array = StructArray::new(
            fields.into(),
            vec![Arc::new(x0), Arc::new(step), Arc::new(y_list)],
            None,
        );
        Ok(Self {
            dtype: DatasetDataType::Trace {
                variant: TraceType::FixedStep,
                y: item_type,
            },
            array: Arc::new(array),
        })
    }

    /// Convert to an arrow array.
    ///
    /// Returns:
    ///     Arrow array.
    pub fn to_arrow_array(&self) -> PyArrowType<arrow::array::ArrayData> {
        PyArrowType(self.array.to_data())
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

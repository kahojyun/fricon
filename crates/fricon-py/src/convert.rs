use std::sync::Arc;

use anyhow::{Context, bail};
use arrow_array::{Float64Array, make_array};
use arrow_pyarrow::PyArrowType;
use fricon::{DatasetRow, DatasetScalar, ScalarArray};
use indexmap::IndexMap;
use itertools::Itertools;
use num::complex::Complex64;
use numpy::{
    AllowTypeChange, PyArray1, PyArrayDescrMethods, PyArrayLike1, PyArrayMethods, PyUntypedArray,
    PyUntypedArrayMethods,
};
use pyo3::{
    Bound, Py, PyAny, PyErr, PyResult, Python,
    exceptions::{PyTypeError, PyValueError},
    intern,
    prelude::PyAnyMethods,
    sync::PyOnceLock,
    types::{PyComplex, PyFloat, PyInt},
};

use crate::Trace;

pub fn extract_float_array(values: &Bound<'_, PyAny>) -> anyhow::Result<Arc<Float64Array>> {
    if let Ok(PyArrowType(data)) = values.extract() {
        let arr = make_array(data);
        return fricon::downcast_array(arr)
            .context("The data type of the given arrow array is not float64.");
    }
    if let Ok(arr) = values.extract::<PyArrayLike1<'_, f64, AllowTypeChange>>() {
        let arr = arr.readonly();
        let arr = arr.as_array().into_iter().copied();
        let arr = Float64Array::from_iter_values(arr);
        return Ok(Arc::new(arr));
    }
    let py_type = values.get_type();
    bail!("Cannot convert values with type {py_type} to float64 array.");
}

pub fn extract_scalar_array(values: &Bound<'_, PyAny>) -> anyhow::Result<ScalarArray> {
    fn extract_with_numpy(array: &Bound<'_, PyAny>) -> PyResult<ScalarArray> {
        let np_array = as_array(array, None)?;
        let dtype = np_array.dtype();
        let kind = dtype.kind();

        match kind {
            b'f' | b'i' => {
                // Convert to float64 array
                let array_f64 = as_array(&np_array, Some("float64"))?;
                let py_array = array_f64.downcast::<PyArray1<f64>>()?;
                let array_readonly = py_array.readonly();
                Ok(array_readonly.as_array().iter().copied().collect())
            }
            b'c' => {
                // Convert to complex128 array
                let array_complex = as_array(&np_array, Some("complex128"))?;
                let py_array = array_complex.downcast::<PyArray1<Complex64>>()?;
                let array_readonly = py_array.readonly();
                Ok(array_readonly.as_array().iter().copied().collect())
            }
            _ => Err(PyTypeError::new_err(format!(
                "Unsupported numpy dtype: {dtype}"
            ))),
        }
    }

    if let Ok(PyArrowType(data)) = values.extract() {
        make_array(data)
            .try_into()
            .context("Expect float64 or struct<real, imag> array.")
    } else if let Ok(arrow_array) = extract_with_numpy(values) {
        Ok(arrow_array)
    } else {
        let py_type = values.get_type();
        bail!("Cannot convert {py_type} to scalar array.");
    }
}

fn extract_scalar(value: &Bound<'_, PyAny>) -> PyResult<DatasetScalar> {
    if let Ok(trace) = value.extract::<Trace>() {
        Ok(trace.0)
    } else if value.is_instance_of::<PyComplex>() {
        let c: Complex64 = value.extract()?;
        Ok(c.into())
    } else if value.is_instance_of::<PyFloat>() || value.is_instance_of::<PyInt>() {
        let f: f64 = value.extract()?;
        Ok(f.into())
    } else if let Ok(v) = value.extract::<Complex64>() {
        Ok(v.into())
    } else if let Ok(arr) = extract_scalar_array(value) {
        Ok(arr.into())
    } else {
        let py_type = value.get_type();
        Err(PyTypeError::new_err(format!(
            "Cannot convert {py_type} to dataset value type."
        )))
    }
}

pub fn build_row(py: Python<'_>, values: IndexMap<String, Py<PyAny>>) -> PyResult<DatasetRow> {
    let row = values
        .into_iter()
        .map(|(name, value)| {
            let scalar = extract_scalar(value.bind(py)).map_err(|e| {
                let new_e =
                    PyValueError::new_err(format!(r#"Cannot convert "{name}" to dataset value."#));
                new_e.set_cause(py, Some(e));
                new_e
            })?;
            Ok::<_, PyErr>((name, scalar))
        })
        .try_collect()?;
    Ok(DatasetRow(row))
}

fn numpy_module(py: Python<'_>) -> PyResult<&Py<PyAny>> {
    static MODULE: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
    MODULE.get_or_try_init(py, || py.import("numpy").map(Into::into))
}

fn as_array<'py>(
    value: &Bound<'py, PyAny>,
    dtype_name: Option<&str>,
) -> PyResult<Bound<'py, PyUntypedArray>> {
    let py = value.py();
    Ok(numpy_module(py)?
        .bind(py)
        .call_method1(intern!(py, "asarray"), (value, dtype_name))?
        .downcast_into()?)
}

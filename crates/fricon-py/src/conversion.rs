use anyhow::{Context, Result, bail, ensure};
use arrow::{
    array::{
        Array, ArrayData, ArrayRef, Float64Array, ListArray, RecordBatch, StructArray,
        downcast_array, make_array,
    },
    buffer::OffsetBuffer,
    datatypes::{DataType, Field, Schema},
    pyarrow::PyArrowType,
};
use fricon::{
    FriconTypeExt, TraceType,
    dataset_schema::{DatasetDataType, DatasetSchema as BusinessSchema, ScalarKind},
};
use indexmap::IndexMap;
use numpy::PyArrayMethods;
use pyo3::{
    prelude::*,
    types::{PyComplex, PyFloat, PyInt, PySequence},
};

use crate::trace::Trace;

pub fn extract_float_array(values: &Bound<'_, PyAny>) -> Result<Float64Array> {
    if let Ok(PyArrowType(data)) = values.extract() {
        let arr = make_array(data);
        if *arr.data_type() == DataType::Float64 {
            return Ok(downcast_array(&arr));
        }
        bail!("The data type of the given arrow array is not float64.");
    }
    if let Ok(arr) = values.extract::<numpy::PyArrayLike1<'_, f64, numpy::AllowTypeChange>>() {
        let arr = arr.readonly();
        let arr = arr.as_array().into_iter().copied();
        let arr = Float64Array::from_iter_values(arr);
        return Ok(arr);
    }
    let py_type = values.get_type();
    bail!("Cannot convert values with type {py_type} to float64 array.");
}

// Removed legacy Arrow-first helper functions in favor of business-type centric builders.

/// Infer DatasetDataType directly from Python value (MVP)
pub fn infer_dataset_type(
    value: &Bound<'_, PyAny>,
) -> Result<fricon::dataset_schema::DatasetDataType> {
    use fricon::TraceType;
    use fricon::dataset_schema::{DatasetDataType, ScalarKind};
    // Trace object
    if let Ok(trace) = value.downcast_exact::<Trace>() {
        let trace_data_type = trace.borrow().data_type().0.clone();
        if trace_data_type.is_trace() {
            let variant = trace_data_type
                .trace_type()
                .ok_or_else(|| anyhow::anyhow!("Unsupported trace type."))?;
            let y = if trace_data_type.is_complex() {
                ScalarKind::Complex128
            } else {
                ScalarKind::Float64
            };
            return Ok(DatasetDataType::Trace { variant, y });
        }
    }
    // Arrow array primitive acceptable
    if let Ok(PyArrowType(data)) = value.extract() {
        let arr = make_array(data);
        let dt = arr.data_type();
        if dt.is_complex() {
            return Ok(DatasetDataType::Scalar(ScalarKind::Complex128));
        }
        if matches!(dt, DataType::Float64) {
            return Ok(DatasetDataType::Scalar(ScalarKind::Float64));
        }
        bail!("Unsupported arrow array data type '{dt}'.");
    }
    // Sequence -> SimpleList trace
    if let Ok(sequence) = value.downcast::<PySequence>() {
        ensure!(
            sequence.len()? > 0,
            "Cannot infer field for empty sequence."
        );
        let first = sequence.get_item(0)?;
        if first.is_instance_of::<PyFloat>() || first.is_instance_of::<PyInt>() {
            return Ok(DatasetDataType::Trace {
                variant: TraceType::SimpleList,
                y: ScalarKind::Float64,
            });
        }
        if first.is_instance_of::<PyComplex>() {
            return Ok(DatasetDataType::Trace {
                variant: TraceType::SimpleList,
                y: ScalarKind::Complex128,
            });
        }
        bail!("Unsupported sequence element type.");
    }
    // Scalars
    if value.is_instance_of::<PyFloat>() || value.is_instance_of::<PyInt>() {
        return Ok(DatasetDataType::Scalar(ScalarKind::Float64));
    }
    if value.is_instance_of::<PyComplex>() {
        return Ok(DatasetDataType::Scalar(ScalarKind::Complex128));
    }
    bail!(
        "Unsupported python type. Only float, int, complex, trace, supported arrow arrays, and numeric sequences are allowed."
    );
}

// Legacy Arrow-first sequence/array/batch builder functions removed.

fn wrap_as_list_array(array: ArrayRef) -> ListArray {
    // Minimal retained helper: wraps array into a single list element (length 1 list of full array)
    let item_field = Field::new("item", array.data_type().clone(), false);
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

fn build_simple_list_trace_array(y_kind: ScalarKind, value: &Bound<'_, PyAny>) -> Result<ArrayRef> {
    if let Ok(trace) = value.downcast_exact::<Trace>() {
        let data_type = trace.borrow().data_type().0.clone();
        ensure!(
            data_type.is_trace(),
            "Provided Trace does not have trace data type"
        );
        return Ok(make_array(trace.borrow().to_arrow_array().0.clone()));
    }
    let sequence = match value.downcast::<PySequence>() {
        Ok(seq) => seq,
        Err(_e) => bail!("Expected a Trace or a Python sequence for SimpleList trace"),
    };
    ensure!(
        sequence.len()? > 0,
        "Cannot build trace from empty sequence."
    );
    // Build inner scalar values array
    let mut float_vec: Vec<f64> = Vec::new();
    let mut complex_real: Vec<f64> = Vec::new();
    let mut complex_imag: Vec<f64> = Vec::new();
    match y_kind {
        ScalarKind::Float64 => {
            for item in sequence.try_iter()? {
                float_vec.push(item?.extract::<f64>()?);
            }
            let ys = Float64Array::from(float_vec);
            let list_array = wrap_as_list_array(std::sync::Arc::new(ys));
            // Represent simple list trace as struct with ys only (like variable/fixed forms). For MVP we mimic Trace.variable_step with implicit xs (index).
            // xs omitted: we encode as a single-field struct { ys } for SimpleList.
            let field = Field::new("ys", list_array.data_type().clone(), false);
            let array = StructArray::new(
                vec![field].into(),
                vec![std::sync::Arc::new(list_array)],
                None,
            );
            Ok(std::sync::Arc::new(array))
        }
        ScalarKind::Complex128 => {
            for item in sequence.try_iter()? {
                let c = item?.extract::<num::complex::Complex64>()?;
                complex_real.push(c.re);
                complex_imag.push(c.im);
            }
            let real_arr = Float64Array::from(complex_real);
            let imag_arr = Float64Array::from(complex_imag);
            // Build complex struct elements then list
            let fields = vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ];
            let struct_elems = StructArray::new(
                fields.clone().into(),
                vec![std::sync::Arc::new(real_arr), std::sync::Arc::new(imag_arr)],
                None,
            );
            let list_array = wrap_as_list_array(std::sync::Arc::new(struct_elems));
            let field = Field::new("ys", list_array.data_type().clone(), false);
            let array = StructArray::new(
                vec![field].into(),
                vec![std::sync::Arc::new(list_array)],
                None,
            );
            Ok(std::sync::Arc::new(array))
        }
    }
}

fn build_array_from_dataset_value(
    dtype: &DatasetDataType,
    value: &Bound<'_, PyAny>,
) -> Result<ArrayRef> {
    match dtype {
        DatasetDataType::Scalar(ScalarKind::Float64) => {
            if let Ok(PyArrowType(data)) = value.extract::<PyArrowType<ArrayData>>() {
                ensure!(
                    data.data_type() == &DataType::Float64,
                    "Arrow array type mismatch"
                );
                return Ok(make_array(data));
            }
            let v = value.extract::<f64>()?;
            let arr = Float64Array::new_scalar(v).into_inner();
            Ok(std::sync::Arc::new(arr))
        }
        DatasetDataType::Scalar(ScalarKind::Complex128) => {
            if let Ok(PyArrowType(data)) = value.extract::<PyArrowType<ArrayData>>() {
                ensure!(
                    data.data_type().is_complex(),
                    "Arrow array not complex struct"
                );
                return Ok(make_array(data));
            }
            let c = value.extract::<num::complex::Complex64>()?;
            let real = Float64Array::new_scalar(c.re).into_inner();
            let imag = Float64Array::new_scalar(c.im).into_inner();
            let fields = vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ];
            let struct_arr = StructArray::new(
                fields.into(),
                vec![std::sync::Arc::new(real), std::sync::Arc::new(imag)],
                None,
            );
            Ok(std::sync::Arc::new(struct_arr))
        }
        DatasetDataType::Trace {
            variant: TraceType::SimpleList,
            y,
        } => build_simple_list_trace_array(*y, value),
        DatasetDataType::Trace {
            variant: TraceType::FixedStep | TraceType::VariableStep,
            ..
        } => {
            let Ok(trace) = value.downcast_exact::<Trace>() else {
                bail!("FixedStep/VariableStep trace columns require a Trace object");
            };
            Ok(make_array(trace.borrow().to_arrow_array().0.clone()))
        }
    }
}

pub fn build_record_batch_from_dataset(
    py: Python<'_>,
    business_schema: &BusinessSchema,
    values: &IndexMap<String, PyObject>,
) -> Result<RecordBatch> {
    ensure!(
        business_schema.fields.len() == values.len(),
        "Values not compatible with schema."
    );
    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(values.len());
    let mut arrow_fields: Vec<Field> = Vec::with_capacity(values.len());
    for field in &business_schema.fields {
        let value = values
            .get(&field.name)
            .with_context(|| format!("Missing value {}", field.name))?
            .bind(py);
        let array = build_array_from_dataset_value(&field.dtype, value)
            .with_context(|| format!("Building array for column {}", field.name))?;
        arrow_fields.push(field.to_arrow_field());
        arrays.push(array);
    }
    let arrow_schema = Schema::new(arrow_fields);
    Ok(RecordBatch::try_new(
        std::sync::Arc::new(arrow_schema),
        arrays,
    )?)
}

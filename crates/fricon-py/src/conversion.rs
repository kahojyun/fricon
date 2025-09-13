use anyhow::{Context, Result, bail, ensure};
use arrow::{
    array::{
        Array, ArrayData, ArrayRef, BooleanArray, Float64Array, Int64Array, ListArray, RecordBatch,
        StringArray, StringBuilder, StructArray, downcast_array, make_array,
    },
    buffer::OffsetBuffer,
    datatypes::{DataType, Field, Schema},
    pyarrow::PyArrowType,
};
use fricon::{FriconTypeExt, dataset_schema::DatasetField};
use indexmap::IndexMap;
use itertools::Itertools;
use num::complex::Complex64;
use numpy::PyArrayMethods;
use pyo3::{
    prelude::*,
    types::{PyBool, PyComplex, PyFloat, PyInt, PySequence, PyString},
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

pub fn extract_scalar_array(values: &Bound<'_, PyAny>) -> Result<ArrayRef> {
    if let Ok(PyArrowType(data)) = values.extract() {
        let arr = make_array(data);
        return match arr.data_type() {
            DataType::Boolean | DataType::Int64 | DataType::Float64 | DataType::Utf8 => Ok(arr),
            t @ DataType::Struct(_) if t.is_complex() => Ok(arr),
            _ => bail!("The data type of the given arrow array is not float64."),
        };
    }
    if let Ok(sequence) = values.downcast::<PySequence>() {
        let field = infer_sequence_item_field("item", sequence)
            .context("Inferring sequence item field.")?;
        return build_array_from_sequence(field.data_type(), sequence);
    }
    let py_type = values.get_type();
    bail!("Cannot convert {py_type} to scalar array.");
}

/// Create a field that preserves extension metadata from an array
pub fn create_field_from_array(name: &str, array: &ArrayRef, nullable: bool) -> Field {
    let data_type = array.data_type();

    // Check if the array's data type has extension metadata
    if data_type.is_complex() {
        fricon::ComplexType::field(name, nullable)
    } else if let Some(trace_type) = data_type.trace_type() {
        let mut field = Field::new(name, data_type.clone(), nullable);
        let _ = field.try_with_extension_type(trace_type);
        field
    } else {
        Field::new(name, data_type.clone(), nullable)
    }
}

/// Create an item field that preserves extension metadata
pub fn create_item_field_from_array(array: &ArrayRef) -> Field {
    create_field_from_array("item", array, false)
}

pub fn wrap_as_list_array_with_field(array: ArrayRef, item_field: Field) -> ListArray {
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

pub fn wrap_as_list_array(array: ArrayRef) -> ListArray {
    // Create a proper list field that preserves extension metadata
    let item_field = create_item_field_from_array(&array);
    wrap_as_list_array_with_field(array, item_field)
}

pub fn infer_scalar_field(name: &str, value: &Bound<'_, PyAny>) -> Result<Field> {
    // Check bool first because bool is a subclass of int.
    if value.is_instance_of::<PyBool>() {
        Ok(Field::new(name, DataType::Boolean, false))
    } else if value.is_instance_of::<PyInt>() {
        Ok(Field::new(name, DataType::Int64, false))
    } else if value.is_instance_of::<PyFloat>() {
        Ok(Field::new(name, DataType::Float64, false))
    } else if value.is_instance_of::<PyComplex>() {
        Ok(fricon::ComplexType::field(name, false))
    } else if value.is_instance_of::<PyString>() {
        Ok(Field::new(name, DataType::Utf8, false))
    } else {
        let py_type = value.get_type();
        bail!("Cannot infer scalar arrow field for python type '{py_type}'.");
    }
}

pub fn infer_sequence_item_field(name: &str, sequence: &Bound<'_, PySequence>) -> Result<Field> {
    ensure!(
        sequence.len()? > 0,
        "Cannot infer field for empty sequence."
    );
    let first_item = sequence.get_item(0)?;
    infer_scalar_field(name, &first_item)
}

pub fn infer_sequence_field(name: &str, sequence: &Bound<'_, PySequence>) -> Result<Field> {
    let item_field = infer_sequence_item_field("item", sequence)?;
    Ok(fricon::TraceType::SimpleList.field(name, std::sync::Arc::new(item_field), false))
}

/// Infer [`arrow::datatypes::Field`] from name and value.
///
/// This function infers the complete Field including extension type metadata,
/// which is not available when only inferring DataType.
///
/// Currently supports:
///
/// 1. Scalar types: bool, int, float, complex, str
/// 2. [`Trace`]
/// 3. [`arrow::array::Array`]
/// 4. Python Sequence protocol
///
/// Infer DatasetField from Python value (MVP types only: float64, complex128, traces)
/// Returns an error for unsupported types instead of inferring arbitrary Arrow types
#[expect(dead_code)]
pub fn infer_dataset_field(name: &str, value: &Bound<'_, PyAny>) -> Result<DatasetField> {
    // Try to infer Arrow field first using existing logic
    let arrow_field = infer_field_arrow(name, value)?;

    // Convert to DatasetField, ensuring only MVP types are supported
    DatasetField::from_arrow_field(&arrow_field)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported data type '{}' for field '{}'. Only float64, complex128, and trace types are supported in this version.",
                arrow_field.data_type(),
                name
            )
        })
}

/// Original infer_field function renamed to infer_field_arrow for internal use
/// TODO: support numpy array
pub fn infer_field_arrow(name: &str, value: &Bound<'_, PyAny>) -> Result<Field> {
    if let Ok(trace) = value.downcast_exact::<Trace>() {
        let trace_data_type = trace.borrow().data_type().0.clone();
        // For trace objects, preserve the extension metadata if it's a trace type
        if trace_data_type.is_trace() {
            if let Some(trace_type) = trace_data_type.trace_type() {
                let mut field = Field::new(name, trace_data_type, false);
                let _ = field.try_with_extension_type(trace_type);
                Ok(field)
            } else {
                Ok(Field::new(name, trace_data_type, false))
            }
        } else {
            Ok(Field::new(name, trace_data_type, false))
        }
    } else if let Ok(PyArrowType(data)) = value.extract() {
        let arr = make_array(data);
        // Use the utility function to preserve extension metadata
        Ok(create_field_from_array(name, &arr, false))
    } else if let Ok(sequence) = value.downcast::<PySequence>() {
        infer_sequence_field(name, sequence)
    } else {
        infer_scalar_field(name, value)
    }
}

pub fn infer_schema(
    py: Python<'_>,
    initial_schema: &Schema,
    values: &IndexMap<String, PyObject>,
) -> Result<Schema> {
    let mut fields: Vec<Field> = Vec::new();

    for (name, value) in values {
        if let Ok(field) = initial_schema.field_with_name(name) {
            fields.push(field.clone());
        } else {
            let field = infer_field_arrow(name, value.bind(py))
                .with_context(|| format!("Inferring field for column '{name}'."))?;
            fields.push(field);
        }
    }

    Ok(Schema::new(fields))
}

pub fn build_array_from_sequence(
    data_type: &DataType,
    sequence: &Bound<'_, PySequence>,
) -> Result<ArrayRef> {
    match data_type {
        DataType::Boolean => {
            let mut builder = BooleanArray::builder(sequence.len()?);
            for v in sequence.try_iter()? {
                let v = v?.extract()?;
                builder.append_value(v);
            }
            Ok(std::sync::Arc::new(builder.finish()))
        }
        DataType::Int64 => {
            let mut builder = Int64Array::builder(sequence.len()?);
            for v in sequence.try_iter()? {
                let v = v?.extract()?;
                builder.append_value(v);
            }
            Ok(std::sync::Arc::new(builder.finish()))
        }
        DataType::Float64 => {
            let mut builder = Float64Array::builder(sequence.len()?);
            for v in sequence.try_iter()? {
                let v = v?.extract()?;
                builder.append_value(v);
            }
            Ok(std::sync::Arc::new(builder.finish()))
        }
        DataType::Utf8 => {
            let mut builder = StringBuilder::new();
            for v in sequence.try_iter()? {
                let v = v?.extract::<String>()?;
                builder.append_value(v);
            }
            Ok(std::sync::Arc::new(builder.finish()))
        }
        _ => bail!("Unsupported data type."),
    }
}

pub fn build_list(
    field: std::sync::Arc<Field>,
    sequence: &Bound<'_, PySequence>,
) -> Result<ListArray> {
    let values = build_array_from_sequence(field.data_type(), sequence)?;
    let offsets = OffsetBuffer::from_lengths([values.len()]);
    Ok(ListArray::try_new(field, offsets, values, None)?)
}

pub fn build_array(value: &Bound<'_, PyAny>, data_type: &DataType) -> Result<ArrayRef> {
    if let Ok(PyArrowType(data)) = value.extract::<PyArrowType<ArrayData>>() {
        ensure!(
            data.data_type() == data_type,
            "Different data type: schema: {data_type}, value: {}",
            data.data_type()
        );
        return Ok(make_array(data));
    }
    match data_type {
        DataType::Boolean => {
            let Ok(value) = value.extract::<bool>() else {
                bail!("Not a boolean value.")
            };
            let array = BooleanArray::new_scalar(value).into_inner();
            Ok(std::sync::Arc::new(array))
        }
        DataType::Int64 => {
            let Ok(value) = value.extract::<i64>() else {
                bail!("Failed to extract int64 value.")
            };
            let array = Int64Array::new_scalar(value).into_inner();
            Ok(std::sync::Arc::new(array))
        }
        DataType::Float64 => {
            let Ok(value) = value.extract::<f64>() else {
                bail!("Failed to extract float64 value.")
            };
            let array = Float64Array::new_scalar(value).into_inner();
            Ok(std::sync::Arc::new(array))
        }
        DataType::Utf8 => {
            let Ok(value) = value.extract::<String>() else {
                bail!("Failed to extract float64 value.")
            };
            let array = StringArray::new_scalar(value).into_inner();
            Ok(std::sync::Arc::new(array))
        }
        // complex scalar
        t @ DataType::Struct(_) if t.is_complex() => {
            let Ok(value) = value.extract::<Complex64>() else {
                bail!("Failed to extract complex value.")
            };
            let real = Float64Array::new_scalar(value.re).into_inner();
            let imag = Float64Array::new_scalar(value.im).into_inner();
            let fields = vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ];
            let array = StructArray::new(
                fields.into(),
                vec![std::sync::Arc::new(real), std::sync::Arc::new(imag)],
                None,
            );
            Ok(std::sync::Arc::new(array))
        }
        // Trace
        t @ DataType::Struct(_fields) => {
            let Ok(value) = value.downcast_exact::<Trace>() else {
                bail!("Failed to extract `Trace` value.")
            };
            let value = value.borrow();
            if *t != value.data_type().0 {
                bail!("Incompatible data type.")
            }
            let array = value.to_arrow_array().0;
            Ok(make_array(array))
        }
        // Sequence
        DataType::List(field) => {
            let Ok(value) = value.downcast() else {
                bail!("Value is not a python `Sequence`");
            };
            let list = build_list(field.clone(), value)?;
            Ok(std::sync::Arc::new(list))
        }
        _ => {
            bail!("Unsupported data type {data_type}, please manually construct a `pyarrow.Array`.")
        }
    }
}

pub fn build_record_batch(
    py: Python<'_>,
    schema: std::sync::Arc<Schema>,
    values: &IndexMap<String, PyObject>,
) -> Result<RecordBatch> {
    ensure!(
        schema.fields().len() == values.len(),
        "Values not compatible with schema."
    );
    let columns = schema
        .fields()
        .into_iter()
        .map(|field| {
            let name = field.name();
            let value = values
                .get(name)
                .with_context(|| format!("Missing value {name}"))?
                .bind(py);
            build_array(value, field.data_type())
                .with_context(|| format!("Building array for column {name}"))
        })
        .try_collect()?;
    Ok(RecordBatch::try_new(schema, columns)?)
}

use arrow::datatypes::DataType;
use pyo3::prelude::*;

use arrow::pyarrow::PyArrowType;

/// Get a pyarrow data type representing 128 bit complex number.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn complex128() -> PyArrowType<DataType> {
    PyArrowType(fricon::ComplexType::storage_type())
}

/// Get a pyarrow data type representing a simple list trace.
///
/// Parameters:
///     item: Data type of the y values.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn simple_list_trace(item: PyArrowType<DataType>) -> PyArrowType<DataType> {
    PyArrowType(fricon::TraceType::simple_list().storage_type(item.0))
}

/// Get a pyarrow data type representing a fixed step trace.
///
/// Parameters:
///     item: Data type of the y values.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn fixed_step_trace(item: PyArrowType<DataType>) -> PyArrowType<DataType> {
    PyArrowType(fricon::TraceType::fixed_step().storage_type(item.0))
}

/// Get a pyarrow data type representing a variable step trace.
///
/// Parameters:
///     item: Data type of the y values.
///
/// Returns:
///     A pyarrow data type.
#[pyfunction]
pub fn variable_step_trace(item: PyArrowType<DataType>) -> PyArrowType<DataType> {
    PyArrowType(fricon::TraceType::variable_step().storage_type(item.0))
}

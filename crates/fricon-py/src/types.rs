use arrow::datatypes::{DataType, Field};
use pyo3::prelude::*;

use arrow::pyarrow::PyArrowType;

/// Get a pyarrow field representing 128 bit complex number.
///
/// Parameters:
///     name: Field name (defaults to empty string)
///     nullable: Whether the field is nullable (defaults to true)
///
/// Returns:
///     A pyarrow field.
#[pyfunction]
pub fn complex128_field(name: String, nullable: Option<bool>) -> PyArrowType<Field> {
    let nullable = nullable.unwrap_or(true);
    PyArrowType(fricon::ComplexType::field(&name, nullable))
}

/// Get a pyarrow field representing a simple list trace.
///
/// Parameters:
///     name: Field name (defaults to empty string)
///     item_type: Data type of the y values (required)
///     nullable: Whether the field is nullable (defaults to true)
///
/// Returns:
///     A pyarrow field.
#[pyfunction]
pub fn simple_list_trace_field(
    name: String,
    item_type: Option<PyArrowType<DataType>>,
    nullable: Option<bool>,
) -> PyArrowType<Field> {
    let nullable = nullable.unwrap_or(true);
    let item_type = item_type.expect("item_type is required for trace fields");
    PyArrowType(fricon::TraceType::simple_list().field(&name, item_type.0, nullable))
}

/// Get a pyarrow field representing a fixed step trace.
///
/// Parameters:
///     name: Field name (defaults to empty string)
///     item_type: Data type of the y values (required)
///     nullable: Whether the field is nullable (defaults to true)
///
/// Returns:
///     A pyarrow field.
#[pyfunction]
pub fn fixed_step_trace_field(
    name: String,
    item_type: Option<PyArrowType<DataType>>,
    nullable: Option<bool>,
) -> PyArrowType<Field> {
    let nullable = nullable.unwrap_or(true);
    let item_type = item_type.expect("item_type is required for trace fields");
    PyArrowType(fricon::TraceType::fixed_step().field(&name, item_type.0, nullable))
}

/// Get a pyarrow field representing a variable step trace.
///
/// Parameters:
///     name: Field name (defaults to empty string)
///     item_type: Data type of the y values (required)
///     nullable: Whether the field is nullable (defaults to true)
///
/// Returns:
///     A pyarrow field.
#[pyfunction]
pub fn variable_step_trace_field(
    name: String,
    item_type: Option<PyArrowType<DataType>>,
    nullable: Option<bool>,
) -> PyArrowType<Field> {
    let nullable = nullable.unwrap_or(true);
    let item_type = item_type.expect("item_type is required for trace fields");
    PyArrowType(fricon::TraceType::variable_step().field(&name, item_type.0, nullable))
}

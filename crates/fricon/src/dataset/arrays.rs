use std::sync::Arc;

use arrow_array::{
    Array, ArrayRef, Float64Array, ListArray, StructArray, cast::AsArray, types::Float64Type,
};
use arrow_buffer::OffsetBuffer;
use arrow_schema::{DataType, Field, extension::ExtensionType};
use derive_more::From;
use num::complex::Complex64;

use crate::dataset::{
    Error,
    scalars::{DatasetScalar, FixedStepTrace, VariableStepTrace},
    types::{ComplexType, DatasetDataType, ScalarKind, TraceKind},
};

#[derive(Debug, Clone)]
pub struct ComplexArray(Arc<StructArray>);

impl ComplexArray {
    pub fn real(&self) -> &Float64Array {
        self.0.column(0).as_ref().as_primitive::<Float64Type>()
    }

    pub fn imag(&self) -> &Float64Array {
        self.0.column(1).as_ref().as_primitive::<Float64Type>()
    }
}

impl From<ComplexArray> for ArrayRef {
    fn from(value: ComplexArray) -> Self {
        value.0
    }
}

impl TryFrom<ArrayRef> for ComplexArray {
    type Error = Error;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let scalar_kind: ScalarKind = value.data_type().try_into()?;
        if scalar_kind == ScalarKind::Complex {
            let struct_array = value.as_struct_opt().ok_or(Error::IncompatibleType)?;
            Ok(ComplexArray(Arc::new(struct_array.clone())))
        } else {
            Err(Error::IncompatibleType)
        }
    }
}

impl FromIterator<Complex64> for ComplexArray {
    fn from_iter<I: IntoIterator<Item = Complex64>>(iter: I) -> Self {
        let (real_values, imag_values): (Vec<_>, Vec<_>) =
            iter.into_iter().map(|c| (c.re, c.im)).unzip();
        let real = Arc::new(Float64Array::from(real_values));
        let imag = Arc::new(Float64Array::from(imag_values));
        let struct_array = StructArray::new(ComplexType::fields(), vec![real, imag], None);
        ComplexArray(Arc::new(struct_array))
    }
}

#[derive(Debug, Clone)]
pub struct ScalarArray {
    array: ArrayRef,
    scalar_kind: ScalarKind,
}

impl ScalarArray {
    #[must_use]
    pub fn scalar_kind(&self) -> ScalarKind {
        self.scalar_kind
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.array.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<ScalarArray> for ArrayRef {
    fn from(array: ScalarArray) -> Self {
        array.array
    }
}

impl TryFrom<ArrayRef> for ScalarArray {
    type Error = Error;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let scalar_kind = value.data_type().try_into()?;
        Ok(Self {
            array: value,
            scalar_kind,
        })
    }
}

impl From<ComplexArray> for ScalarArray {
    fn from(array: ComplexArray) -> Self {
        Self {
            array: array.into(),
            scalar_kind: ScalarKind::Complex,
        }
    }
}

impl FromIterator<f64> for ScalarArray {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        Self {
            array: Arc::new(Float64Array::from_iter_values(iter)),
            scalar_kind: ScalarKind::Numeric,
        }
    }
}

impl FromIterator<Complex64> for ScalarArray {
    fn from_iter<T: IntoIterator<Item = Complex64>>(iter: T) -> Self {
        Self {
            array: ComplexArray::from_iter(iter).into(),
            scalar_kind: ScalarKind::Complex,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScalarListArray {
    array: Arc<ListArray>,
    scalar_kind: ScalarKind,
}

impl ScalarListArray {
    pub fn from_single_item(item: ScalarArray) -> Self {
        let item_field = Arc::new(item.scalar_kind.to_item_field());
        let offsets = OffsetBuffer::from_lengths([item.array.len()]);
        let array = Arc::new(ListArray::new(item_field, offsets, item.array, None));
        Self {
            array,
            scalar_kind: item.scalar_kind,
        }
    }

    pub fn scalar_kind(&self) -> ScalarKind {
        self.scalar_kind
    }
}

impl From<ScalarListArray> for ArrayRef {
    fn from(value: ScalarListArray) -> Self {
        value.array
    }
}

impl TryFrom<ArrayRef> for ScalarListArray {
    type Error = Error;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let array: &ListArray = value.as_list_opt().ok_or(Error::IncompatibleType)?;
        let scalar_kind = array.values().data_type().try_into()?;
        Ok(Self {
            array: Arc::new(array.clone()),
            scalar_kind,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FixedStepTraceArray {
    array: Arc<StructArray>,
    scalar_kind: ScalarKind,
}

impl FixedStepTraceArray {
    pub fn scalar_kind(&self) -> ScalarKind {
        self.scalar_kind
    }

    pub fn x0(&self) -> &Float64Array {
        self.array.column(0).as_ref().as_primitive::<Float64Type>()
    }

    pub fn step(&self) -> &Float64Array {
        self.array.column(1).as_ref().as_primitive::<Float64Type>()
    }

    pub fn y(&self) -> ScalarListArray {
        let y: &ListArray = self.array.column(2).as_ref().as_list::<i32>();
        ScalarListArray {
            array: Arc::new(y.clone()),
            scalar_kind: self.scalar_kind,
        }
    }

    pub fn expand_row(&self, row: usize) -> Option<(Vec<f64>, ArrayRef)> {
        if row >= self.array.len() || self.array.is_null(row) {
            return None;
        }

        let x0 = self.x0().value(row);
        let step = self.step().value(row);
        let y_array = self.y().array;
        if y_array.is_null(row) {
            return None;
        }
        let y_values = y_array.value(row);

        #[expect(
            clippy::cast_precision_loss,
            reason = "Array index is unlikely to exceed 2^53"
        )]
        let x = (0..y_values.len())
            .map(|i| x0 + (i as f64) * step)
            .collect();
        Some((x, y_values))
    }
}

impl From<FixedStepTrace> for FixedStepTraceArray {
    fn from(value: FixedStepTrace) -> Self {
        let x0: ArrayRef = Arc::new(Float64Array::from(vec![value.x0()]));
        let step: ArrayRef = Arc::new(Float64Array::from(vec![value.step()]));
        let y: ArrayRef = ScalarListArray::from_single_item(value.y().clone()).into();
        let struct_array = StructArray::try_from(vec![("x0", x0), ("step", step), ("y", y)])
            .expect("Should have same length of 1.");
        Self {
            array: Arc::new(struct_array),
            scalar_kind: value.scalar_kind(),
        }
    }
}

impl From<FixedStepTraceArray> for ArrayRef {
    fn from(value: FixedStepTraceArray) -> Self {
        value.array
    }
}

impl TryFrom<ArrayRef> for FixedStepTraceArray {
    type Error = Error;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let array = value.as_struct_opt().ok_or(Error::IncompatibleType)?;
        TraceKind::FixedStep.supports_data_type(array.data_type())?;
        let y = array
            .column(2)
            .as_list_opt::<i32>()
            .ok_or(Error::IncompatibleType)?;
        let scalar_kind = y.values().data_type().try_into()?;
        Ok(Self {
            array: Arc::new(array.clone()),
            scalar_kind,
        })
    }
}

#[derive(Debug, Clone)]
pub struct VariableStepTraceArray {
    array: Arc<StructArray>,
    scalar_kind: ScalarKind,
}

impl VariableStepTraceArray {
    pub fn scalar_kind(&self) -> ScalarKind {
        self.scalar_kind
    }

    pub fn x(&self) -> &ListArray {
        self.array.column(0).as_ref().as_list::<i32>()
    }

    pub fn y(&self) -> ScalarListArray {
        let y: &ListArray = self.array.column(1).as_ref().as_list::<i32>();
        ScalarListArray {
            array: Arc::new(y.clone()),
            scalar_kind: self.scalar_kind,
        }
    }

    pub fn expand_row(&self, row: usize) -> Result<Option<(Vec<f64>, ArrayRef)>, Error> {
        if row >= self.array.len() || self.array.is_null(row) {
            return Ok(None);
        }

        let x_array = self.x();
        let y_array = self.y().array;

        if x_array.is_null(row) || y_array.is_null(row) {
            return Ok(None);
        }

        let x_values = x_array.value(row);
        let y_values = y_array.value(row);

        let x_f64: &Float64Array = x_values.as_primitive_opt().ok_or(Error::IncompatibleType)?;

        Ok(Some((x_f64.values().to_vec(), y_values)))
    }
}

impl From<VariableStepTrace> for VariableStepTraceArray {
    fn from(value: VariableStepTrace) -> Self {
        let x_field = Arc::new(Field::new_list_field(DataType::Float64, false));
        let x_offsets = OffsetBuffer::from_lengths([value.x().len()]);
        let x: ArrayRef = Arc::new(ListArray::new(x_field, x_offsets, value.x().clone(), None));
        let y: ArrayRef = ScalarListArray::from_single_item(value.y().clone()).into();
        let struct_array =
            StructArray::try_from(vec![("x", x), ("y", y)]).expect("Should have same length of 1.");
        Self {
            array: Arc::new(struct_array),
            scalar_kind: value.scalar_kind(),
        }
    }
}

impl From<VariableStepTraceArray> for ArrayRef {
    fn from(value: VariableStepTraceArray) -> Self {
        value.array
    }
}

impl TryFrom<ArrayRef> for VariableStepTraceArray {
    type Error = Error;
    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let array = value.as_struct_opt().ok_or(Error::IncompatibleType)?;
        TraceKind::VariableStep.supports_data_type(array.data_type())?;
        let y = array
            .column(1)
            .as_list_opt::<i32>()
            .ok_or(Error::IncompatibleType)?;
        let scalar_kind = y.values().data_type().try_into()?;
        Ok(Self {
            array: Arc::new(array.clone()),
            scalar_kind,
        })
    }
}

#[derive(Debug, Clone, From)]
pub enum DatasetArray {
    Numeric(Arc<Float64Array>),
    Complex(ComplexArray),
    SimpleTrace(ScalarListArray),
    FixedStepTrace(FixedStepTraceArray),
    VariableStepTrace(VariableStepTraceArray),
}

impl DatasetArray {
    #[must_use]
    pub fn data_type(&self) -> DatasetDataType {
        match self {
            DatasetArray::Numeric(_) => DatasetDataType::Scalar(ScalarKind::Numeric),
            DatasetArray::Complex(_) => DatasetDataType::Scalar(ScalarKind::Complex),
            DatasetArray::SimpleTrace(t) => {
                DatasetDataType::Trace(TraceKind::Simple, t.scalar_kind())
            }
            DatasetArray::FixedStepTrace(t) => {
                DatasetDataType::Trace(TraceKind::FixedStep, t.scalar_kind())
            }
            DatasetArray::VariableStepTrace(t) => {
                DatasetDataType::Trace(TraceKind::VariableStep, t.scalar_kind())
            }
        }
    }

    #[must_use]
    pub fn num_rows(&self) -> usize {
        match self {
            DatasetArray::Numeric(a) => a.len(),
            DatasetArray::Complex(a) => a.0.len(),
            DatasetArray::SimpleTrace(a) => a.array.len(),
            DatasetArray::FixedStepTrace(a) => a.array.len(),
            DatasetArray::VariableStepTrace(a) => a.array.len(),
        }
    }

    #[must_use]
    pub fn as_numeric(&self) -> Option<&Float64Array> {
        match self {
            DatasetArray::Numeric(a) => Some(a.as_ref()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_complex(&self) -> Option<&ComplexArray> {
        match self {
            DatasetArray::Complex(a) => Some(a),
            _ => None,
        }
    }

    pub fn expand_trace(&self, row: usize) -> Result<Option<(Vec<f64>, ArrayRef)>, Error> {
        match self {
            DatasetArray::Numeric(_) | DatasetArray::Complex(_) => Err(Error::IncompatibleType),
            DatasetArray::SimpleTrace(t) => {
                if row >= t.array.len() || t.array.is_null(row) {
                    return Ok(None);
                }
                let values = t.array.value(row);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "Array index is unlikely to exceed 2^53"
                )]
                let x = (0..values.len()).map(|i| i as f64).collect();
                Ok(Some((x, values)))
            }
            DatasetArray::FixedStepTrace(t) => Ok(t.expand_row(row)),
            DatasetArray::VariableStepTrace(t) => t.expand_row(row),
        }
    }
}

impl From<DatasetScalar> for DatasetArray {
    fn from(value: DatasetScalar) -> Self {
        match value {
            DatasetScalar::Numeric(v) => Arc::new(Float64Array::from(vec![v])).into(),
            DatasetScalar::Complex(v) => ComplexArray::from_iter(vec![v]).into(),
            DatasetScalar::SimpleTrace(v) => ScalarListArray::from_single_item(v).into(),
            DatasetScalar::FixedStepTrace(v) => FixedStepTraceArray::from(v).into(),
            DatasetScalar::VariableStepTrace(v) => VariableStepTraceArray::from(v).into(),
        }
    }
}

impl TryFrom<ArrayRef> for DatasetArray {
    type Error = Error;

    fn try_from(value: ArrayRef) -> Result<Self, Self::Error> {
        let data_type = value.data_type();
        if let Some((trace, _)) = TraceKind::parse_data_type(data_type) {
            match trace {
                TraceKind::Simple => Ok(DatasetArray::SimpleTrace(value.try_into()?)),
                TraceKind::FixedStep => Ok(DatasetArray::FixedStepTrace(value.try_into()?)),
                TraceKind::VariableStep => Ok(DatasetArray::VariableStepTrace(value.try_into()?)),
            }
        } else {
            let scalar_kind: ScalarKind = data_type.try_into()?;
            match scalar_kind {
                ScalarKind::Numeric => {
                    let array: &Float64Array =
                        value.as_primitive_opt().ok_or(Error::IncompatibleType)?;
                    Ok(DatasetArray::Numeric(Arc::new(array.clone())))
                }
                ScalarKind::Complex => Ok(DatasetArray::Complex(value.try_into()?)),
            }
        }
    }
}

impl From<DatasetArray> for ArrayRef {
    fn from(value: DatasetArray) -> Self {
        match value {
            DatasetArray::Numeric(a) => a,
            DatasetArray::Complex(a) => a.into(),
            DatasetArray::SimpleTrace(a) => a.into(),
            DatasetArray::FixedStepTrace(a) => a.into(),
            DatasetArray::VariableStepTrace(a) => a.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::scalars::{DatasetScalar, FixedStepTrace, VariableStepTrace};

    fn make_list(values: Vec<f64>, item_nullable: bool) -> ArrayRef {
        let field = Arc::new(Field::new_list_field(DataType::Float64, item_nullable));
        let offsets = OffsetBuffer::from_lengths([values.len()]);
        Arc::new(ListArray::new(
            field,
            offsets,
            Arc::new(Float64Array::from(values)),
            None,
        ))
    }

    fn make_variable_trace_struct(x: Vec<f64>, y: Vec<f64>, nullable: bool) -> ArrayRef {
        let x_list = make_list(x, nullable);
        let y_list = make_list(y, nullable);
        let struct_array = StructArray::try_new(
            vec![
                Arc::new(Field::new("x", x_list.data_type().clone(), nullable)),
                Arc::new(Field::new("y", y_list.data_type().clone(), nullable)),
            ]
            .into(),
            vec![x_list, y_list],
            None,
        )
        .expect("valid variable-step trace struct");
        Arc::new(struct_array)
    }

    fn make_fixed_trace_struct(x0: f64, step: f64, y: Vec<f64>, nullable: bool) -> ArrayRef {
        let struct_array = StructArray::try_new(
            vec![
                Arc::new(Field::new("x0", DataType::Float64, nullable)),
                Arc::new(Field::new("step", DataType::Float64, nullable)),
                Arc::new(Field::new(
                    "y",
                    DataType::new_list(DataType::Float64, nullable),
                    nullable,
                )),
            ]
            .into(),
            vec![
                Arc::new(Float64Array::from(vec![x0])) as ArrayRef,
                Arc::new(Float64Array::from(vec![step])) as ArrayRef,
                make_list(y, nullable),
            ],
            None,
        )
        .expect("valid fixed-step trace struct");
        Arc::new(struct_array)
    }

    #[test]
    fn variable_step_trace_array_try_from_infers_numeric_scalar_kind() {
        let trace = VariableStepTrace::new(
            Arc::new(Float64Array::from(vec![0.0, 1.0, 2.0])),
            ScalarArray::from_iter(vec![10.0, 20.0, 30.0]),
        )
        .expect("valid trace");
        let array: ArrayRef = DatasetArray::from(DatasetScalar::VariableStepTrace(trace)).into();
        let parsed = VariableStepTraceArray::try_from(array).expect("parse variable-step trace");
        assert_eq!(parsed.scalar_kind(), ScalarKind::Numeric);
    }

    #[test]
    fn fixed_step_trace_array_try_from_infers_numeric_scalar_kind() {
        let trace = FixedStepTrace::new(0.0, 0.5, ScalarArray::from_iter(vec![1.0, 2.0, 3.0]));
        let array: ArrayRef = DatasetArray::from(DatasetScalar::FixedStepTrace(trace)).into();
        let parsed = FixedStepTraceArray::try_from(array).expect("parse fixed-step trace");
        assert_eq!(parsed.scalar_kind(), ScalarKind::Numeric);
    }

    #[test]
    fn dataset_array_try_from_variable_step_non_null_struct_and_expand() {
        let array = make_variable_trace_struct(vec![0.0, 2.0, 4.0], vec![1.0, 3.0, 5.0], false);
        let parsed = DatasetArray::try_from(array).expect("parse variable-step trace");
        assert_eq!(
            parsed.data_type(),
            DatasetDataType::Trace(TraceKind::VariableStep, ScalarKind::Numeric)
        );

        let (x, y) = parsed
            .expand_trace(0)
            .expect("expand trace")
            .expect("row exists");
        assert_eq!(x, vec![0.0, 2.0, 4.0]);

        let y_parsed = DatasetArray::try_from(y).expect("parse y values");
        assert_eq!(
            y_parsed
                .as_numeric()
                .expect("numeric y values")
                .values()
                .to_vec(),
            vec![1.0, 3.0, 5.0]
        );
    }

    #[test]
    fn dataset_array_try_from_fixed_step_non_null_struct_and_expand() {
        let array = make_fixed_trace_struct(1.0, 0.5, vec![10.0, 20.0, 30.0], false);
        let parsed = DatasetArray::try_from(array).expect("parse fixed-step trace");
        assert_eq!(
            parsed.data_type(),
            DatasetDataType::Trace(TraceKind::FixedStep, ScalarKind::Numeric)
        );

        let (x, y) = parsed
            .expand_trace(0)
            .expect("expand trace")
            .expect("row exists");
        assert_eq!(x, vec![1.0, 1.5, 2.0]);

        let y_parsed = DatasetArray::try_from(y).expect("parse y values");
        assert_eq!(
            y_parsed
                .as_numeric()
                .expect("numeric y values")
                .values()
                .to_vec(),
            vec![10.0, 20.0, 30.0]
        );
    }
}

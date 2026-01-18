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
    pub fn real(&self) -> Arc<Float64Array> {
        let array = self.0.column(0).as_ref().as_primitive::<Float64Type>();
        Arc::new(array.clone())
    }

    pub fn imag(&self) -> Arc<Float64Array> {
        let array = self.0.column(1).as_ref().as_primitive::<Float64Type>();
        Arc::new(array.clone())
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

    pub fn x0(&self) -> Arc<Float64Array> {
        let array = self.array.column(0).as_ref().as_primitive::<Float64Type>();
        Arc::new(array.clone())
    }

    pub fn step(&self) -> Arc<Float64Array> {
        let array = self.array.column(1).as_ref().as_primitive::<Float64Type>();
        Arc::new(array.clone())
    }

    pub fn y(&self) -> ScalarListArray {
        let y: &ListArray = self.array.column(2).as_ref().as_list::<i32>();
        ScalarListArray {
            array: Arc::new(y.clone()),
            scalar_kind: self.scalar_kind,
        }
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
        let scalar_kind = array.column(2).data_type().try_into()?;
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

    pub fn x(&self) -> Arc<ListArray> {
        let array: &ListArray = self.array.column(0).as_ref().as_list::<i32>();
        Arc::new(array.clone())
    }

    pub fn y(&self) -> ScalarListArray {
        let y: &ListArray = self.array.column(1).as_ref().as_list::<i32>();
        ScalarListArray {
            array: Arc::new(y.clone()),
            scalar_kind: self.scalar_kind,
        }
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
        let scalar_kind = array.column(1).data_type().try_into()?;
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

use std::sync::Arc;

use arrow_array::Float64Array;
use derive_more::From;
use indexmap::IndexMap;
use num::complex::Complex64;

use crate::{
    DatasetSchema,
    dataset::{
        Error,
        arrays::ScalarArray,
        types::{DatasetDataType, ScalarKind, TraceKind},
    },
};

#[derive(Debug, Clone)]
pub struct FixedStepTrace {
    x0: f64,
    step: f64,
    y: ScalarArray,
}

impl FixedStepTrace {
    #[must_use]
    pub fn new(x0: f64, step: f64, y: ScalarArray) -> Self {
        Self { x0, step, y }
    }
    #[must_use]
    pub fn x0(&self) -> f64 {
        self.x0
    }
    #[must_use]
    pub fn step(&self) -> f64 {
        self.step
    }
    #[must_use]
    pub fn y(&self) -> &ScalarArray {
        &self.y
    }
    #[must_use]
    pub fn scalar_kind(&self) -> ScalarKind {
        self.y.scalar_kind()
    }
}

#[derive(Debug, Clone)]
pub struct VariableStepTrace {
    x: Arc<Float64Array>,
    y: ScalarArray,
}

impl VariableStepTrace {
    pub fn new(x: Arc<Float64Array>, y: ScalarArray) -> Result<Self, Error> {
        if x.len() == y.len() {
            Ok(Self { x, y })
        } else {
            Err(Error::TraceLengthMismatch)
        }
    }
    #[must_use]
    pub fn x(&self) -> &Arc<Float64Array> {
        &self.x
    }
    #[must_use]
    pub fn y(&self) -> &ScalarArray {
        &self.y
    }
    #[must_use]
    pub fn scalar_kind(&self) -> ScalarKind {
        self.y.scalar_kind()
    }
}

#[derive(Debug, Clone, From)]
pub enum DatasetScalar {
    Numeric(f64),
    Complex(Complex64),
    SimpleTrace(ScalarArray),
    FixedStepTrace(FixedStepTrace),
    VariableStepTrace(VariableStepTrace),
}

impl DatasetScalar {
    #[must_use]
    pub fn data_type(&self) -> DatasetDataType {
        match self {
            DatasetScalar::Numeric(_) => DatasetDataType::Scalar(ScalarKind::Numeric),
            DatasetScalar::Complex(_) => DatasetDataType::Scalar(ScalarKind::Complex),
            DatasetScalar::SimpleTrace(t) => {
                DatasetDataType::Trace(TraceKind::Simple, t.scalar_kind())
            }
            DatasetScalar::FixedStepTrace(t) => {
                DatasetDataType::Trace(TraceKind::FixedStep, t.scalar_kind())
            }
            DatasetScalar::VariableStepTrace(t) => {
                DatasetDataType::Trace(TraceKind::VariableStep, t.scalar_kind())
            }
        }
    }
}

pub struct DatasetRow(pub IndexMap<String, DatasetScalar>);

impl DatasetRow {
    #[must_use]
    pub fn to_schema(&self) -> DatasetSchema {
        let columns = self
            .0
            .iter()
            .map(|(name, scalar)| (name.clone(), scalar.data_type()))
            .collect();
        DatasetSchema::new(columns)
    }
}

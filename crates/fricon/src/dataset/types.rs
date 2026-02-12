use std::{
    fmt,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use arrow_schema::{
    ArrowError, DataType, Field, FieldRef, Fields, Schema, extension::ExtensionType,
};
use indexmap::IndexMap;
use itertools::Itertools;

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Numeric,
    Complex,
}

impl ScalarKind {
    fn to_data_type(self) -> DataType {
        match self {
            ScalarKind::Numeric => DataType::Float64,
            ScalarKind::Complex => ComplexType::data_type(),
        }
    }

    pub fn to_field(self, name: impl Into<String>, nullable: bool) -> Field {
        match self {
            ScalarKind::Numeric => Field::new(name, self.to_data_type(), nullable),
            ScalarKind::Complex => ComplexType::field(name, nullable),
        }
    }

    #[must_use]
    pub fn to_item_field(self) -> Field {
        self.to_field("item", false)
    }
}

impl From<ScalarKind> for DataType {
    fn from(value: ScalarKind) -> Self {
        value.to_data_type()
    }
}

impl TryFrom<&DataType> for ScalarKind {
    type Error = Error;

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        if value.is_numeric() {
            Ok(ScalarKind::Numeric)
        } else if *value == ComplexType::data_type() {
            Ok(ScalarKind::Complex)
        } else {
            Err(Error::IncompatibleType)
        }
    }
}

pub struct ComplexType;

impl ComplexType {
    pub fn fields() -> Fields {
        static FIELDS: LazyLock<Fields> = LazyLock::new(|| {
            vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ]
            .into()
        });
        FIELDS.clone()
    }

    pub fn data_type() -> DataType {
        DataType::Struct(Self::fields())
    }

    pub fn field(name: impl Into<String>, nullable: bool) -> Field {
        Field::new(name, Self::data_type(), nullable).with_extension_type(Self)
    }
}

impl ExtensionType for ComplexType {
    const NAME: &'static str = "fricon.complex";
    type Metadata = ();
    fn metadata(&self) -> &Self::Metadata {
        &()
    }
    fn serialize_metadata(&self) -> Option<String> {
        None
    }
    fn deserialize_metadata(_metadata: Option<&str>) -> Result<Self::Metadata, ArrowError> {
        Ok(())
    }
    fn supports_data_type(&self, data_type: &DataType) -> Result<(), ArrowError> {
        if *data_type == ComplexType::data_type() {
            Ok(())
        } else {
            Err(ArrowError::InvalidArgumentError(format!(
                "Expected struct<real: non-null Float64, imag: non-null Float64>, found \
                 {data_type}"
            )))
        }
    }
    fn try_new(data_type: &DataType, _metadata: Self::Metadata) -> Result<Self, ArrowError> {
        let res = ComplexType;
        res.supports_data_type(data_type)?;
        Ok(res)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceKind {
    Simple,
    FixedStep,
    VariableStep,
}

impl TraceKind {
    #[must_use]
    pub fn to_data_type(self, item: FieldRef) -> DataType {
        match self {
            TraceKind::Simple => DataType::List(item),
            TraceKind::FixedStep => DataType::Struct(
                vec![
                    Field::new("x0", DataType::Float64, false),
                    Field::new("step", DataType::Float64, false),
                    Field::new("y", DataType::List(item), false),
                ]
                .into(),
            ),
            TraceKind::VariableStep => DataType::Struct(
                vec![
                    Field::new("x", DataType::new_list(DataType::Float64, false), false),
                    Field::new("y", DataType::List(item), false),
                ]
                .into(),
            ),
        }
    }

    pub fn to_field(self, name: impl Into<String>, item: FieldRef, nullable: bool) -> Field {
        Field::new(name, self.to_data_type(item), nullable).with_extension_type(self)
    }

    #[must_use]
    pub fn parse_data_type(data_type: &DataType) -> Option<(TraceKind, &Field)> {
        fn parse_fixed_step(fields: &[FieldRef]) -> Option<(TraceKind, &Field)> {
            fields
                .iter()
                .map(|f| f.name())
                .eq(["x0", "step", "y"])
                .then(|| match [0, 1, 2].map(|i| fields[i].data_type()) {
                    [DataType::Float64, DataType::Float64, DataType::List(y)] => {
                        Some((TraceKind::FixedStep, y.as_ref()))
                    }
                    _ => None,
                })
                .flatten()
        }

        fn parse_variable_step(fields: &[FieldRef]) -> Option<(TraceKind, &Field)> {
            fields
                .iter()
                .map(|f| f.name())
                .eq(["x", "y"])
                .then(|| match [0, 1].map(|i| fields[i].data_type()) {
                    [DataType::List(x), DataType::List(y)]
                        if matches!(x.data_type(), DataType::Float64) =>
                    {
                        Some((TraceKind::VariableStep, y.as_ref()))
                    }
                    _ => None,
                })
                .flatten()
        }

        match data_type {
            DataType::List(f) => Some((TraceKind::Simple, f.as_ref())),
            DataType::Struct(fs) => parse_fixed_step(fs).or_else(|| parse_variable_step(fs)),
            _ => None,
        }
    }
}

impl fmt::Display for TraceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TraceKind::Simple => "simple",
                TraceKind::FixedStep => "fixed",
                TraceKind::VariableStep => "variable",
            }
        )
    }
}

impl FromStr for TraceKind {
    type Err = ArrowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(TraceKind::Simple),
            "fixed" => Ok(TraceKind::FixedStep),
            "variable" => Ok(TraceKind::VariableStep),
            _ => Err(ArrowError::ParseError(format!("unknown trace kind {s}"))),
        }
    }
}

impl ExtensionType for TraceKind {
    const NAME: &'static str = "fricon.trace";
    type Metadata = Self;

    fn metadata(&self) -> &Self::Metadata {
        self
    }

    fn serialize_metadata(&self) -> Option<String> {
        Some(self.to_string())
    }

    fn deserialize_metadata(metadata: Option<&str>) -> Result<Self::Metadata, ArrowError> {
        metadata.map_or_else(
            || Err(ArrowError::ParseError("Metadata not found".to_string())),
            str::parse,
        )
    }

    fn supports_data_type(&self, data_type: &DataType) -> Result<(), ArrowError> {
        if TraceKind::parse_data_type(data_type).is_some_and(|(kind, _)| *self == kind) {
            Ok(())
        } else {
            Err(ArrowError::InvalidArgumentError(format!(
                "Trace {self} doesn't support data type {data_type}"
            )))
        }
    }

    fn try_new(data_type: &DataType, metadata: Self::Metadata) -> Result<Self, ArrowError> {
        metadata.supports_data_type(data_type)?;
        Ok(metadata)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetDataType {
    Scalar(ScalarKind),
    Trace(TraceKind, ScalarKind),
}

impl DatasetDataType {
    fn to_field(self, name: impl Into<String>, nullable: bool) -> Field {
        match self {
            DatasetDataType::Scalar(scalar_kind) => scalar_kind.to_field(name, nullable),
            DatasetDataType::Trace(trace_kind, scalar_kind) => {
                trace_kind.to_field(name, Arc::new(scalar_kind.to_item_field()), nullable)
            }
        }
    }

    #[must_use]
    pub fn is_complex(self) -> bool {
        matches!(
            self,
            DatasetDataType::Scalar(ScalarKind::Complex)
                | DatasetDataType::Trace(_, ScalarKind::Complex)
        )
    }
}

impl TryFrom<&DataType> for DatasetDataType {
    type Error = Error;

    fn try_from(value: &DataType) -> Result<Self, Self::Error> {
        if let Some((trace, field)) = TraceKind::parse_data_type(value) {
            Ok(DatasetDataType::Trace(trace, field.data_type().try_into()?))
        } else {
            Ok(DatasetDataType::Scalar(value.try_into()?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetSchema {
    columns: IndexMap<String, DatasetDataType>,
}

impl DatasetSchema {
    #[must_use]
    pub fn new(columns: IndexMap<String, DatasetDataType>) -> Self {
        Self { columns }
    }

    #[must_use]
    pub fn columns(&self) -> &IndexMap<String, DatasetDataType> {
        &self.columns
    }

    #[must_use]
    pub fn to_arrow_schema(&self) -> Schema {
        let fields: Vec<_> = self
            .columns
            .iter()
            .map(|(name, data_type)| Arc::new(data_type.to_field(name, false)))
            .collect();
        Schema::new(fields)
    }
}

impl TryFrom<&Schema> for DatasetSchema {
    type Error = Error;

    fn try_from(value: &Schema) -> Result<Self, Self::Error> {
        let columns = value
            .fields
            .iter()
            .map(|x| {
                Ok::<_, Error>((
                    x.name().to_owned(),
                    DatasetDataType::try_from(x.data_type())?,
                ))
            })
            .try_collect()?;
        Ok(Self { columns })
    }
}

#[cfg(test)]
mod tests {
    use arrow_schema::{DataType, Field};

    use super::TraceKind;

    #[test]
    fn trace_kind_parse_variable_step_allows_nullable_fields() {
        let data_type = DataType::Struct(
            vec![
                Field::new("x", DataType::new_list(DataType::Float64, true), true),
                Field::new("y", DataType::new_list(DataType::Float64, true), true),
            ]
            .into(),
        );
        let parsed = TraceKind::parse_data_type(&data_type).map(|(kind, _)| kind);
        assert_eq!(parsed, Some(TraceKind::VariableStep));
    }

    #[test]
    fn trace_kind_parse_fixed_step_allows_nullable_fields() {
        let data_type = DataType::Struct(
            vec![
                Field::new("x0", DataType::Float64, true),
                Field::new("step", DataType::Float64, true),
                Field::new("y", DataType::new_list(DataType::Float64, true), true),
            ]
            .into(),
        );
        let parsed = TraceKind::parse_data_type(&data_type).map(|(kind, _)| kind);
        assert_eq!(parsed, Some(TraceKind::FixedStep));
    }
}

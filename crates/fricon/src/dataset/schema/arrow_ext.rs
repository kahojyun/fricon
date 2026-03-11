use std::sync::LazyLock;

use arrow_schema::{ArrowError, DataType, Field, Fields, extension::ExtensionType};

pub(crate) struct ComplexType;

impl ComplexType {
    pub(crate) fn fields() -> Fields {
        static FIELDS: LazyLock<Fields> = LazyLock::new(|| {
            vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ]
            .into()
        });
        FIELDS.clone()
    }

    pub(crate) fn data_type() -> DataType {
        DataType::Struct(Self::fields())
    }

    pub(crate) fn field(name: impl Into<String>, nullable: bool) -> Field {
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

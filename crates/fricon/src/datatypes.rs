use arrow::datatypes::{DataType, Field};
use arrow_schema::{FieldRef, extension::ExtensionType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ComplexType;

impl ExtensionType for ComplexType {
    const NAME: &'static str = "fricon.complex";
    type Metadata = ();

    fn metadata(&self) -> &Self::Metadata {
        &()
    }

    fn serialize_metadata(&self) -> Option<String> {
        None
    }

    fn deserialize_metadata(_metadata: Option<&str>) -> Result<(), arrow_schema::ArrowError> {
        Ok(())
    }

    fn supports_data_type(&self, data_type: &DataType) -> Result<(), arrow_schema::ArrowError> {
        match data_type {
            DataType::Struct(fields) if fields.len() == 2 => {
                let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                if field_names == ["real", "imag"]
                    && fields[0].data_type() == &DataType::Float64
                    && fields[1].data_type() == &DataType::Float64
                {
                    Ok(())
                } else {
                    Err(arrow_schema::ArrowError::InvalidArgumentError(
                        "ComplexType requires struct with 'real' and 'imag' fields of type Float64"
                            .to_string(),
                    ))
                }
            }
            _ => Err(arrow_schema::ArrowError::InvalidArgumentError(
                "ComplexType requires Struct data type".to_string(),
            )),
        }
    }

    fn try_new(
        data_type: &DataType,
        _metadata: Self::Metadata,
    ) -> Result<Self, arrow_schema::ArrowError> {
        let temp_self = Self;
        temp_self.supports_data_type(data_type)?;
        Ok(temp_self)
    }
}

impl ComplexType {
    #[must_use]
    pub fn storage_type() -> DataType {
        DataType::Struct(
            vec![
                Field::new("real", DataType::Float64, false),
                Field::new("imag", DataType::Float64, false),
            ]
            .into(),
        )
    }

    #[must_use]
    pub fn field(name: &str, nullable: bool) -> Field {
        Field::new(name, Self::storage_type(), nullable).with_extension_type(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceType {
    SimpleList,
    FixedStep,
    VariableStep,
}

impl ExtensionType for TraceType {
    const NAME: &'static str = "fricon.trace";
    type Metadata = TraceType;

    fn metadata(&self) -> &Self::Metadata {
        self
    }

    fn serialize_metadata(&self) -> Option<String> {
        Some(serde_json::to_string(self).unwrap_or_else(|_| format!("{self:?}")))
    }

    fn deserialize_metadata(
        metadata: Option<&str>,
    ) -> Result<Self::Metadata, arrow_schema::ArrowError> {
        let metadata = metadata.ok_or_else(|| {
            arrow_schema::ArrowError::InvalidArgumentError(
                "TraceType metadata required".to_string(),
            )
        })?;

        serde_json::from_str(metadata).map_err(|e| {
            arrow_schema::ArrowError::InvalidArgumentError(format!(
                "Failed to deserialize TraceType: {e}"
            ))
        })
    }

    fn supports_data_type(&self, data_type: &DataType) -> Result<(), arrow_schema::ArrowError> {
        if let Some((trace_type, _item_field)) = data_type.parse_trace_datatype()
            && trace_type == *self
        {
            return Ok(());
        }
        Err(arrow_schema::ArrowError::InvalidArgumentError(format!(
            "TraceType::{self:?} does not match provided data type"
        )))
    }

    fn try_new(
        data_type: &DataType,
        trace_type: Self::Metadata,
    ) -> Result<Self, arrow_schema::ArrowError> {
        let temp_self = trace_type;
        temp_self.supports_data_type(data_type)?;
        Ok(temp_self)
    }
}

impl TraceType {
    #[must_use]
    pub fn simple_list() -> Self {
        TraceType::SimpleList
    }

    #[must_use]
    pub fn fixed_step() -> Self {
        TraceType::FixedStep
    }

    #[must_use]
    pub fn variable_step() -> Self {
        TraceType::VariableStep
    }

    #[must_use]
    pub fn storage_type(&self, y_item_field: Arc<Field>) -> DataType {
        match self {
            TraceType::SimpleList => DataType::List(y_item_field),
            TraceType::FixedStep => DataType::Struct(
                vec![
                    Field::new("x0", DataType::Float64, false),
                    Field::new("step", DataType::Float64, false),
                    Field::new("y", DataType::List(y_item_field), false),
                ]
                .into(),
            ),
            TraceType::VariableStep => DataType::Struct(
                vec![
                    Field::new("x", DataType::new_list(DataType::Float64, false), false),
                    Field::new("y", DataType::List(y_item_field), false),
                ]
                .into(),
            ),
        }
    }

    #[must_use]
    pub fn field(&self, name: &str, y_item_field: Arc<Field>, nullable: bool) -> Field {
        let storage_type = self.storage_type(y_item_field);
        let mut field = Field::new(name, storage_type, nullable);
        let _ = field.try_with_extension_type(*self);
        field
    }

    #[must_use]
    pub fn simple_list_field(name: &str, y_item_field: Arc<Field>, nullable: bool) -> Field {
        Self::simple_list().field(name, y_item_field, nullable)
    }

    #[must_use]
    pub fn fixed_step_field(name: &str, y_item_field: Arc<Field>, nullable: bool) -> Field {
        Self::fixed_step().field(name, y_item_field, nullable)
    }

    #[must_use]
    pub fn variable_step_field(name: &str, y_item_field: Arc<Field>, nullable: bool) -> Field {
        Self::variable_step().field(name, y_item_field, nullable)
    }
}

impl std::fmt::Display for TraceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceType::SimpleList => write!(f, "simple_list"),
            TraceType::FixedStep => write!(f, "fixed_step"),
            TraceType::VariableStep => write!(f, "variable_step"),
        }
    }
}

pub trait FriconTypeExt {
    fn is_complex(&self) -> bool;
    fn is_trace(&self) -> bool;
    fn parse_trace_datatype(&self) -> Option<(TraceType, &FieldRef)>;
}

impl FriconTypeExt for DataType {
    fn is_complex(&self) -> bool {
        ComplexType.supports_data_type(self).is_ok()
    }

    fn is_trace(&self) -> bool {
        self.parse_trace_datatype().is_some()
    }

    fn parse_trace_datatype(&self) -> Option<(TraceType, &FieldRef)> {
        match self {
            DataType::List(item_field) => {
                // SimpleList: List<item>
                Some((TraceType::SimpleList, item_field))
            }
            DataType::Struct(fields) => {
                // FixedStep: Struct{x0, step, y: List<item>}
                if fields.len() == 3 {
                    let names: Vec<_> = fields.iter().map(|f| f.name().as_str()).collect();
                    if names == ["x0", "step", "y"]
                        && fields[0].data_type() == &DataType::Float64
                        && fields[1].data_type() == &DataType::Float64
                        && matches!(fields[2].data_type(), DataType::List(_))
                        && let DataType::List(item_field) = fields[2].data_type()
                    {
                        return Some((TraceType::FixedStep, item_field));
                    }
                }
                // VariableStep: Struct{x: List<f64>, y: List<item>}
                if fields.len() == 2 {
                    let names: Vec<_> = fields.iter().map(|f| f.name().as_str()).collect();
                    if names == ["x", "y"]
                        && matches!(fields[1].data_type(), DataType::List(_))
                        && let DataType::List(item_field) = fields[1].data_type()
                    {
                        return Some((TraceType::VariableStep, item_field));
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl FriconTypeExt for Field {
    fn is_complex(&self) -> bool {
        self.try_extension_type::<ComplexType>().is_ok()
    }

    fn is_trace(&self) -> bool {
        self.try_extension_type::<TraceType>().is_ok()
    }

    fn parse_trace_datatype(&self) -> Option<(TraceType, &FieldRef)> {
        self.data_type().parse_trace_datatype()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_complex_type() {
        let data_type = ComplexType::storage_type();

        assert!(data_type.is_complex());
        assert!(!data_type.is_trace());

        let field = ComplexType::field("complex_field", false);
        assert_eq!(field.name(), "complex_field");
        assert_eq!(field.data_type(), &data_type);
        assert!(field.is_complex());
        assert!(!field.is_trace());

        // Check extension metadata
        assert_eq!(field.extension_type_name(), Some("fricon.complex"));

        assert_eq!(ComplexType::NAME, "fricon.complex");
    }

    #[test]
    fn test_trace_type_field_creation() {
        // Test direct field creation from TraceType
        let item_field = Arc::new(Field::new("item", DataType::Float64, false));
        let simple_field = TraceType::SimpleList.field("simple", item_field.clone(), false);
        assert!(simple_field.is_trace());
        assert_eq!(
            simple_field.parse_trace_datatype(),
            Some((TraceType::SimpleList, &item_field))
        );

        let fixed_field = TraceType::FixedStep.field("fixed", item_field.clone(), false);
        assert!(fixed_field.is_trace());
        assert_eq!(
            fixed_field.parse_trace_datatype(),
            Some((TraceType::FixedStep, &item_field))
        );

        let variable_field = TraceType::VariableStep.field("variable", item_field.clone(), false);
        assert!(variable_field.is_trace());
        assert_eq!(
            variable_field.parse_trace_datatype(),
            Some((TraceType::VariableStep, &item_field))
        );
    }

    #[test]
    fn test_trace_type_display() {
        assert_eq!(TraceType::SimpleList.to_string(), "simple_list");
        assert_eq!(TraceType::FixedStep.to_string(), "fixed_step");
        assert_eq!(TraceType::VariableStep.to_string(), "variable_step");
    }

    #[test]
    fn test_trace_with_custom_y_datatype() {
        // Test creating traces with different y data types
        let simple_int = TraceType::simple_list();
        let int_item_field = Arc::new(Field::new("item", DataType::Int32, false));
        let simple_int_type = simple_int.storage_type(int_item_field.clone());
        assert!(simple_int_type.is_trace());
        assert_eq!(
            simple_int_type.parse_trace_datatype(),
            Some((TraceType::SimpleList, &int_item_field))
        );

        let simple_string = TraceType::simple_list();
        let string_item_field = Arc::new(Field::new("item", DataType::Utf8, false));
        let simple_string_type = simple_string.storage_type(string_item_field.clone());
        assert!(simple_string_type.is_trace());
        assert_eq!(
            simple_string_type.parse_trace_datatype(),
            Some((TraceType::SimpleList, &string_item_field))
        );

        let simple_bool = TraceType::simple_list();
        let bool_item_field = Arc::new(Field::new("item", DataType::Boolean, false));
        let simple_bool_type = simple_bool.storage_type(bool_item_field.clone());
        assert!(simple_bool_type.is_trace());
        assert_eq!(
            simple_bool_type.parse_trace_datatype(),
            Some((TraceType::SimpleList, &bool_item_field))
        );

        // Test fixed step with different y data types
        let fixed_int = TraceType::fixed_step();
        let fixed_int_type = fixed_int.storage_type(int_item_field.clone());
        assert!(fixed_int_type.is_trace());
        assert_eq!(
            fixed_int_type.parse_trace_datatype(),
            Some((TraceType::FixedStep, &int_item_field))
        );

        // Test variable step with different y data types
        let variable_string = TraceType::variable_step();
        let variable_string_type = variable_string.storage_type(string_item_field.clone());
        assert!(variable_string_type.is_trace());
        assert_eq!(
            variable_string_type.parse_trace_datatype(),
            Some((TraceType::VariableStep, &string_item_field))
        );
    }

    #[test]
    fn test_trace_type_serde_metadata() {
        // Test serde serialization and deserialization of metadata
        let trace_type = TraceType::simple_list();
        let serialized = trace_type.serialize_metadata().unwrap();

        // Should be valid JSON
        let deserialized: TraceType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, TraceType::SimpleList);

        // Test ExtensionType deserialization
        let from_metadata = TraceType::deserialize_metadata(Some(&serialized)).unwrap();
        assert_eq!(from_metadata, TraceType::SimpleList);

        // Test all variants
        let variants = vec![
            (TraceType::simple_list(), TraceType::SimpleList),
            (TraceType::fixed_step(), TraceType::FixedStep),
            (TraceType::variable_step(), TraceType::VariableStep),
        ];

        for (trace_type, expected_variant) in variants {
            let serialized = trace_type.serialize_metadata().unwrap();
            let deserialized = TraceType::deserialize_metadata(Some(&serialized)).unwrap();
            assert_eq!(deserialized, expected_variant);
        }
    }

    #[test]
    fn test_parse_trace_datatype_trait_method() {
        // Simple list
        let item_field = Arc::new(Field::new("item", DataType::Float64, false));
        let simple_type = TraceType::SimpleList.storage_type(item_field.clone());
        let parsed = simple_type.parse_trace_datatype();
        assert!(matches!(parsed, Some((TraceType::SimpleList, _))));

        // Fixed step
        let fixed_type = TraceType::FixedStep.storage_type(item_field.clone());
        let parsed_fixed = fixed_type.parse_trace_datatype();
        assert!(matches!(parsed_fixed, Some((TraceType::FixedStep, _))));

        // Variable step
        let variable_type = TraceType::VariableStep.storage_type(item_field.clone());
        let parsed_variable = variable_type.parse_trace_datatype();
        assert!(matches!(
            parsed_variable,
            Some((TraceType::VariableStep, _))
        ));

        // Field variants
        let simple_field = TraceType::SimpleList.field("simple", item_field.clone(), false);
        assert!(matches!(
            simple_field.parse_trace_datatype(),
            Some((TraceType::SimpleList, _))
        ));

        let fixed_field = TraceType::FixedStep.field("fixed", item_field.clone(), false);
        assert!(matches!(
            fixed_field.parse_trace_datatype(),
            Some((TraceType::FixedStep, _))
        ));

        let variable_field = TraceType::VariableStep.field("variable", item_field, false);
        assert!(matches!(
            variable_field.parse_trace_datatype(),
            Some((TraceType::VariableStep, _))
        ));

        // Non-trace type returns None
        let regular_type = DataType::Int32;
        assert!(regular_type.parse_trace_datatype().is_none());
        let regular_field = Field::new("regular", DataType::Int32, false);
        assert!(regular_field.parse_trace_datatype().is_none());
    }
}

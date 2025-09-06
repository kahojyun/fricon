use arrow::datatypes::{DataType, Field};
use arrow_schema::extension::ExtensionType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ComplexType {
    // ComplexType is a unit struct since the storage type is always the same
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
        let temp_self = Self::default();
        temp_self.supports_data_type(data_type)?;
        Ok(temp_self)
    }
}

impl ComplexType {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
        Field::new(name, Self::storage_type(), nullable).with_extension_type(Self::default())
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
        match self {
            TraceType::SimpleList => {
                if let DataType::List(_) = data_type {
                    Ok(())
                } else {
                    Err(arrow_schema::ArrowError::InvalidArgumentError(
                        "SimpleList trace requires List data type".to_string(),
                    ))
                }
            }
            TraceType::FixedStep => {
                if let DataType::Struct(fields) = data_type
                    && fields.len() == 3
                {
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == ["x0", "step", "y"]
                        && fields[0].data_type() == &DataType::Float64
                        && fields[1].data_type() == &DataType::Float64
                        && matches!(fields[2].data_type(), DataType::List(_))
                    {
                        return Ok(());
                    }
                }
                Err(arrow_schema::ArrowError::InvalidArgumentError(
                    "FixedStep trace requires struct with x0, step, y fields".to_string(),
                ))
            }
            TraceType::VariableStep => {
                if let DataType::Struct(fields) = data_type
                    && fields.len() == 2
                {
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name().as_str()).collect();
                    if field_names == ["x", "y"]
                        && matches!(fields[0].data_type(), DataType::List(_))
                        && matches!(fields[1].data_type(), DataType::List(_))
                    {
                        return Ok(());
                    }
                }
                Err(arrow_schema::ArrowError::InvalidArgumentError(
                    "VariableStep trace requires struct with x, y fields".to_string(),
                ))
            }
        }
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
    pub fn storage_type(&self, y_item_type: DataType) -> DataType {
        match self {
            TraceType::SimpleList => DataType::new_list(y_item_type, false),
            TraceType::FixedStep => DataType::Struct(
                vec![
                    Field::new("x0", DataType::Float64, false),
                    Field::new("step", DataType::Float64, false),
                    Field::new("y", DataType::new_list(y_item_type, false), false),
                ]
                .into(),
            ),
            TraceType::VariableStep => DataType::Struct(
                vec![
                    Field::new("x", DataType::new_list(DataType::Float64, false), false),
                    Field::new("y", DataType::new_list(y_item_type, false), false),
                ]
                .into(),
            ),
        }
    }

    #[must_use]
    pub fn field(&self, name: &str, y_item_type: DataType, nullable: bool) -> Field {
        let storage_type = self.storage_type(y_item_type);
        let mut field = Field::new(name, storage_type, nullable);
        let _ = field.try_with_extension_type(*self);
        field
    }

    #[must_use]
    pub fn simple_list_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        Self::simple_list().field(name, y_item_type, nullable)
    }

    #[must_use]
    pub fn fixed_step_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        Self::fixed_step().field(name, y_item_type, nullable)
    }

    #[must_use]
    pub fn variable_step_field(name: &str, y_item_type: DataType, nullable: bool) -> Field {
        Self::variable_step().field(name, y_item_type, nullable)
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
    fn trace_type(&self) -> Option<TraceType>;
}

impl FriconTypeExt for DataType {
    fn is_complex(&self) -> bool {
        ComplexType::default().supports_data_type(self).is_ok()
    }

    fn is_trace(&self) -> bool {
        self.trace_type().is_some()
    }

    fn trace_type(&self) -> Option<TraceType> {
        [
            TraceType::SimpleList,
            TraceType::FixedStep,
            TraceType::VariableStep,
        ]
        .into_iter()
        .find(|&trace_type| trace_type.supports_data_type(self).is_ok())
    }
}

impl FriconTypeExt for Field {
    fn is_complex(&self) -> bool {
        self.try_extension_type::<ComplexType>().is_ok()
    }

    fn is_trace(&self) -> bool {
        self.try_extension_type::<TraceType>().is_ok()
    }

    fn trace_type(&self) -> Option<TraceType> {
        if let Ok(trace_type) = self.try_extension_type::<TraceType>() {
            Some(trace_type)
        } else {
            self.data_type().trace_type()
        }
    }
}

// helper removed — construct Schema::new(...) directly where needed

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::Schema;

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

        // Test new() method and extension name
        let _complex_type = ComplexType::new();
        assert_eq!(ComplexType::NAME, "fricon.complex");
    }

    #[test]
    fn test_trace_types() {
        // Test simple list trace
        let simple_trace = TraceType::simple_list();
        let simple_data_type = simple_trace.storage_type(DataType::Float64);
        assert!(simple_data_type.is_trace());
        assert_eq!(simple_data_type.trace_type(), Some(TraceType::SimpleList));

        // Test fixed step trace
        let fixed_trace = TraceType::fixed_step();
        let fixed_data_type = fixed_trace.storage_type(DataType::Float64);
        assert!(fixed_data_type.is_trace());
        assert_eq!(fixed_data_type.trace_type(), Some(TraceType::FixedStep));

        // Test variable step trace
        let variable_trace = TraceType::variable_step();
        let variable_data_type = variable_trace.storage_type(DataType::Float64);
        assert!(variable_data_type.is_trace());
        assert_eq!(
            variable_data_type.trace_type(),
            Some(TraceType::VariableStep)
        );

        // Test extension metadata
        let simple_field = TraceType::simple_list().field("simple", DataType::Float64, false);
        assert_eq!(simple_field.extension_type_name(), Some("fricon.trace"));
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_type(), Some(TraceType::SimpleList));

        let fixed_field = TraceType::fixed_step().field("fixed", DataType::Float64, false);
        assert_eq!(fixed_field.extension_type_name(), Some("fricon.trace"));
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_type(), Some(TraceType::FixedStep));

        let variable_field = TraceType::variable_step().field("variable", DataType::Float64, false);
        assert_eq!(variable_field.extension_type_name(), Some("fricon.trace"));
        assert!(variable_field.is_trace());
        assert_eq!(variable_field.trace_type(), Some(TraceType::VariableStep));
    }

    #[test]
    fn test_trace_type_field_creation() {
        // Test direct field creation from TraceType
        let simple_field = TraceType::SimpleList.field("simple", DataType::Float64, false);
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_type(), Some(TraceType::SimpleList));

        let fixed_field = TraceType::FixedStep.field("fixed", DataType::Float64, false);
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_type(), Some(TraceType::FixedStep));

        let variable_field = TraceType::VariableStep.field("variable", DataType::Float64, false);
        assert!(variable_field.is_trace());
        assert_eq!(variable_field.trace_type(), Some(TraceType::VariableStep));
    }

    #[test]
    fn test_trace_type_convenience_methods() {
        // Test convenience methods
        let simple_field = TraceType::simple_list_field("simple", DataType::Float64, false);
        assert!(simple_field.is_trace());
        assert_eq!(simple_field.trace_type(), Some(TraceType::SimpleList));

        let fixed_field = TraceType::fixed_step_field("fixed", DataType::Float64, false);
        assert!(fixed_field.is_trace());
        assert_eq!(fixed_field.trace_type(), Some(TraceType::FixedStep));

        let variable_field = TraceType::variable_step_field("variable", DataType::Float64, false);
        assert!(variable_field.is_trace());
        assert_eq!(variable_field.trace_type(), Some(TraceType::VariableStep));
    }

    #[test]
    fn test_schema_creation() {
        // Test creating schemas directly without builder
        let schema = Schema::new(vec![
            ComplexType::field("complex_data", false),
            TraceType::simple_list_field("simple_trace", DataType::Float64, true),
            TraceType::fixed_step_field("fixed_trace", DataType::Float64, false),
            TraceType::variable_step_field("variable_trace", DataType::Float64, true),
            Field::new("regular_field", DataType::Int32, false),
        ]);

        assert_eq!(schema.fields().len(), 5);

        // Check complex field
        let complex_field = schema.field_with_name("complex_data").unwrap();
        assert!(complex_field.is_complex());

        // Check trace fields
        let simple_field = schema.field_with_name("simple_trace").unwrap();
        assert_eq!(simple_field.trace_type(), Some(TraceType::SimpleList));

        let fixed_field = schema.field_with_name("fixed_trace").unwrap();
        assert_eq!(fixed_field.trace_type(), Some(TraceType::FixedStep));

        let variable_field = schema.field_with_name("variable_trace").unwrap();
        assert_eq!(variable_field.trace_type(), Some(TraceType::VariableStep));
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
        let simple_int_type = simple_int.storage_type(DataType::Int32);
        assert!(simple_int_type.is_trace());
        assert_eq!(simple_int_type.trace_type(), Some(TraceType::SimpleList));

        let simple_string = TraceType::simple_list();
        let simple_string_type = simple_string.storage_type(DataType::Utf8);
        assert!(simple_string_type.is_trace());
        assert_eq!(simple_string_type.trace_type(), Some(TraceType::SimpleList));

        let simple_bool = TraceType::simple_list();
        let simple_bool_type = simple_bool.storage_type(DataType::Boolean);
        assert!(simple_bool_type.is_trace());
        assert_eq!(simple_bool_type.trace_type(), Some(TraceType::SimpleList));

        // Test fixed step with different y data types
        let fixed_int = TraceType::fixed_step();
        let fixed_int_type = fixed_int.storage_type(DataType::Int32);
        assert!(fixed_int_type.is_trace());
        assert_eq!(fixed_int_type.trace_type(), Some(TraceType::FixedStep));

        // Test variable step with different y data types
        let variable_string = TraceType::variable_step();
        let variable_string_type = variable_string.storage_type(DataType::Utf8);
        assert!(variable_string_type.is_trace());
        assert_eq!(
            variable_string_type.trace_type(),
            Some(TraceType::VariableStep)
        );

        // Test field creation with custom y data types
        let int_field = TraceType::simple_list_field("int_trace", DataType::Int32, false);
        assert!(int_field.is_trace());
        assert_eq!(int_field.trace_type(), Some(TraceType::SimpleList));

        let string_field = TraceType::FixedStep.field("string_trace", DataType::Utf8, false);
        assert!(string_field.is_trace());
        assert_eq!(string_field.trace_type(), Some(TraceType::FixedStep));
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
}

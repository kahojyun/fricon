# Arrow Schema Utilities Design

## Overview

This design proposes the implementation of a centralized Arrow schema utility module in the `fricon` crate to consolidate dataset schema operations. Currently, custom Arrow datatypes (complex128, trace) are scattered across fricon-py, and schema inspection functionality is duplicated in fricon-ui's charting module. The goal is to create a unified schema utilities module that serves both writing/reading operations and visualization needs.

## Technology Stack & Dependencies

- **Core**: Rust with Apache Arrow
- **Schema Operations**: arrow-rs datatypes and schema APIs
- **Serialization**: serde for JSON schema export
- **Integration**: fricon-py (Python bindings), fricon-ui (Tauri desktop app)

## Architecture

### Schema Utilities Module Structure

```mermaid
classDiagram
    class SchemaUtils {
        +inspect_dataset_schema(path) : DatasetSchemaInfo
        +get_column_info(field) : ColumnInfo
        +classify_data_type(arrow_type) : ColumnDataType
        +extract_unique_values(path, column) : Vec<ColumnValue>
        +is_visualization_supported(data_type) : bool
        +get_schema_summary(schema) : SchemaSummary
    }

    class CustomDataTypes {
        +complex128() : DataType
        +trace_variable_step(item_type) : DataType
        +trace_fixed_step(item_type) : DataType
        +is_complex_type(data_type) : bool
        +is_trace_type(data_type) : bool
        +get_trace_y_type(trace_type) : DataType
    }

    class DatasetSchemaInfo {
        +columns : Vec<ColumnInfo>
        +shape : DatasetShape
        +index_columns : Vec<String>
        +custom_types : HashMap<String, CustomTypeInfo>
        +visualization_compatibility : VisualizationCompatibility
    }

    class ColumnInfo {
        +name : String
        +data_type : ColumnDataType
        +arrow_type : DataType
        +is_index_column : bool
        +nullable : bool
        +unique_value_count : Option<usize>
        +sample_values : Option<Vec<ColumnValue>>
    }

    class DatasetShape {
        +row_count : Option<usize>
        +column_count : usize
        +memory_size_bytes : Option<usize>
    }

    class VisualizationCompatibility {
        +plottable_columns : Vec<String>
        +numeric_columns : Vec<String>
        +categorical_columns : Vec<String>
        +unsupported_columns : Vec<UnsupportedColumn>
    }

    SchemaUtils --> DatasetSchemaInfo
    SchemaUtils --> CustomDataTypes
    DatasetSchemaInfo --> ColumnInfo
    DatasetSchemaInfo --> DatasetShape
    DatasetSchemaInfo --> VisualizationCompatibility
```

### Integration with Existing Components

```mermaid
sequenceDiagram
    participant PyAPI as Python API (fricon-py)
    participant Core as Fricon Core
    participant SchemaUtils as Schema Utils
    participant ChartAPI as Chart API (fricon-ui)
    participant ArrowFile as Arrow Files

    Note over PyAPI,ArrowFile: Dataset Creation Flow
    PyAPI->>Core: create_dataset(schema_info)
    Core->>SchemaUtils: validate_custom_types(schema)
    SchemaUtils-->>Core: ValidationResult
    Core->>ArrowFile: write dataset with schema

    Note over PyAPI,ArrowFile: Schema Inspection Flow
    ChartAPI->>Core: inspect_dataset(dataset_id)
    Core->>SchemaUtils: inspect_dataset_schema(path)
    SchemaUtils->>ArrowFile: read schema + sample data
    SchemaUtils-->>Core: DatasetSchemaInfo
    Core-->>ChartAPI: schema info + compatibility

    Note over PyAPI,ArrowFile: Python Data Type Support
    PyAPI->>SchemaUtils: complex128()
    SchemaUtils-->>PyAPI: Arrow DataType
    PyAPI->>SchemaUtils: trace_variable_step(item_type)
    SchemaUtils-->>PyAPI: Arrow DataType
```

## Component Architecture

### Core Schema Utilities Module

#### Schema Inspection Engine
Provides comprehensive dataset schema analysis:

- **Arrow Schema Reading**: Direct reading from IPC files
- **Type Classification**: Categorizes Arrow types for visualization compatibility
- **Shape Analysis**: Extracts dataset dimensions and memory footprint
- **Sample Data Extraction**: Retrieves representative values for UI display

#### Custom Data Types Registry
Centralizes all custom Arrow data type definitions:

- **Complex Number Type**: Struct with real/imaginary Float64 fields
- **Trace Types**: Variable-step and fixed-step time series structures
- **Type Validation**: Ensures consistency across Python and Rust implementations
- **Schema Compatibility**: Verifies custom types work with Arrow format

#### Visualization Compatibility Analyzer
Determines which columns can be visualized:

- **Numeric Type Detection**: Identifies plottable numeric data
- **Categorical Analysis**: Finds columns suitable for grouping/filtering
- **Custom Type Support**: Handles complex numbers and traces in charts
- **Limitation Reporting**: Lists unsupported columns with reasons

### Data Models & Schema Representation

#### DatasetSchemaInfo
Primary schema information container:

```rust
pub struct DatasetSchemaInfo {
    pub columns: Vec<ColumnInfo>,
    pub shape: DatasetShape,
    pub index_columns: Vec<String>,
    pub custom_types: HashMap<String, CustomTypeInfo>,
    pub visualization_compatibility: VisualizationCompatibility,
}
```

#### ColumnInfo
Detailed column metadata:

```rust
pub struct ColumnInfo {
    pub name: String,
    pub data_type: ColumnDataType,
    pub arrow_type: DataType,
    pub is_index_column: bool,
    pub nullable: bool,
    pub unique_value_count: Option<usize>,
    pub sample_values: Option<Vec<ColumnValue>>,
}
```

#### Custom Type Information
Metadata for non-standard Arrow types:

```rust
pub struct CustomTypeInfo {
    pub type_name: String,
    pub underlying_structure: ArrowTypeStructure,
    pub visualization_support: CustomTypeVisualization,
    pub conversion_hints: ConversionHints,
}
```

### Schema Operations API

#### Core Functions
Essential schema inspection operations:

- `inspect_dataset_schema(path: &Path) -> Result<DatasetSchemaInfo>`
- `get_column_compatibility(column: &ColumnInfo) -> VisualizationCompatibility`
- `extract_shape_info(path: &Path) -> Result<DatasetShape>`
- `sample_column_values(path: &Path, column: &str, limit: usize) -> Result<Vec<ColumnValue>>`

#### Custom Type Operations
Centralized custom type management:

- `complex128() -> DataType`
- `trace_variable_step(item_type: DataType) -> DataType`
- `trace_fixed_step(item_type: DataType) -> DataType`
- `is_custom_type(data_type: &DataType) -> bool`
- `parse_custom_type(data_type: &DataType) -> Option<CustomTypeInfo>`

#### Validation & Compatibility
Schema validation and compatibility checking:

- `validate_schema_for_writing(schema: &Schema) -> Result<ValidationResult>`
- `check_visualization_compatibility(schema: &Schema) -> VisualizationCompatibility`
- `get_supported_chart_types(column: &ColumnInfo) -> Vec<ChartType>`

## Integration Strategy

### fricon-py Integration
Migrate custom type definitions from Python bindings:

- **Type Export**: Export complex128() and trace_() functions from schema_utils
- **Validation Integration**: Use centralized validation in DatasetWriter
- **Schema Inference**: Leverage unified type classification
- **Error Consistency**: Standardize schema-related error messages

### fricon-ui Integration
Replace duplicate chart schema logic:

- **Schema Reader**: Use DatasetSchemaInfo instead of custom ChartSchemaResponse
- **Type Classification**: Utilize centralized ColumnDataType classification
- **Compatibility Checking**: Leverage visualization_compatibility analysis
- **Performance**: Cache schema info to avoid repeated file reads

### API Consistency
Ensure consistent schema handling across all interfaces:

- **Unified Types**: Same DataType definitions across Python/Rust/UI
- **Standard Errors**: Consistent error types for schema validation failures
- **Documentation**: Centralized documentation for custom types and limitations

## Testing Strategy

### Unit Testing Framework
Comprehensive test coverage for schema utilities:

- **Custom Type Tests**: Validate complex128 and trace type generation
- **Schema Reading Tests**: Test Arrow file schema extraction
- **Compatibility Tests**: Verify visualization compatibility analysis
- **Edge Case Handling**: Test malformed schemas, empty datasets, large files

### Integration Testing
End-to-end testing across components:

- **Python API Tests**: Validate fricon-py custom type integration
- **UI Integration Tests**: Test chart module schema consumption
- **Performance Tests**: Benchmark schema reading on large datasets
- **Compatibility Tests**: Ensure backward compatibility with existing datasets

### Test Data Scenarios
Representative test datasets covering various schema patterns:

- **Basic Types**: Primitives, strings, lists
- **Custom Types**: Complex numbers, traces (both variants)
- **Mixed Schemas**: Combination of standard and custom types
- **Large Datasets**: Performance testing with substantial data volumes
- **Edge Cases**: Empty datasets, single-column datasets, nested structures

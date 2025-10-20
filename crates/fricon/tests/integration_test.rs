#![allow(clippy::pedantic, clippy::restriction)]
use std::sync::Arc;

use arrow_array::{Array, Float64Array, RecordBatch};
use fricon::{
    AppManager, Client, DatasetId, DatasetRow, DatasetScalar, FixedStepTrace, ScalarArray,
    VariableStepTrace, WorkspaceRoot,
};
use indexmap::IndexMap;
use num::complex::Complex64;
use tempfile::TempDir;

fn create_test_rows() -> Vec<DatasetRow> {
    vec![
        DatasetRow({
            let mut row = IndexMap::new();
            row.insert("id".to_string(), DatasetScalar::Numeric(1.0));
            row.insert("numeric_value".to_string(), DatasetScalar::Numeric(100.0));
            row.insert(
                "complex_value".to_string(),
                DatasetScalar::Complex(Complex64::new(1.0, 0.5)),
            );

            // Create a SimpleTrace with numeric values
            let simple_trace_data = vec![10.0, 20.0, 30.0];
            let simple_scalar_array = ScalarArray::from_iter(simple_trace_data);
            row.insert(
                "simple_trace".to_string(),
                DatasetScalar::SimpleTrace(simple_scalar_array),
            );

            // Create a SimpleTrace with complex values
            let complex_trace_data = vec![
                Complex64::new(1.0, 0.1),
                Complex64::new(2.0, 0.2),
                Complex64::new(3.0, 0.3),
            ];
            let complex_scalar_array = ScalarArray::from_iter(complex_trace_data);
            row.insert(
                "complex_trace".to_string(),
                DatasetScalar::SimpleTrace(complex_scalar_array),
            );

            // Create a FixedStepTrace
            let fixed_trace_data = vec![15.0, 25.0, 35.0];
            let fixed_scalar_array = ScalarArray::from_iter(fixed_trace_data);
            let fixed_trace = FixedStepTrace::new(0.0, 5.0, fixed_scalar_array);
            row.insert(
                "fixed_trace".to_string(),
                DatasetScalar::FixedStepTrace(fixed_trace),
            );

            // Create a VariableStepTrace
            let x_values = Arc::new(Float64Array::from(vec![0.0, 2.5, 7.0, 15.0]));
            let var_trace_data = vec![5.0, 10.0, 15.0, 20.0];
            let var_scalar_array = ScalarArray::from_iter(var_trace_data);
            let variable_trace = VariableStepTrace::new(x_values, var_scalar_array).unwrap();
            row.insert(
                "variable_trace".to_string(),
                DatasetScalar::VariableStepTrace(variable_trace),
            );
            row
        }),
        DatasetRow({
            let mut row = IndexMap::new();
            row.insert("id".to_string(), DatasetScalar::Numeric(2.0));
            row.insert("numeric_value".to_string(), DatasetScalar::Numeric(200.0));
            row.insert(
                "complex_value".to_string(),
                DatasetScalar::Complex(Complex64::new(2.0, 1.0)),
            );

            // Create a SimpleTrace with numeric values
            let simple_trace_data = vec![40.0, 50.0, 60.0];
            let simple_scalar_array = ScalarArray::from_iter(simple_trace_data);
            row.insert(
                "simple_trace".to_string(),
                DatasetScalar::SimpleTrace(simple_scalar_array),
            );

            // Create a SimpleTrace with complex values
            let complex_trace_data = vec![
                Complex64::new(4.0, 0.4),
                Complex64::new(5.0, 0.5),
                Complex64::new(6.0, 0.6),
            ];
            let complex_scalar_array = ScalarArray::from_iter(complex_trace_data);
            row.insert(
                "complex_trace".to_string(),
                DatasetScalar::SimpleTrace(complex_scalar_array),
            );

            // Create a FixedStepTrace
            let fixed_trace_data = vec![45.0, 55.0, 65.0];
            let fixed_scalar_array = ScalarArray::from_iter(fixed_trace_data);
            let fixed_trace = FixedStepTrace::new(10.0, 3.0, fixed_scalar_array);
            row.insert(
                "fixed_trace".to_string(),
                DatasetScalar::FixedStepTrace(fixed_trace),
            );

            // Create a VariableStepTrace
            let x_values = Arc::new(Float64Array::from(vec![1.0, 3.5, 8.0, 16.0]));
            let var_trace_data = vec![25.0, 30.0, 35.0, 40.0];
            let var_scalar_array = ScalarArray::from_iter(var_trace_data);
            let variable_trace = VariableStepTrace::new(x_values, var_scalar_array).unwrap();
            row.insert(
                "variable_trace".to_string(),
                DatasetScalar::VariableStepTrace(variable_trace),
            );
            row
        }),
        DatasetRow({
            let mut row = IndexMap::new();
            row.insert("id".to_string(), DatasetScalar::Numeric(3.0));
            row.insert("numeric_value".to_string(), DatasetScalar::Numeric(300.0));
            row.insert(
                "complex_value".to_string(),
                DatasetScalar::Complex(Complex64::new(3.0, 1.5)),
            );

            // Create a SimpleTrace with numeric values
            let simple_trace_data = vec![70.0, 80.0, 90.0];
            let simple_scalar_array = ScalarArray::from_iter(simple_trace_data);
            row.insert(
                "simple_trace".to_string(),
                DatasetScalar::SimpleTrace(simple_scalar_array),
            );

            // Create a SimpleTrace with complex values
            let complex_trace_data = vec![
                Complex64::new(7.0, 0.7),
                Complex64::new(8.0, 0.8),
                Complex64::new(9.0, 0.9),
            ];
            let complex_scalar_array = ScalarArray::from_iter(complex_trace_data);
            row.insert(
                "complex_trace".to_string(),
                DatasetScalar::SimpleTrace(complex_scalar_array),
            );

            // Create a FixedStepTrace
            let fixed_trace_data = vec![75.0, 85.0, 95.0];
            let fixed_scalar_array = ScalarArray::from_iter(fixed_trace_data);
            let fixed_trace = FixedStepTrace::new(20.0, 4.0, fixed_scalar_array);
            row.insert(
                "fixed_trace".to_string(),
                DatasetScalar::FixedStepTrace(fixed_trace),
            );

            // Create a VariableStepTrace
            let x_values = Arc::new(Float64Array::from(vec![2.0, 4.5, 9.0, 17.0]));
            let var_trace_data = vec![45.0, 50.0, 55.0, 60.0];
            let var_scalar_array = ScalarArray::from_iter(var_trace_data);
            let variable_trace = VariableStepTrace::new(x_values, var_scalar_array).unwrap();
            row.insert(
                "variable_trace".to_string(),
                DatasetScalar::VariableStepTrace(variable_trace),
            );
            row
        }),
    ]
}

#[tokio::test]
async fn test_dataset_create_and_load() -> anyhow::Result<()> {
    // Create a temporary directory for the workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize the workspace
    WorkspaceRoot::create_new(workspace_path)?;

    // Start the server
    let app_manager = AppManager::serve_with_path(workspace_path).await?;

    // Connect the client
    let client = Client::connect(workspace_path).await?;

    // Create test data
    let test_rows = create_test_rows();
    let test_schema = test_rows[0].to_schema();

    // Create dataset through client
    let mut writer = client.create_dataset(
        "test_dataset".to_string(),
        "Test dataset for integration test".to_string(),
        vec!["test".to_string(), "integration".to_string()],
        test_schema,
    )?;

    // Write the test rows
    for row in test_rows {
        writer.write(row).await?;
    }

    // Finish writing and get the dataset
    let dataset = writer.finish().await?;

    // Verify dataset was created
    assert_eq!(dataset.name(), "test_dataset");
    assert_eq!(dataset.tags(), &["test", "integration"]);

    // Load dataset using DatasetManager directly
    let dataset_manager = app_manager.handle().dataset_manager();
    let reader = dataset_manager
        .get_dataset_reader(DatasetId::Id(dataset.id()))
        .await?;
    let loaded_batches: Vec<RecordBatch> = reader
        .batches()
        .expect("expected completed dataset")
        .to_vec();

    // Verify loaded data matches original
    assert_eq!(loaded_batches.len(), 1);
    let loaded_batch = &loaded_batches[0];

    assert_eq!(loaded_batch.num_rows(), 3); // We wrote 3 rows
    assert_eq!(loaded_batch.num_columns(), 7); // We have 7 columns

    // Verify the schema contains all expected columns
    let schema = loaded_batch.schema();
    let field_names: Vec<&str> = schema.fields.iter().map(|f| f.name().as_str()).collect();

    // Check that we have all the expected columns
    assert!(field_names.contains(&"id"));
    assert!(field_names.contains(&"numeric_value"));
    assert!(field_names.contains(&"complex_value"));
    assert!(field_names.contains(&"simple_trace"));
    assert!(field_names.contains(&"complex_trace"));
    assert!(field_names.contains(&"fixed_trace"));
    assert!(field_names.contains(&"variable_trace"));

    // Verify id column has values 1.0, 2.0, 3.0
    if let Some(id_col) = schema.column_with_name("id") {
        let id_array = loaded_batch.column(id_col.0);
        let id_float_array = id_array.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(id_float_array.len(), 3);
        assert_eq!(id_float_array.value(0), 1.0);
        assert_eq!(id_float_array.value(1), 2.0);
        assert_eq!(id_float_array.value(2), 3.0);
    }

    // Verify numeric_value column has values 100.0, 200.0, 300.0
    if let Some(numeric_col) = schema.column_with_name("numeric_value") {
        let numeric_array = loaded_batch.column(numeric_col.0);
        let numeric_float_array = numeric_array
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(numeric_float_array.len(), 3);
        assert_eq!(numeric_float_array.value(0), 100.0);
        assert_eq!(numeric_float_array.value(1), 200.0);
        assert_eq!(numeric_float_array.value(2), 300.0);
    }

    // Shutdown the server
    app_manager.shutdown().await;
    temp_dir.close()?;

    Ok(())
}

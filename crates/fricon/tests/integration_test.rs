#![allow(clippy::pedantic, clippy::restriction)]
use std::sync::Arc;

use arrow_array::{Array, Float64Array, RecordBatch};
use fricon::{
    AppManager, Client, ClientError, DatasetId, DatasetListQuery, DatasetRow, DatasetScalar,
    DatasetStatus, ExistingUiProbeResult, FixedStepTrace, ScalarArray, VariableStepTrace,
    WorkspaceRoot, app::UiCommand,
};
use indexmap::IndexMap;
use num::complex::Complex64;
use tempfile::TempDir;
use tokio::time::{Duration, Instant};
use tonic::Code;

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

async fn wait_for_dataset_status(
    app_manager: &AppManager,
    dataset_name: &str,
    expected_status: DatasetStatus,
) -> anyhow::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let datasets = app_manager
            .handle()
            .list_datasets(DatasetListQuery {
                search: Some(dataset_name.to_string()),
                ..DatasetListQuery::default()
            })
            .await?;

        if datasets.len() == 1 && datasets[0].metadata.status == expected_status {
            return Ok(());
        }

        if Instant::now() >= deadline {
            anyhow::bail!(
                "Dataset '{dataset_name}' did not reach expected status '{expected_status:?}' in \
                 time"
            );
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn test_dataset_create_metadata_payload_finish_completes() -> anyhow::Result<()> {
    // Create a temporary directory for the workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize the workspace
    WorkspaceRoot::create_new(workspace_path)?;

    // Start the server
    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;

    // Connect the client
    let client = Client::connect(workspace_path).await?;

    // Create test data
    let test_rows = create_test_rows();
    let test_schema = test_rows[0].to_schema();

    // Create dataset through client
    let mut writer = client
        .create_dataset(
            "test_dataset".to_string(),
            "Test dataset for integration test".to_string(),
            vec!["test".to_string(), "integration".to_string()],
            test_schema.clone(),
        )
        .await?;

    // Write the test rows
    for row in test_rows {
        writer.write(row).await?;
    }

    // Finish writing and get the dataset
    let dataset = writer.finish().await?;

    // Verify dataset was created
    assert_eq!(dataset.name(), "test_dataset");
    assert_eq!(dataset.tags(), &["test", "integration"]);
    assert_eq!(dataset.status(), DatasetStatus::Completed);

    // Load dataset using DatasetManager directly
    let reader = app_manager
        .handle()
        .get_dataset_reader(DatasetId::Id(dataset.id()))
        .await?;
    let loaded_batches: Vec<RecordBatch> = reader.batches();

    // Verify loaded data matches original
    assert_eq!(reader.schema(), &test_schema);
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

#[tokio::test]
async fn test_dataset_create_abort_returns_aborted_metadata() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;
    let client = Client::connect(workspace_path).await?;

    let test_rows = create_test_rows();
    let test_schema = test_rows[0].to_schema();

    let mut writer = client
        .create_dataset(
            "aborted_dataset_by_method".to_string(),
            "This dataset is aborted explicitly".to_string(),
            vec!["test".to_string(), "abort".to_string()],
            test_schema,
        )
        .await?;

    writer.write(create_test_rows().remove(0)).await?;
    let dataset = writer.abort().await?;

    assert_eq!(dataset.name(), "aborted_dataset_by_method");
    assert_eq!(dataset.status(), DatasetStatus::Aborted);

    let datasets = app_manager
        .handle()
        .list_datasets(DatasetListQuery {
            search: Some("aborted_dataset_by_method".to_string()),
            ..DatasetListQuery::default()
        })
        .await?;
    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].metadata.status, DatasetStatus::Aborted);

    app_manager.shutdown().await;
    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn test_dataset_create_without_finish_is_aborted() -> anyhow::Result<()> {
    // Create a temporary directory for the workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize the workspace
    WorkspaceRoot::create_new(workspace_path)?;

    // Start the server
    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;

    // Connect the client
    let client = Client::connect(workspace_path).await?;

    // Create test data
    let test_rows = create_test_rows();
    let test_schema = test_rows[0].to_schema();

    // Create dataset through client
    let mut writer = client
        .create_dataset(
            "aborted_dataset".to_string(),
            "This dataset will be aborted".to_string(),
            vec!["test".to_string(), "abort".to_string()],
            test_schema.clone(),
        )
        .await?;

    // Write a row
    // We recreate test_rows to avoid needing to clone it, since DatasetRow doesn't
    // derive Clone
    writer.write(create_test_rows().remove(0)).await?;

    // Drop the writer without calling finish()
    drop(writer);

    wait_for_dataset_status(&app_manager, "aborted_dataset", DatasetStatus::Aborted).await?;

    // Shutdown the server
    app_manager.shutdown().await;
    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn test_probe_existing_ui_reports_not_running_without_server() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let probe_result = Client::probe_existing_ui(workspace_path).await?;

    assert_eq!(probe_result, ExistingUiProbeResult::NotRunning);

    temp_dir.close()?;
    Ok(())
}

#[tokio::test]
async fn test_probe_existing_ui_reports_not_running_for_migration_needed_workspace()
-> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;
    std::fs::write(
        workspace_path.join(".fricon_workspace.json"),
        serde_json::json!({ "version": 0 }).to_string(),
    )?;

    let probe_result = Client::probe_existing_ui(workspace_path).await?;

    assert_eq!(probe_result, ExistingUiProbeResult::NotRunning);

    temp_dir.close()?;
    Ok(())
}

#[tokio::test]
async fn test_show_ui_requires_attached_ui_subscriber() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;
    let client = Client::connect(workspace_path).await?;

    assert_eq!(
        Client::probe_existing_ui(workspace_path).await?,
        ExistingUiProbeResult::UiUnavailable
    );

    let error = client
        .show_ui()
        .await
        .expect_err("show_ui should fail when no UI subscriber is attached");
    let ClientError::Status(status) = &error else {
        panic!("show_ui should return ClientError::Status, got: {error:?}");
    };
    assert_eq!(status.code(), Code::FailedPrecondition);

    app_manager.shutdown().await;
    temp_dir.close()?;
    Ok(())
}

#[tokio::test]
async fn test_probe_existing_ui_delegates_when_subscriber_is_attached() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;
    let mut event_rx = app_manager.handle().subscribe_ui_commands()?;

    let probe_result = Client::probe_existing_ui(workspace_path).await?;

    assert_eq!(probe_result, ExistingUiProbeResult::UiShown);
    assert!(matches!(event_rx.recv().await?, UiCommand::ShowUi));

    app_manager.shutdown().await;
    temp_dir.close()?;
    Ok(())
}

#[tokio::test]
async fn test_get_missing_dataset_returns_typed_not_found_error() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;
    let client = Client::connect(workspace_path).await?;

    let error = match client.get_dataset_by_id(999_999).await {
        Ok(_) => anyhow::bail!("missing dataset should return a typed client error"),
        Err(error) => error,
    };
    assert!(matches!(error, ClientError::DatasetNotFound));

    app_manager.shutdown().await;
    temp_dir.close()?;
    Ok(())
}

#[tokio::test]
async fn test_deleted_dataset_returns_typed_deleted_error() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();
    WorkspaceRoot::create_new(workspace_path)?;

    let app_manager =
        AppManager::new_with_path(workspace_path)?.start(&tokio::runtime::Handle::current())?;
    let client = Client::connect(workspace_path).await?;

    let test_rows = create_test_rows();
    let test_schema = test_rows[0].to_schema();
    let mut writer = client
        .create_dataset(
            "deleted_dataset".to_string(),
            "Dataset that will be deleted".to_string(),
            vec!["test".to_string()],
            test_schema,
        )
        .await?;
    writer.write(create_test_rows().remove(0)).await?;
    let dataset = writer.finish().await?;

    app_manager.handle().trash_dataset(dataset.id()).await?;
    app_manager.handle().delete_dataset(dataset.id()).await?;

    let error = match client.get_dataset_by_id(dataset.id()).await {
        Ok(_) => anyhow::bail!("deleted dataset should surface a typed client error"),
        Err(error) => error,
    };
    assert!(matches!(error, ClientError::DatasetDeleted));

    app_manager.shutdown().await;
    temp_dir.close()?;
    Ok(())
}

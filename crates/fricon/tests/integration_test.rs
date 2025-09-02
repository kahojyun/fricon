use std::sync::Arc;

use arrow::{
    array::{Array, Int32Array, RecordBatch, StringArray},
    datatypes::{DataType, Field, Schema},
};
use fricon::{AppManager, Client, DatasetId, init_workspace};
use tempfile::TempDir;

fn create_test_batch() -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("value", DataType::Int32, true),
    ]);

    let id_array = Int32Array::from(vec![1, 2, 3]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);
    let value_array = Int32Array::from(vec![Some(100), Some(200), None]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array),
            Arc::new(name_array),
            Arc::new(value_array),
        ],
    )
    .unwrap()
}

#[tokio::test]
async fn test_dataset_create_and_load() -> anyhow::Result<()> {
    // Create a temporary directory for the workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize the workspace
    init_workspace(workspace_path).await?;

    // Start the server
    let app_manager = AppManager::serve(workspace_path).await?;

    // Wait a bit for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect the client
    let client = Client::connect(workspace_path).await?;

    // Create test data
    let test_batch = create_test_batch();

    // Create dataset through client
    let mut writer = client.create_dataset(
        "test_dataset".to_string(),
        "Test dataset for integration test".to_string(),
        vec!["test".to_string(), "integration".to_string()],
    )?;

    // Write the test batch
    writer.write(test_batch.clone()).await?;

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

    assert_eq!(loaded_batch.num_rows(), test_batch.num_rows());
    assert_eq!(loaded_batch.num_columns(), test_batch.num_columns());
    assert_eq!(loaded_batch.schema(), test_batch.schema());

    // Compare column by column
    for col_idx in 0..loaded_batch.num_columns() {
        let loaded_col = loaded_batch.column(col_idx);
        let expected_col = test_batch.column(col_idx);

        if col_idx == 0 {
            // id column
            let loaded_array = loaded_col.as_any().downcast_ref::<Int32Array>().unwrap();
            let expected_array = expected_col.as_any().downcast_ref::<Int32Array>().unwrap();
            assert_eq!(loaded_array.values(), expected_array.values());
        } else if col_idx == 1 {
            // name column
            let loaded_array = loaded_col.as_any().downcast_ref::<StringArray>().unwrap();
            let expected_array = expected_col.as_any().downcast_ref::<StringArray>().unwrap();
            for i in 0..loaded_array.len() {
                assert_eq!(loaded_array.value(i), expected_array.value(i));
            }
        } else if col_idx == 2 {
            // value column
            let loaded_array = loaded_col.as_any().downcast_ref::<Int32Array>().unwrap();
            let expected_array = expected_col.as_any().downcast_ref::<Int32Array>().unwrap();
            for i in 0..loaded_array.len() {
                assert_eq!(loaded_array.is_valid(i), expected_array.is_valid(i));
                if loaded_array.is_valid(i) {
                    assert_eq!(loaded_array.value(i), expected_array.value(i));
                }
            }
        }
    }

    // Shutdown the server
    app_manager.shutdown().await;
    temp_dir.close()?;

    Ok(())
}

#[tokio::test]
async fn test_dataset_multiple_batches() -> anyhow::Result<()> {
    // Create a temporary directory for the workspace
    let temp_dir = TempDir::new()?;
    let workspace_path = temp_dir.path();

    // Initialize the workspace
    init_workspace(workspace_path).await?;

    // Start the server
    let app_manager = AppManager::serve(workspace_path).await?;

    // Wait a bit for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect the client
    let client = Client::connect(workspace_path).await?;

    // Create multiple test batches
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
    ]);

    let batch1 = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![
            Arc::new(Int32Array::from(vec![1, 2])),
            Arc::new(StringArray::from(vec!["Alice", "Bob"])),
        ],
    )
    .unwrap();

    let batch2 = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(Int32Array::from(vec![3, 4])),
            Arc::new(StringArray::from(vec!["Charlie", "David"])),
        ],
    )
    .unwrap();

    // Create dataset through client
    let mut writer = client.create_dataset(
        "multi_batch_dataset".to_string(),
        "Dataset with multiple batches".to_string(),
        vec!["multi".to_string()],
    )?;

    // Write both batches
    writer.write(batch1).await?;
    writer.write(batch2).await?;

    // Finish writing and get the dataset
    let dataset = writer.finish().await?;

    // Load dataset using DatasetManager
    let dataset_manager = app_manager.handle().dataset_manager();
    let reader = dataset_manager
        .get_dataset_reader(DatasetId::Id(dataset.id()))
        .await?;
    let loaded_batches: Vec<RecordBatch> = reader
        .batches()
        .expect("expected completed dataset")
        .to_vec();

    let total_rows: usize = loaded_batches.iter().map(RecordBatch::num_rows).sum();
    assert_eq!(total_rows, 4);

    // Shutdown the server
    app_manager.shutdown().await;
    temp_dir.close()?;

    Ok(())
}

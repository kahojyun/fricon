//! UI Chart Service for local chart data processing
//!
//! This module provides chart visualization support within the fricon-ui layer,
//! migrated from the core fricon crate to maintain clear architectural boundaries.
//! It handles:
//! - Local chart data processing from Arrow files
//! - Chart configuration management
//! - Real-time chart updates for active datasets
//! - ECharts-optimized data formatting

use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use arrow::{array::ArrayRef, ipc::reader::FileReader, record_batch::RecordBatch};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use fricon::{
    AppHandle, DatasetStatus,
    dataset_manager::{DatasetId, DatasetMetadata},
    schema_utils::{
        ColumnDataType, ColumnInfo, ColumnValue, extract_column_value_at, inspect_dataset_schema,
    },
};

/// Chart-specific errors for UI layer
#[derive(Debug, thiserror::Error)]
pub enum UIChartError {
    #[error("Dataset {dataset_id} not found")]
    DatasetNotFound { dataset_id: i32 },

    #[error("Dataset {dataset_id} is not ready for charting: {status:?}")]
    DatasetNotReady {
        dataset_id: i32,
        status: DatasetStatus,
    },

    #[error("Arrow file not found for dataset {dataset_id}")]
    ArrowFileNotFound { dataset_id: i32 },

    #[error("Chart data processing error: {0}")]
    DataProcessing(#[from] anyhow::Error),

    #[error("Configuration error: {0}")]
    Configuration(#[from] fricon::configuration_service::ConfigurationError),
}

/// Filter for index column values (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexColumnFilter {
    pub column: String,
    pub value: ColumnValue,
}

/// Chart data request from frontend (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataRequest {
    pub dataset_id: i32,
    pub x_column: String,
    pub y_columns: Vec<String>,
    pub index_column_filters: Vec<IndexColumnFilter>,
}

/// Schema response for chart configuration UI (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSchemaResponse {
    pub columns: Vec<ChartColumnInfo>,
    pub index_columns: Vec<String>,
}

/// Column information for chart configuration (UI-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartColumnInfo {
    pub name: String,
    pub data_type: ColumnDataType,
    pub is_index_column: bool,
    pub unique_values: Option<Vec<ColumnValue>>,
}

impl From<ColumnInfo> for ChartColumnInfo {
    fn from(info: ColumnInfo) -> Self {
        Self {
            name: info.name,
            data_type: info.data_type,
            is_index_column: info.is_index_column,
            unique_values: info.sample_values, // Use sample_values as unique_values
        }
    }
}

/// ECharts optimized dataset format (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataset {
    pub dimensions: Vec<String>,
    pub source: Vec<Vec<ColumnValue>>,
}

/// ECharts series configuration (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsSeries {
    pub name: String,
    #[serde(rename = "type")]
    pub chart_type: String,
    pub data_group_id: usize,
}

/// Response with ECharts formatted data (re-exported from core)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EChartsDataResponse {
    pub dataset: EChartsDataset,
    pub series: Vec<EChartsSeries>,
}

/// Chart update notification for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartUpdate {
    pub dataset_id: i32,
    pub update_type: ChartUpdateType,
    pub data: Option<EChartsDataResponse>,
    pub timestamp: DateTime<Utc>,
}

/// Types of chart updates for live plotting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartUpdateType {
    BatchAdded,
    DatasetCompleted,
    BufferFull,
}

/// Live plotter for real-time chart data management (UI layer)
pub struct LivePlotter {
    dataset_id: i32,
    data_buffer: Arc<Mutex<Vec<RecordBatch>>>,
    subscribers: Arc<Mutex<Vec<broadcast::Sender<ChartUpdate>>>>,
    max_buffer_size: usize,
}

impl LivePlotter {
    const MAX_BUFFER_BATCHES: usize = 100;

    /// Create a new live plotter for the given dataset
    #[must_use]
    pub fn new(dataset_id: i32) -> Self {
        Self {
            dataset_id,
            data_buffer: Arc::new(Mutex::new(Vec::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            max_buffer_size: Self::MAX_BUFFER_BATCHES,
        }
    }

    /// Write a new batch to the live buffer and notify subscribers
    pub fn write_batch(&self, batch: RecordBatch) -> Result<()> {
        let mut buffer = self.data_buffer.lock().unwrap();
        buffer.push(batch);

        // Maintain rolling window
        if buffer.len() > self.max_buffer_size {
            buffer.remove(0);
        }

        // Notify subscribers
        self.notify_subscribers(ChartUpdateType::BatchAdded);

        Ok(())
    }

    /// Get current chart data from memory buffer
    pub fn get_current_data(&self, request: ChartDataRequest) -> Result<EChartsDataResponse> {
        let buffer = self.data_buffer.lock().unwrap();

        if buffer.is_empty() {
            return Ok(EChartsDataResponse {
                dataset: EChartsDataset {
                    dimensions: vec![],
                    source: vec![],
                },
                series: vec![],
            });
        }

        // Concatenate all batches and process like file-based data
        let combined_batch = arrow::compute::concat_batches(&buffer[0].schema(), buffer.iter())?;

        // Use chart data processing logic
        UIChartService::process_batch_data(request, &combined_batch)
    }

    /// Subscribe to chart updates for this live plotter
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<ChartUpdate> {
        let (tx, rx) = broadcast::channel(1000);

        {
            let mut subscribers = self.subscribers.lock().unwrap();
            subscribers.push(tx);
        }

        rx
    }

    /// Notify all subscribers of a chart update
    fn notify_subscribers(&self, update_type: ChartUpdateType) {
        let update = ChartUpdate {
            dataset_id: self.dataset_id,
            update_type,
            data: None, // Data will be fetched on demand
            timestamp: Utc::now(),
        };

        // Notify local subscribers
        let subscribers = self.subscribers.lock().unwrap();
        for sender in subscribers.iter() {
            let _ = sender.send(update.clone());
        }
    }

    /// Mark dataset as completed and clean up
    pub fn mark_completed(&self) {
        self.notify_subscribers(ChartUpdateType::DatasetCompleted);
    }
}

/// UI Chart Service for local chart processing
pub struct UIChartService {
    app: AppHandle,
    live_plotters: Arc<Mutex<HashMap<i32, Arc<LivePlotter>>>>,
}

impl UIChartService {
    /// Create a new UI chart service instance
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            live_plotters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get chart schema information for a dataset
    pub async fn get_schema(&self, dataset_id: i32) -> Result<ChartSchemaResponse, UIChartError> {
        debug!("Getting chart schema for dataset {}", dataset_id);

        // Get dataset metadata for index columns
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await
            .map_err(|_| UIChartError::DatasetNotFound { dataset_id })?;

        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(dataset.metadata.uuid);

        // Read Arrow schema from IPC file using schema_utils
        let arrow_file = dataset_path.join("dataset.arrow");
        if !arrow_file.exists() {
            return Err(UIChartError::ArrowFileNotFound { dataset_id });
        }

        let schema_info = inspect_dataset_schema(&arrow_file, &dataset.metadata.index_columns)
            .map_err(UIChartError::DataProcessing)?;

        // Convert to chart-specific format
        let columns: Vec<ChartColumnInfo> = schema_info
            .columns
            .into_iter()
            .map(std::convert::Into::into)
            .collect();

        Ok(ChartSchemaResponse {
            columns,
            index_columns: dataset.metadata.index_columns,
        })
    }

    /// Get chart data with automatic data source detection
    pub async fn get_data(
        &self,
        request: ChartDataRequest,
    ) -> Result<EChartsDataResponse, UIChartError> {
        debug!("Getting chart data for dataset {}", request.dataset_id);

        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(request.dataset_id))
            .await
            .map_err(|_| UIChartError::DatasetNotFound {
                dataset_id: request.dataset_id,
            })?;

        match dataset.metadata.status {
            DatasetStatus::Completed => {
                // Read from Arrow file
                self.read_chart_data_from_file(request, dataset.metadata)
                    .await
            }
            DatasetStatus::Writing => {
                // Read from live plotter if available
                self.read_chart_data_from_memory(request).await
            }
            status => Err(UIChartError::DatasetNotReady {
                dataset_id: request.dataset_id,
                status,
            }),
        }
    }

    /// Read chart data from completed Arrow file
    async fn read_chart_data_from_file(
        &self,
        request: ChartDataRequest,
        metadata: DatasetMetadata,
    ) -> Result<EChartsDataResponse, UIChartError> {
        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(metadata.uuid);

        let arrow_file = dataset_path.join("dataset.arrow");
        if !arrow_file.exists() {
            return Err(UIChartError::ArrowFileNotFound {
                dataset_id: request.dataset_id,
            });
        }

        let file = File::open(&arrow_file)
            .with_context(|| format!("Failed to open Arrow file: {}", arrow_file.display()))?;

        let reader =
            FileReader::try_new(file, None).context("Failed to create Arrow file reader")?;

        let mut chart_data = Vec::new();

        for batch_result in reader {
            let batch = batch_result?;
            let filtered_indices =
                Self::apply_index_filters(&batch, &request.index_column_filters)?;

            for &row_idx in &filtered_indices {
                let mut row_values = Vec::new();

                // X-axis value
                if let Some(x_value) = Self::extract_value_at(&batch, &request.x_column, row_idx)? {
                    row_values.push(x_value);

                    // Y-axis values (numeric only)
                    let mut all_y_valid = true;
                    let mut y_values = Vec::new();

                    for y_column in &request.y_columns {
                        if let Some(y_value) =
                            Self::extract_numeric_value_at(&batch, y_column, row_idx)?
                        {
                            y_values.push(ColumnValue::Number(y_value));
                        } else {
                            all_y_valid = false;
                            break;
                        }
                    }

                    if all_y_valid {
                        row_values.extend(y_values);
                        chart_data.push(row_values);
                    }
                }
            }
        }

        Ok(Self::format_for_echarts(request, chart_data))
    }

    /// Read chart data from live plotter memory buffer
    async fn read_chart_data_from_memory(
        &self,
        request: ChartDataRequest,
    ) -> Result<EChartsDataResponse, UIChartError> {
        let plotters = self.live_plotters.lock().unwrap();

        if let Some(plotter) = plotters.get(&request.dataset_id) {
            plotter
                .get_current_data(request)
                .map_err(UIChartError::DataProcessing)
        } else {
            // No live plotter available, return empty data
            Ok(EChartsDataResponse {
                dataset: EChartsDataset {
                    dimensions: vec![],
                    source: vec![],
                },
                series: vec![],
            })
        }
    }

    /// Process chart data from a single RecordBatch
    pub fn process_batch_data(
        request: ChartDataRequest,
        batch: &RecordBatch,
    ) -> Result<EChartsDataResponse> {
        let filtered_indices = Self::apply_index_filters(batch, &request.index_column_filters)?;
        let mut chart_data = Vec::new();

        for &row_idx in &filtered_indices {
            let mut row_values = Vec::new();

            // X-axis value
            if let Some(x_value) = Self::extract_value_at(batch, &request.x_column, row_idx)? {
                row_values.push(x_value);

                // Y-axis values (numeric only)
                let mut all_y_valid = true;
                let mut y_values = Vec::new();

                for y_column in &request.y_columns {
                    if let Some(y_value) = Self::extract_numeric_value_at(batch, y_column, row_idx)?
                    {
                        y_values.push(ColumnValue::Number(y_value));
                    } else {
                        all_y_valid = false;
                        break;
                    }
                }

                if all_y_valid {
                    row_values.extend(y_values);
                    chart_data.push(row_values);
                }
            }
        }

        Ok(Self::format_for_echarts(request, chart_data))
    }

    /// Create or get live plotter for a dataset
    pub async fn create_live_plotter(
        &self,
        dataset_id: i32,
    ) -> Result<Arc<LivePlotter>, UIChartError> {
        // Check if plotter already exists
        {
            let plotters = self.live_plotters.lock().unwrap();
            if let Some(plotter) = plotters.get(&dataset_id) {
                return Ok(plotter.clone());
            }
        }

        // Verify dataset exists and is in writing state
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await
            .map_err(|_| UIChartError::DatasetNotFound { dataset_id })?;

        if dataset.metadata.status != DatasetStatus::Writing {
            return Err(UIChartError::DatasetNotReady {
                dataset_id,
                status: dataset.metadata.status,
            });
        }

        // Create and insert the new plotter
        let plotter = Arc::new(LivePlotter::new(dataset_id));
        {
            let mut plotters = self.live_plotters.lock().unwrap();
            plotters.insert(dataset_id, plotter.clone());
        }

        Ok(plotter)
    }

    /// Subscribe to live chart updates for a dataset
    pub fn subscribe_updates(
        &self,
        dataset_id: i32,
    ) -> Result<broadcast::Receiver<ChartUpdate>, UIChartError> {
        let plotters = self.live_plotters.lock().unwrap();

        if let Some(plotter) = plotters.get(&dataset_id) {
            Ok(plotter.subscribe())
        } else {
            // Create a new plotter on demand for subscription
            drop(plotters);
            let _ = self.create_live_plotter(dataset_id);

            let plotters = self.live_plotters.lock().unwrap();
            if let Some(plotter) = plotters.get(&dataset_id) {
                Ok(plotter.subscribe())
            } else {
                // Return empty channel if creation failed
                let (_tx, rx) = broadcast::channel(1);
                Ok(rx)
            }
        }
    }

    /// Clean up completed dataset resources
    pub async fn cleanup_completed_dataset(&self, dataset_id: i32) {
        let mut plotters = self.live_plotters.lock().unwrap();

        if let Some(plotter) = plotters.remove(&dataset_id) {
            plotter.mark_completed();
        }
    }

    // Helper methods migrated from core chart implementation

    fn apply_index_filters(
        batch: &RecordBatch,
        filters: &[IndexColumnFilter],
    ) -> Result<Vec<usize>> {
        if filters.is_empty() {
            return Ok((0..batch.num_rows()).collect());
        }

        let mut matching_indices = Vec::new();

        'row_loop: for row_idx in 0..batch.num_rows() {
            for filter in filters {
                let column_index = batch
                    .schema()
                    .column_with_name(&filter.column)
                    .map(|(idx, _)| idx)
                    .with_context(|| format!("Filter column '{}' not found", filter.column))?;

                let column = batch.column(column_index);

                if let Some(row_value) = extract_column_value_at(column, row_idx)? {
                    if row_value != filter.value {
                        continue 'row_loop;
                    }
                } else {
                    // Null values don't match any filter
                    continue 'row_loop;
                }
            }
            matching_indices.push(row_idx);
        }

        Ok(matching_indices)
    }

    fn extract_value_at(
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<ColumnValue>> {
        let column_index = batch
            .schema()
            .column_with_name(column_name)
            .map(|(idx, _)| idx)
            .with_context(|| format!("Column '{column_name}' not found"))?;

        let column = batch.column(column_index);
        extract_column_value_at(column, row_idx)
    }

    fn extract_numeric_value_at(
        batch: &RecordBatch,
        column_name: &str,
        row_idx: usize,
    ) -> Result<Option<f64>> {
        if let Some(value) = Self::extract_value_at(batch, column_name, row_idx)? {
            match value {
                ColumnValue::Number(n) => Ok(Some(n)),
                _ => Ok(None), // Non-numeric values are filtered out
            }
        } else {
            Ok(None)
        }
    }

    fn format_for_echarts(
        request: ChartDataRequest,
        rows: Vec<Vec<ColumnValue>>,
    ) -> EChartsDataResponse {
        let mut dimensions = vec![request.x_column];
        dimensions.extend(request.y_columns.iter().cloned());

        let dataset = EChartsDataset {
            dimensions: dimensions.clone(),
            source: rows,
        };

        let series = request
            .y_columns
            .into_iter()
            .enumerate()
            .map(|(idx, name)| EChartsSeries {
                name,
                chart_type: "line".to_string(),
                data_group_id: idx + 1, // +1 because 0 is X-axis
            })
            .collect();

        EChartsDataResponse { dataset, series }
    }
}

impl std::fmt::Debug for LivePlotter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivePlotter")
            .field("dataset_id", &self.dataset_id)
            .field("max_buffer_size", &self.max_buffer_size)
            .finish()
    }
}

impl std::fmt::Debug for UIChartService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UIChartService")
            .field("app", &"<AppHandle>")
            .finish()
    }
}

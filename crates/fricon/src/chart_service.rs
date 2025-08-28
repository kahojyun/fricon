//! Chart Service for unified visualization support
//!
//! This module provides a centralized service for chart operations that supports both
//! completed datasets (file-based) and active datasets (memory-based with live updates).
//!
//! The `ChartService` integrates with the existing `DatasetManager` and provides:
//! - Unified data source abstraction (File vs Memory)
//! - Real-time chart updates for active datasets
//! - Live plotting capabilities with data update notifications
//! - Backward compatibility with existing chart functionality

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{
    app::AppHandle,
    chart::{ChartDataRequest, ChartSchemaResponse, EChartsDataResponse},
    database::DatasetStatus,
    dataset_manager::DatasetId,
};

/// Errors specific to chart service operations
#[derive(Debug, thiserror::Error)]
pub enum ChartServiceError {
    #[error("Dataset {dataset_id} is not ready for charting: {status:?}")]
    DatasetNotReady {
        dataset_id: i32,
        status: DatasetStatus,
    },

    #[error("Live plotter not found for dataset {dataset_id}")]
    LivePlotterNotFound { dataset_id: i32 },

    #[error("Dataset {dataset_id} not found")]
    DatasetNotFound { dataset_id: i32 },

    #[error("Chart data processing error: {0}")]
    DataProcessing(#[from] anyhow::Error),
}

/// Data source abstraction for unified chart operations
#[derive(Debug, Clone)]
pub enum DataSource {
    /// Completed dataset - read from Arrow file
    File(PathBuf),
    /// Active dataset - read from memory buffer
    Memory(Arc<LivePlotter>),
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

/// Live plotter for real-time chart data management
pub struct LivePlotter {
    dataset_id: i32,
    data_buffer: Arc<Mutex<Vec<RecordBatch>>>,
    subscribers: Arc<Mutex<Vec<broadcast::Sender<ChartUpdate>>>>,
    max_buffer_size: usize,
    app_handle: AppHandle, // Add AppHandle for event broadcasting
}

impl LivePlotter {
    const MAX_BUFFER_BATCHES: usize = 100;

    /// Create a new live plotter for the given dataset
    #[must_use]
    pub fn new(dataset_id: i32, app_handle: AppHandle) -> Self {
        Self {
            dataset_id,
            data_buffer: Arc::new(Mutex::new(Vec::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            max_buffer_size: Self::MAX_BUFFER_BATCHES,
            app_handle,
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

        // Notify subscribers (throttled)
        self.notify_subscribers(ChartUpdateType::BatchAdded);

        Ok(())
    }

    /// Get current chart data from memory buffer
    pub fn get_current_data(&self, request: ChartDataRequest) -> Result<EChartsDataResponse> {
        let buffer = self.data_buffer.lock().unwrap();

        if buffer.is_empty() {
            return Ok(EChartsDataResponse {
                dataset: crate::chart::EChartsDataset {
                    dimensions: vec![],
                    source: vec![],
                },
                series: vec![],
            });
        }

        // Concatenate all batches and process like file-based data
        let combined_batch = arrow::compute::concat_batches(&buffer[0].schema(), buffer.iter())?;

        // Use existing chart data processing logic
        crate::chart::ChartDataReader::process_batch_data(request, &combined_batch)
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

        // Send through AppEvent system for broader broadcasting
        self.app_handle
            .send_event(crate::app::AppEvent::ChartUpdate(update.clone()));

        // Also notify local subscribers
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

/// Centralized chart service for unified visualization support
pub struct ChartService {
    app: AppHandle,
    live_plotters: Arc<Mutex<HashMap<i32, Arc<LivePlotter>>>>,
}

impl ChartService {
    /// Create a new chart service instance
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            live_plotters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get chart schema information for a dataset (unified for both data sources)
    pub async fn get_schema(
        &self,
        dataset_id: i32,
    ) -> Result<ChartSchemaResponse, ChartServiceError> {
        // Use existing chart schema reader logic
        let schema_reader = crate::chart::ChartSchemaReader::new(self.app.clone());
        schema_reader
            .read_chart_schema(dataset_id)
            .await
            .map_err(ChartServiceError::DataProcessing)
    }

    /// Get chart data with automatic data source detection
    pub async fn get_data(
        &self,
        request: ChartDataRequest,
    ) -> Result<EChartsDataResponse, ChartServiceError> {
        let data_source = self.resolve_data_source(request.dataset_id).await?;

        match data_source {
            DataSource::File(_path) => {
                // Use existing file-based chart data reader
                let data_reader = crate::chart::ChartDataReader::new(self.app.clone());
                data_reader
                    .read_chart_data(request)
                    .await
                    .map_err(ChartServiceError::DataProcessing)
            }
            DataSource::Memory(live_plotter) => {
                // Read from memory buffer
                live_plotter
                    .get_current_data(request)
                    .map_err(ChartServiceError::DataProcessing)
            }
        }
    }

    /// Create or get live plotter for a dataset
    pub async fn create_live_plotter(
        &self,
        dataset_id: i32,
    ) -> Result<Arc<LivePlotter>, ChartServiceError> {
        // Check if plotter already exists
        {
            let plotters = self.live_plotters.lock().unwrap();
            if let Some(plotter) = plotters.get(&dataset_id) {
                return Ok(plotter.clone());
            }
        } // Drop the mutex guard here

        // Verify dataset exists and is in writing state
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await
            .map_err(|_| ChartServiceError::DatasetNotFound { dataset_id })?;

        if dataset.metadata.status != DatasetStatus::Writing {
            return Err(ChartServiceError::DatasetNotReady {
                dataset_id,
                status: dataset.metadata.status,
            });
        }

        // Create and insert the new plotter
        let plotter = Arc::new(LivePlotter::new(dataset_id, self.app.clone()));
        {
            let mut plotters = self.live_plotters.lock().unwrap();
            plotters.insert(dataset_id, plotter.clone());
        } // Drop the mutex guard here too

        Ok(plotter)
    }

    /// Subscribe to live chart updates for a dataset
    pub fn subscribe_updates(
        &self,
        dataset_id: i32,
    ) -> Result<broadcast::Receiver<ChartUpdate>, ChartServiceError> {
        let plotters = self.live_plotters.lock().unwrap();

        if let Some(plotter) = plotters.get(&dataset_id) {
            Ok(plotter.subscribe())
        } else {
            Err(ChartServiceError::LivePlotterNotFound { dataset_id })
        }
    }

    /// Clean up completed dataset resources
    pub async fn cleanup_completed_dataset(&self, dataset_id: i32) {
        let mut plotters = self.live_plotters.lock().unwrap();

        if let Some(plotter) = plotters.remove(&dataset_id) {
            plotter.mark_completed();
        }
    }

    /// Resolve data source for a dataset based on its status
    async fn resolve_data_source(&self, dataset_id: i32) -> Result<DataSource, ChartServiceError> {
        let dataset = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await
            .map_err(|_| ChartServiceError::DatasetNotFound { dataset_id })?;

        match dataset.metadata.status {
            DatasetStatus::Completed => {
                let path = self
                    .app
                    .root()
                    .paths()
                    .dataset_path_from_uuid(dataset.metadata.uuid)
                    .join("dataset.arrow");
                Ok(DataSource::File(path))
            }
            DatasetStatus::Writing => {
                // Check for existing plotter
                let existing_plotter = {
                    let plotters = self.live_plotters.lock().unwrap();
                    plotters.get(&dataset_id).cloned()
                }; // Drop mutex guard here

                if let Some(plotter) = existing_plotter {
                    Ok(DataSource::Memory(plotter))
                } else {
                    // Create live plotter on demand
                    let plotter = self.create_live_plotter(dataset_id).await?;
                    Ok(DataSource::Memory(plotter))
                }
            }
            status => Err(ChartServiceError::DatasetNotReady { dataset_id, status }),
        }
    }
}

impl std::fmt::Debug for LivePlotter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivePlotter")
            .field("dataset_id", &self.dataset_id)
            .field("max_buffer_size", &self.max_buffer_size)
            .field("app_handle", &"<AppHandle>")
            .finish()
    }
}

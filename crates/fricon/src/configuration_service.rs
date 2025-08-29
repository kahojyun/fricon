//! Configuration Service for dataset-specific JSON configurations
//!
//! This module provides functionality to store and manage JSON configurations
//! for datasets in a generic and extensible way. Configurations are stored as
//! files alongside dataset.arrow in each dataset directory.
//!
//! The service supports:
//! - Generic configuration CRUD operations with type safety
//! - File-based storage for cross-component sharing
//! - Configuration change event notifications
//! - Atomic configuration updates

use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::app::AppHandle;
use crate::dataset_manager::{DatasetId, DatasetManagerError};

/// Configuration-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Configuration type '{config_type}' not found for dataset {dataset_id}")]
    ConfigNotFound {
        dataset_id: i32,
        config_type: String,
    },

    #[error("Invalid configuration format for '{config_type}': {message}")]
    InvalidFormat {
        config_type: String,
        message: String,
    },

    #[error("Configuration file size exceeds limit for '{config_type}'")]
    SizeExceeded { config_type: String },

    #[error("Dataset not found: {dataset_id}")]
    DatasetNotFound { dataset_id: i32 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Dataset manager error: {0}")]
    DatasetManager(#[from] DatasetManagerError),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Configuration change events for notification system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationEvent {
    ConfigSaved {
        dataset_id: i32,
        config_type: String,
    },
    ConfigDeleted {
        dataset_id: i32,
        config_type: String,
    },
    ConfigUpdated {
        dataset_id: i32,
        config_type: String,
    },
}

/// Configuration service for managing dataset-specific JSON configurations
pub struct ConfigurationService {
    app: AppHandle,
}

impl ConfigurationService {
    /// Maximum configuration file size (1MB)
    const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

    /// Configuration file naming pattern
    const CONFIG_FILE_PATTERN: &'static str = "{}-config.json";

    /// Create a new configuration service instance
    #[must_use]
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Save configuration data for a dataset and config type
    ///
    /// # Arguments
    /// * `dataset_id` - The dataset ID to save configuration for
    /// * `config_type` - Type identifier for the configuration (e.g., "ui", "client")
    /// * `config` - The configuration data to save (must be serializable)
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct ChartConfig {
    ///     default_chart_type: String,
    ///     theme: String,
    /// }
    ///
    /// let config = ChartConfig {
    ///     default_chart_type: "line".to_string(),
    ///     theme: "dark".to_string(),
    /// };
    ///
    /// service.save_config(dataset_id, "ui", &config).await?;
    /// ```
    pub async fn save_config<T>(
        &self,
        dataset_id: i32,
        config_type: &str,
        config: &T,
    ) -> Result<(), ConfigurationError>
    where
        T: Serialize,
    {
        debug!(
            "Saving configuration '{}' for dataset {}",
            config_type, dataset_id
        );

        // Get dataset record to obtain UUID
        let record = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await?;

        let config_path = self.config_file_path(record.metadata.uuid, config_type);

        // Ensure the dataset directory exists
        self.ensure_config_directory(record.metadata.uuid).await?;

        // Serialize configuration to JSON
        let json_data = serde_json::to_string_pretty(config).map_err(|e| {
            ConfigurationError::InvalidFormat {
                config_type: config_type.to_string(),
                message: e.to_string(),
            }
        })?;

        // Check size limit
        if json_data.len() as u64 > Self::MAX_CONFIG_SIZE {
            return Err(ConfigurationError::SizeExceeded {
                config_type: config_type.to_string(),
            });
        }

        // Atomic write using temporary file
        self.save_config_atomic(&config_path, &json_data).await?;

        // Emit configuration change event
        self.app
            .send_event(crate::app::AppEvent::ConfigurationChanged(
                ConfigurationEvent::ConfigSaved {
                    dataset_id,
                    config_type: config_type.to_string(),
                },
            ));

        info!(
            "Saved configuration '{}' for dataset {} at {}",
            config_type,
            dataset_id,
            config_path.display()
        );

        Ok(())
    }

    /// Load configuration data for a dataset and config type
    ///
    /// # Arguments
    /// * `dataset_id` - The dataset ID to load configuration for
    /// * `config_type` - Type identifier for the configuration
    ///
    /// # Returns
    /// * `Ok(Some(T))` - Configuration found and deserialized successfully
    /// * `Ok(None)` - Configuration file does not exist
    /// * `Err(_)` - Error occurred during loading or deserialization
    pub async fn load_config<T>(
        &self,
        dataset_id: i32,
        config_type: &str,
    ) -> Result<Option<T>, ConfigurationError>
    where
        T: DeserializeOwned,
    {
        debug!(
            "Loading configuration '{}' for dataset {}",
            config_type, dataset_id
        );

        // Get dataset record to obtain UUID
        let record = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await?;

        let config_path = self.config_file_path(record.metadata.uuid, config_type);

        if !config_path.exists() {
            debug!(
                "Configuration '{}' not found for dataset {} at {}",
                config_type,
                dataset_id,
                config_path.display()
            );
            return Ok(None);
        }

        // Read and deserialize configuration
        let json_data = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: T =
            serde_json::from_str(&json_data).map_err(|e| ConfigurationError::InvalidFormat {
                config_type: config_type.to_string(),
                message: e.to_string(),
            })?;

        debug!(
            "Loaded configuration '{}' for dataset {} from {}",
            config_type,
            dataset_id,
            config_path.display()
        );

        Ok(Some(config))
    }

    /// Delete configuration data for a dataset and config type
    pub async fn delete_config(
        &self,
        dataset_id: i32,
        config_type: &str,
    ) -> Result<(), ConfigurationError> {
        debug!(
            "Deleting configuration '{}' for dataset {}",
            config_type, dataset_id
        );

        // Get dataset record to obtain UUID
        let record = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await?;

        let config_path = self.config_file_path(record.metadata.uuid, config_type);

        if config_path.exists() {
            fs::remove_file(&config_path).with_context(|| {
                format!("Failed to delete config file: {}", config_path.display())
            })?;

            // Emit configuration change event
            self.app
                .send_event(crate::app::AppEvent::ConfigurationChanged(
                    ConfigurationEvent::ConfigDeleted {
                        dataset_id,
                        config_type: config_type.to_string(),
                    },
                ));

            info!(
                "Deleted configuration '{}' for dataset {} at {}",
                config_type,
                dataset_id,
                config_path.display()
            );
        } else {
            warn!(
                "Configuration '{}' not found for deletion for dataset {} at {}",
                config_type,
                dataset_id,
                config_path.display()
            );
        }

        Ok(())
    }

    /// List all available configuration types for a dataset
    pub async fn list_config_types(
        &self,
        dataset_id: i32,
    ) -> Result<Vec<String>, ConfigurationError> {
        debug!("Listing configuration types for dataset {}", dataset_id);

        // Get dataset record to obtain UUID
        let record = self
            .app
            .dataset_manager()
            .get_dataset(DatasetId::Id(dataset_id))
            .await?;

        let dataset_path = self
            .app
            .root()
            .paths()
            .dataset_path_from_uuid(record.metadata.uuid);

        if !dataset_path.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&dataset_path).with_context(|| {
            format!(
                "Failed to read dataset directory: {}",
                dataset_path.display()
            )
        })?;

        let mut config_types = Vec::new();

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Check if file matches configuration pattern
            if file_name_str.ends_with("-config.json") {
                if let Some(config_type) = file_name_str.strip_suffix("-config.json") {
                    config_types.push(config_type.to_string());
                }
            }
        }

        config_types.sort();

        debug!(
            "Found {} configuration types for dataset {}: {:?}",
            config_types.len(),
            dataset_id,
            config_types
        );

        Ok(config_types)
    }

    /// Get the configuration file path for a dataset UUID and config type
    fn config_file_path(&self, dataset_uuid: Uuid, config_type: &str) -> PathBuf {
        let dataset_path = self.app.root().paths().dataset_path_from_uuid(dataset_uuid);

        dataset_path.join(format!("{}-config.json", config_type))
    }

    /// Ensure the configuration directory exists
    async fn ensure_config_directory(&self, dataset_uuid: Uuid) -> Result<(), ConfigurationError> {
        let dataset_path = self.app.root().paths().dataset_path_from_uuid(dataset_uuid);

        fs::create_dir_all(&dataset_path).with_context(|| {
            format!(
                "Failed to create dataset directory: {}",
                dataset_path.display()
            )
        })?;

        Ok(())
    }

    /// Perform atomic configuration file write using temporary file
    async fn save_config_atomic(
        &self,
        config_path: &Path,
        json_data: &str,
    ) -> Result<(), ConfigurationError> {
        let temp_path = config_path.with_extension("json.tmp");

        // Write to temporary file first
        {
            let file = File::create(&temp_path)
                .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;
            let mut writer = BufWriter::new(file);

            use std::io::Write;
            writer.write_all(json_data.as_bytes()).with_context(|| {
                format!("Failed to write to temp file: {}", temp_path.display())
            })?;

            writer
                .flush()
                .with_context(|| format!("Failed to flush temp file: {}", temp_path.display()))?;
        }

        // Atomic rename
        fs::rename(&temp_path, config_path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                config_path.display()
            )
        })?;

        Ok(())
    }
}

impl std::fmt::Debug for ConfigurationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurationService")
            .field("app", &"<AppHandle>")
            .finish()
    }
}

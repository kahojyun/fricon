//! Chart Configuration Types for fricon-ui
//!
//! This module defines configuration structures specific to chart visualization
//! in the fricon-ui layer. These configurations are stored using the generic
//! configuration service provided by the core fricon crate.

use serde::{Deserialize, Serialize};

use crate::chart_service::{ColumnValue, IndexColumnFilter};

/// Chart configuration for UI-specific visualization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfiguration {
    /// Default visualization settings
    pub default_visualization: DefaultVisualizationConfig,
    /// Saved chart views for quick access
    pub saved_views: Vec<SavedChartView>,
    /// Display and theme settings
    pub display_settings: DisplaySettings,
}

impl Default for ChartConfiguration {
    fn default() -> Self {
        Self {
            default_visualization: DefaultVisualizationConfig::default(),
            saved_views: Vec::new(),
            display_settings: DisplaySettings::default(),
        }
    }
}

/// Default visualization configuration for new charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultVisualizationConfig {
    /// Default chart type to use
    pub chart_type: ChartType,
    /// Default X-axis column (if any)
    pub x_axis: Option<String>,
    /// Default Y-axis columns
    pub y_axes: Vec<String>,
    /// Default index column filters
    pub index_filters: Vec<IndexColumnFilter>,
}

impl Default for DefaultVisualizationConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            x_axis: None,
            y_axes: Vec::new(),
            index_filters: Vec::new(),
        }
    }
}

/// Supported chart types in the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Line,
    Scatter,
    Bar,
    Heatmap,
}

impl Default for ChartType {
    fn default() -> Self {
        ChartType::Line
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Line => write!(f, "line"),
            ChartType::Scatter => write!(f, "scatter"),
            ChartType::Bar => write!(f, "bar"),
            ChartType::Heatmap => write!(f, "heatmap"),
        }
    }
}

/// Saved chart view configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedChartView {
    /// Unique identifier for the saved view
    pub id: String,
    /// Human-readable name for the view
    pub name: String,
    /// Chart type for this view
    pub chart_type: ChartType,
    /// X-axis column
    pub x_axis: String,
    /// Y-axis columns
    pub y_axes: Vec<String>,
    /// Index column filters
    pub index_filters: Vec<IndexColumnFilter>,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SavedChartView {
    /// Create a new saved chart view
    pub fn new(
        id: String,
        name: String,
        chart_type: ChartType,
        x_axis: String,
        y_axes: Vec<String>,
        index_filters: Vec<IndexColumnFilter>,
    ) -> Self {
        Self {
            id,
            name,
            chart_type,
            x_axis,
            y_axes,
            index_filters,
            description: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Set description for the saved view
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Display and theme settings for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// UI theme preference
    pub theme: Theme,
    /// Whether to show grid lines
    pub grid_lines: bool,
    /// Whether to show data labels on points
    pub data_labels: bool,
    /// Animation settings
    pub animation: AnimationSettings,
    /// Color palette preference
    pub color_palette: ColorPalette,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            grid_lines: true,
            data_labels: false,
            animation: AnimationSettings::default(),
            color_palette: ColorPalette::Default,
        }
    }
}

/// UI theme options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

/// Animation settings for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Whether animations are enabled
    pub enabled: bool,
    /// Animation duration in milliseconds
    pub duration: u32,
    /// Animation easing function
    pub easing: AnimationEasing,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 750,
            easing: AnimationEasing::CubicOut,
        }
    }
}

/// Animation easing options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationEasing {
    Linear,
    QuadIn,
    QuadOut,
    CubicIn,
    CubicOut,
    QuartIn,
    QuartOut,
    BounceOut,
}

/// Color palette options for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorPalette {
    Default,
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Blues,
    Greens,
    Reds,
    Custom(Vec<String>),
}

/// Chart interaction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSettings {
    /// Whether zooming is enabled
    pub zoom_enabled: bool,
    /// Whether panning is enabled
    pub pan_enabled: bool,
    /// Whether data selection is enabled
    pub selection_enabled: bool,
    /// Tooltip configuration
    pub tooltip: TooltipSettings,
}

impl Default for InteractionSettings {
    fn default() -> Self {
        Self {
            zoom_enabled: true,
            pan_enabled: true,
            selection_enabled: true,
            tooltip: TooltipSettings::default(),
        }
    }
}

/// Tooltip configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipSettings {
    /// Whether tooltips are enabled
    pub enabled: bool,
    /// Tooltip trigger mode
    pub trigger: TooltipTrigger,
    /// Whether to show all series in tooltip
    pub show_all_series: bool,
    /// Custom format string for tooltip content
    pub format: Option<String>,
}

impl Default for TooltipSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: TooltipTrigger::Item,
            show_all_series: false,
            format: None,
        }
    }
}

/// Tooltip trigger modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TooltipTrigger {
    Item,
    Axis,
    None,
}

/// Live plotting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePlottingConfig {
    /// Whether live plotting is enabled for this dataset
    pub enabled: bool,
    /// Update interval in milliseconds
    pub update_interval: u32,
    /// Maximum number of data points to show in live mode
    pub max_points: usize,
    /// Whether to auto-scroll to latest data
    pub auto_scroll: bool,
}

impl Default for LivePlottingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: 1000, // 1 second
            max_points: 1000,
            auto_scroll: true,
        }
    }
}

/// Export settings for chart images and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Default image format for chart exports
    pub image_format: ImageFormat,
    /// Image resolution for exports
    pub image_resolution: ImageResolution,
    /// Data export format preference
    pub data_format: DataExportFormat,
    /// Whether to include metadata in exports
    pub include_metadata: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            image_format: ImageFormat::Png,
            image_resolution: ImageResolution::High,
            data_format: DataExportFormat::Csv,
            include_metadata: true,
        }
    }
}

/// Supported image export formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Pdf,
}

/// Image resolution options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageResolution {
    Low,    // 72 DPI
    Medium, // 150 DPI
    High,   // 300 DPI
    Ultra,  // 600 DPI
}

/// Data export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataExportFormat {
    Csv,
    Json,
    Excel,
    Parquet,
}

/// Complete chart configuration including all sub-configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteChartConfiguration {
    /// Basic chart configuration
    #[serde(flatten)]
    pub chart: ChartConfiguration,
    /// Interaction settings
    pub interaction: InteractionSettings,
    /// Live plotting configuration
    pub live_plotting: LivePlottingConfig,
    /// Export settings
    pub export: ExportSettings,
}

impl Default for CompleteChartConfiguration {
    fn default() -> Self {
        Self {
            chart: ChartConfiguration::default(),
            interaction: InteractionSettings::default(),
            live_plotting: LivePlottingConfig::default(),
            export: ExportSettings::default(),
        }
    }
}

/// Configuration service helper for chart configurations
pub struct ChartConfigurationManager {
    app: fricon::AppHandle,
}

impl ChartConfigurationManager {
    /// Configuration type identifier for UI chart configs
    pub const CONFIG_TYPE: &'static str = "ui";

    /// Create a new chart configuration manager
    #[must_use]
    pub fn new(app: fricon::AppHandle) -> Self {
        Self { app }
    }

    /// Save chart configuration for a dataset
    pub async fn save_config(
        &self,
        dataset_id: i32,
        config: &CompleteChartConfiguration,
    ) -> Result<(), fricon::configuration_service::ConfigurationError> {
        self.app
            .configuration_service()
            .save_config(dataset_id, Self::CONFIG_TYPE, config)
            .await
    }

    /// Load chart configuration for a dataset
    pub async fn load_config(
        &self,
        dataset_id: i32,
    ) -> Result<Option<CompleteChartConfiguration>, fricon::configuration_service::ConfigurationError>
    {
        self.app
            .configuration_service()
            .load_config(dataset_id, Self::CONFIG_TYPE)
            .await
    }

    /// Load chart configuration with defaults if not found
    pub async fn load_config_with_defaults(
        &self,
        dataset_id: i32,
    ) -> Result<CompleteChartConfiguration, fricon::configuration_service::ConfigurationError> {
        match self.load_config(dataset_id).await? {
            Some(config) => Ok(config),
            None => Ok(CompleteChartConfiguration::default()),
        }
    }

    /// Delete chart configuration for a dataset
    pub async fn delete_config(
        &self,
        dataset_id: i32,
    ) -> Result<(), fricon::configuration_service::ConfigurationError> {
        self.app
            .configuration_service()
            .delete_config(dataset_id, Self::CONFIG_TYPE)
            .await
    }

    /// Add a saved view to the configuration
    pub async fn add_saved_view(
        &self,
        dataset_id: i32,
        view: SavedChartView,
    ) -> Result<(), fricon::configuration_service::ConfigurationError> {
        let mut config = self.load_config_with_defaults(dataset_id).await?;
        config.chart.saved_views.push(view);
        self.save_config(dataset_id, &config).await
    }

    /// Remove a saved view from the configuration
    pub async fn remove_saved_view(
        &self,
        dataset_id: i32,
        view_id: &str,
    ) -> Result<(), fricon::configuration_service::ConfigurationError> {
        let mut config = self.load_config_with_defaults(dataset_id).await?;
        config.chart.saved_views.retain(|view| view.id != view_id);
        self.save_config(dataset_id, &config).await
    }

    /// Update display settings
    pub async fn update_display_settings(
        &self,
        dataset_id: i32,
        display_settings: DisplaySettings,
    ) -> Result<(), fricon::configuration_service::ConfigurationError> {
        let mut config = self.load_config_with_defaults(dataset_id).await?;
        config.chart.display_settings = display_settings;
        self.save_config(dataset_id, &config).await
    }
}

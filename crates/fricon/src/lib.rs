//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod app;
mod chart;
mod client;
mod database;
mod dataset_manager;
mod ipc;
mod proto;
mod schema_utils;
mod server;
mod utils;
mod workspace;

pub use self::{
    app::{AppEvent, AppHandle, AppManager, init as init_workspace},
    chart::{
        ChartDataReader, ChartDataRequest, ChartSchemaReader, ChartSchemaResponse, ColumnInfo,
        EChartsDataResponse, IndexColumnFilter,
    },
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset_manager::DatasetMetadata,
    schema_utils::{
        ColumnDataType, ColumnValue, DatasetSchemaInfo, SchemaSummary, classify_data_type,
        compatibility::{ChartType, VisualizationCompatibility},
        custom_types::{
            CustomTypeInfo, complex128, get_all_custom_types, get_custom_type_info,
            get_trace_y_type, is_complex_type, is_trace_fixed_step, is_trace_type,
            is_trace_variable_step, trace, trace_fixed_step, trace_variable_step,
            validate_schema_custom_types,
        },
        extract_column_value_at, extract_unique_values, get_column_info, get_schema_summary,
        inspect_dataset_schema,
        inspector::{DatasetShape, SchemaInspector},
        is_visualization_supported,
    },
    server::{DatasetRecord, run as run_server},
    workspace::get_log_dir,
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

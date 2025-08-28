//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod app;
pub mod chart;
mod client;
mod database;
mod dataset_manager;
mod ipc;
mod proto;
pub mod schema_utils;
mod server;
mod utils;
mod workspace;

// Core client-server components - commonly used directly
pub use self::{
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset_manager::DatasetMetadata,
    server::{DatasetRecord, run as run_server},
};

// Application and workspace management
pub use self::{
    app::{AppEvent, AppHandle, AppManager, init as init_workspace},
    workspace::get_log_dir,
};

// Module re-exports for organized access
// Users can access these as fricon::chart::*, fricon::schema_utils::*, etc.
// The modules are made public above

// Commonly used types from modules with many exports
pub use self::schema_utils::{
    ColumnDataType, ColumnValue, DatasetSchemaInfo, inspect_dataset_schema,
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

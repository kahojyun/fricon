//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache
//!   Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod app;
mod client;
mod database;
mod dataset_manager;
mod dataset_tasks;
mod ipc;
mod live;
mod proto;
mod reader;
mod server;
mod utils;
mod workspace;
mod write_registry;
mod write_session;

pub use self::{
    app::{AppEvent, AppHandle, AppManager},
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetManager, DatasetManagerError, DatasetMetadata,
    },
    reader::DatasetReader,
    server::DatasetRecord,
    workspace::{WorkspaceRoot, get_log_dir},
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

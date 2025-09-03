//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod app;
mod background_writer;
mod client;
mod database;
mod dataset_manager; // core manager
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
    app::{AppEvent, AppHandle, AppManager, init as init_workspace},
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetManager, DatasetManagerError, DatasetMetadata,
    },
    reader::DatasetReader,
    server::DatasetRecord,
    workspace::get_log_dir,
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

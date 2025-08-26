//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod app;
mod client;
mod database;
mod dataset;
mod dataset_manager;
mod ipc;
mod proto;
mod server;
mod utils;
mod workspace;

pub use self::{
    app::{AppEvent, AppHandle, AppManager, init as init_workspace},
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset::Metadata as DatasetMetadata,
    server::{DatasetRecord, run as run_server},
    workspace::get_log_dir,
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

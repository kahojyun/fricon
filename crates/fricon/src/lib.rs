//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
mod client;
mod database;
mod dataset;
mod ipc;
mod paths;
mod proto;
mod server;
mod workspace;

pub use self::{
    client::{Client, Dataset, DatasetWriter},
    dataset::Metadata as DatasetMetadata,
    server::{DatasetRecord, run as run_server},
    workspace::init as init_workspace,
};

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

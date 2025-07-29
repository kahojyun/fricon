//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache Arrow format
//! - **Client-Server Architecture**: gRPC-based communication

pub mod client;
pub mod server;
pub mod workspace;

mod database;
mod dataset;
mod ipc;
mod paths;
mod proto;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

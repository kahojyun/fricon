pub mod client;
mod dataset;
mod db;
mod ipc;
pub mod paths;
pub mod proto;
pub mod server;
pub mod workspace;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

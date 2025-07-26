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

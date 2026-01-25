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
mod dataset;
mod dataset_fs;
mod dataset_manager;
mod ipc;
mod proto;
mod server;
mod utils;
mod workspace;

pub use self::{
    app::{AppEvent, AppHandle, AppManager},
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset::{
        DatasetArray, DatasetDataType, DatasetRow, DatasetScalar, DatasetSchema, FixedStepTrace,
        ScalarArray, ScalarKind, TraceKind, VariableStepTrace,
    },
    dataset_manager::{
        CreateDatasetRequest, DatasetId, DatasetManager, DatasetMetadata, DatasetReader,
        DatasetUpdate, SelectOptions,
    },
    server::DatasetRecord,
    workspace::{WorkspaceRoot, get_log_dir},
};

const DEFAULT_DATASET_LIST_LIMIT: i64 = 200;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

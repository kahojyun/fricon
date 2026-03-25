//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache
//!   Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
pub mod app;
pub mod client;
mod database;
pub mod dataset;
mod proto;
mod transport;
pub mod workspace;

pub use self::{
    app::{AppHandle, AppManager, CatalogAppError, IngestAppError, ReadAppError},
    client::{Client, ClientError, Dataset, DatasetWriter, ExistingUiProbeResult},
    dataset::{
        DatasetArray, DatasetDataType, DatasetEvent, DatasetId, DatasetListQuery, DatasetMetadata,
        DatasetReader, DatasetRecord, DatasetRow, DatasetScalar, DatasetSchema, DatasetSortBy,
        DatasetStatus, DatasetUpdate, FixedStepTrace, ScalarArray, ScalarKind, SelectOptions,
        SortDirection, TraceKind, VariableStepTrace,
    },
    workspace::{WorkspaceError, WorkspaceRoot, get_log_dir},
};

const DEFAULT_DATASET_LIST_LIMIT: i64 = 200;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version of the IPC/gRPC protocol between clients and the workspace server.
const IPC_PROTOCOL_VERSION: u32 = 1;

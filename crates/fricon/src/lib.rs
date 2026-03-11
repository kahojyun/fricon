//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache
//!   Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
pub mod client;
mod database;
pub mod dataset_catalog;
pub mod dataset_ingest;
pub mod dataset_read;
pub mod dataset_schema;
#[expect(
    dead_code,
    unreachable_pub,
    reason = "Compatibility shim during capability split"
)]
mod dataset_manager;
mod proto;
pub mod runtime;
mod storage;
mod transport;
mod utils;
pub mod workspace;

mod app {
    pub(crate) use crate::runtime::app::*;
}

mod dataset {
    pub(crate) use crate::dataset_schema::*;
}

mod dataset_fs {
    pub(crate) use crate::storage::*;
}

mod ipc {
    pub(crate) use crate::transport::ipc::*;
}

pub use self::{
    client::{Client, Dataset, DatasetWriter},
    database::DatasetStatus,
    dataset_catalog::{
        DatasetCatalogService, DatasetId, DatasetListQuery, DatasetMetadata, DatasetRecord,
        DatasetSortBy, DatasetUpdate, SortDirection,
    },
    dataset_ingest::{CreateDatasetRequest, DatasetIngestService},
    dataset_read::{DatasetReadService, DatasetReader, SelectOptions},
    dataset_schema::{
        DatasetArray, DatasetDataType, DatasetRow, DatasetScalar, DatasetSchema, FixedStepTrace,
        ScalarArray, ScalarKind, TraceKind, VariableStepTrace,
    },
    runtime::app::{AppEvent, AppHandle, AppManager},
    workspace::{WorkspaceRoot, get_log_dir},
};

const DEFAULT_DATASET_LIST_LIMIT: i64 = 200;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

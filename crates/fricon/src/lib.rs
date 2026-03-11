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
pub mod dataset;
mod proto;
mod transport;
mod utils;
pub mod workspace;

pub use self::{
    app::{AppEvent, AppHandle, AppManager},
    client::{Client, Dataset, DatasetWriter},
    dataset::{
        CreateDatasetRequest, DatasetArray, DatasetCatalogService, DatasetDataType, DatasetId,
        DatasetIngestService, DatasetListQuery, DatasetMetadata, DatasetReadService, DatasetReader,
        DatasetRecord, DatasetRow, DatasetScalar, DatasetSchema, DatasetSortBy, DatasetStatus,
        DatasetUpdate, FixedStepTrace, ScalarArray, ScalarKind, SelectOptions, SortDirection,
        TraceKind, VariableStepTrace,
    },
    workspace::{WorkspaceRoot, get_log_dir},
};

const DEFAULT_DATASET_LIST_LIMIT: i64 = 200;

/// Version of fricon crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

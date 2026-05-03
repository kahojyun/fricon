//! # fricon
//!
//! Data collection automation framework:
//!
//! - **Workspace Management**: Initialize and manage data workspaces
//! - **Dataset Operations**: Create, store, and query datasets using Apache
//!   Arrow format
//! - **Client-Server Architecture**: gRPC-based communication
pub mod app;
pub mod cli;
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
        ColumnMeaning, ColumnMetadata, DatasetArray, DatasetDataType, DatasetEvent, DatasetId,
        DatasetInterpretation, DatasetListQuery, DatasetMetadata, DatasetReader, DatasetRecord,
        DatasetRow, DatasetScalar, DatasetSchema, DatasetSortBy, DatasetStatus, DatasetUpdate,
        FixedStepTrace, InterpretationSource, PhysicalColumnOrdinal, ProjectedSemanticAxis,
        ProjectedSemanticRoles, ProjectedSemanticSource, ResolvedColumn, ResolvedDuplicatePolicy,
        ResolvedIndexRealization, ResolvedLogicalIndexPoint, ResolvedLogicalIndexReference,
        ResolvedPhysicalColumnReference, ResolvedScanAxis, ResolvedScanAxisMode,
        ResolvedSemanticReference, ScalarArray, ScalarKind, ScanAxis, ScanAxisMode, ScanAxisValue,
        ScanPlan, SelectOptions, SemanticProjectionOptions, SortDirection, TraceKind,
        VariableStepTrace, VisibleColumnOrdinal, logical_index_id, physical_column_id,
        project_semantic_source,
    },
    workspace::{WorkspaceError, WorkspaceRoot, get_log_dir},
};

const DEFAULT_DATASET_LIST_LIMIT: i64 = 200;

/// Version of fricon crate.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version of the IPC/gRPC protocol between clients and the workspace server.
const IPC_PROTOCOL_VERSION: u32 = 4;

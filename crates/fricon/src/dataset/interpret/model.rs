use crate::dataset::semantics::{DatasetDType, ScanAxisValue};

#[derive(Debug, Clone, PartialEq)]
pub struct DatasetInterpretation {
    pub columns: Vec<ResolvedColumn>,
    pub value_columns: Vec<VisibleColumnOrdinal>,
    pub logical_index_columns: Vec<VisibleColumnOrdinal>,
    pub chart_axis_candidate_columns: Vec<VisibleColumnOrdinal>,
    pub duplicate_policy: ResolvedDuplicatePolicy,
    pub index_realization: ResolvedIndexRealization,
    pub scan_axes: Vec<ResolvedScanAxis>,
    pub logical_index_points: Vec<ResolvedLogicalIndexPoint>,
    pub source: InterpretationSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalColumnOrdinal(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisibleColumnOrdinal(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "resolved column flags are compatibility and UI-facing projections"
)]
pub struct ResolvedColumn {
    pub name: String,
    pub physical_ordinal: PhysicalColumnOrdinal,
    pub visible_ordinal: Option<VisibleColumnOrdinal>,
    pub dtype: DatasetDType,
    pub meaning: ColumnMeaning,
    pub is_index: bool,
    pub is_system: bool,
    pub hidden_by_default: bool,
    pub is_chart_axis_candidate: bool,
    pub unit: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMeaning {
    UserValue,
    CompatibilityIndex,
    SystemRecordId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedIndexRealization {
    None,
    Implicit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedScanAxis {
    pub name: String,
    pub label: Option<String>,
    pub mode: ResolvedScanAxisMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedScanAxisMode {
    Static { values: Vec<ScanAxisValue> },
    ImplicitIndex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLogicalIndexPoint {
    pub record_id: u64,
    pub indices: Vec<u64>,
    pub coordinates: Vec<ScanAxisValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretationSource {
    Manifest,
    CompatibilityInference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedDuplicatePolicy {
    LatestByRecordId,
    CompatibilityRowOrderPlaceholder,
}

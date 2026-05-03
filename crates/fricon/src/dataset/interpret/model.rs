use crate::dataset::semantics::{DatasetDType, ScanAxisValue};

const COLUMN_ID_PREFIX: &str = "column:";
const LOGICAL_INDEX_ID_PREFIX: &str = "logicalIndex:";

#[derive(Debug, Clone, PartialEq)]
pub struct DatasetInterpretation {
    pub columns: Vec<ResolvedColumn>,
    pub semantic_references: Vec<ResolvedSemanticReference>,
    pub value_references: Vec<ResolvedSemanticReference>,
    pub plotted_coordinates: Vec<ResolvedSemanticReference>,
    pub sweep_axes: Vec<ResolvedSemanticReference>,
    pub group_axes: Vec<ResolvedSemanticReference>,
    pub filter_axes: Vec<ResolvedSemanticReference>,
    pub chart_axis_candidates: Vec<ResolvedSemanticReference>,
    pub value_columns: Vec<VisibleColumnOrdinal>,
    pub logical_index_columns: Vec<VisibleColumnOrdinal>,
    pub chart_axis_candidate_columns: Vec<VisibleColumnOrdinal>,
    pub duplicate_policy: ResolvedDuplicatePolicy,
    pub index_realization: ResolvedIndexRealization,
    pub scan_axes: Vec<ResolvedScanAxis>,
    pub source: InterpretationSource,
}

impl DatasetInterpretation {
    #[must_use]
    pub fn semantic_reference(&self, id: &str) -> Option<&ResolvedSemanticReference> {
        self.semantic_references
            .iter()
            .find(|reference| reference.id() == id)
    }

    #[must_use]
    pub fn physical_column_for_id(&self, id: &str) -> Option<&ResolvedPhysicalColumnReference> {
        self.semantic_reference(id)
            .and_then(|reference| match reference {
                ResolvedSemanticReference::PhysicalColumn(column) => Some(column),
                ResolvedSemanticReference::LogicalIndex(_) => None,
            })
    }

    #[must_use]
    pub fn logical_axis_for_id(&self, id: &str) -> Option<&ResolvedLogicalIndexReference> {
        self.semantic_reference(id)
            .and_then(|reference| match reference {
                ResolvedSemanticReference::PhysicalColumn(_) => None,
                ResolvedSemanticReference::LogicalIndex(axis) => Some(axis),
            })
    }
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
    pub id: String,
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
    pub is_complex: bool,
    pub is_trace: bool,
    pub is_numeric_axis_candidate: bool,
}

impl ResolvedColumn {
    #[must_use]
    pub fn as_semantic_reference(&self, is_compatibility: bool) -> ResolvedSemanticReference {
        ResolvedSemanticReference::PhysicalColumn(ResolvedPhysicalColumnReference {
            id: self.id.clone(),
            name: self.name.clone(),
            physical_ordinal: self.physical_ordinal,
            visible_ordinal: self.visible_ordinal,
            dtype: self.dtype.clone(),
            meaning: self.meaning,
            is_index: self.is_index,
            is_system: self.is_system,
            is_compatibility,
            hidden_by_default: self.hidden_by_default,
            is_chart_axis_candidate: self.is_chart_axis_candidate,
            unit: self.unit.clone(),
            label: self.label.clone(),
            is_complex: self.is_complex,
            is_trace: self.is_trace,
            numeric_axis: self.is_numeric_axis_candidate,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedSemanticReference {
    PhysicalColumn(ResolvedPhysicalColumnReference),
    LogicalIndex(ResolvedLogicalIndexReference),
}

impl ResolvedSemanticReference {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::PhysicalColumn(column) => &column.id,
            Self::LogicalIndex(axis) => &axis.id,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::PhysicalColumn(column) => &column.name,
            Self::LogicalIndex(axis) => &axis.name,
        }
    }

    #[must_use]
    pub fn label(&self) -> Option<&String> {
        match self {
            Self::PhysicalColumn(column) => column.label.as_ref(),
            Self::LogicalIndex(axis) => axis.label.as_ref(),
        }
    }

    #[must_use]
    pub const fn hidden_by_default(&self) -> bool {
        match self {
            Self::PhysicalColumn(column) => column.hidden_by_default,
            Self::LogicalIndex(axis) => axis.hidden_by_default,
        }
    }

    #[must_use]
    pub const fn numeric_axis(&self) -> bool {
        match self {
            Self::PhysicalColumn(column) => column.numeric_axis,
            Self::LogicalIndex(axis) => axis.numeric_axis,
        }
    }

    #[must_use]
    pub const fn is_compatibility(&self) -> bool {
        match self {
            Self::PhysicalColumn(column) => column.is_compatibility,
            Self::LogicalIndex(axis) => axis.is_compatibility,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "semantic references expose resolved role and UI capabilities"
)]
pub struct ResolvedPhysicalColumnReference {
    pub id: String,
    pub name: String,
    pub physical_ordinal: PhysicalColumnOrdinal,
    pub visible_ordinal: Option<VisibleColumnOrdinal>,
    pub dtype: DatasetDType,
    pub meaning: ColumnMeaning,
    pub is_index: bool,
    pub is_system: bool,
    pub is_compatibility: bool,
    pub hidden_by_default: bool,
    pub is_chart_axis_candidate: bool,
    pub unit: Option<String>,
    pub label: Option<String>,
    pub is_complex: bool,
    pub is_trace: bool,
    pub numeric_axis: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLogicalIndexReference {
    pub id: String,
    pub name: String,
    pub axis_ordinal: usize,
    pub label: Option<String>,
    pub hidden_by_default: bool,
    pub numeric_axis: bool,
    pub is_compatibility: bool,
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
    Sidecar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedScanAxis {
    pub id: String,
    pub axis_ordinal: usize,
    pub name: String,
    pub label: Option<String>,
    pub mode: ResolvedScanAxisMode,
    pub numeric_axis: bool,
}

impl ResolvedScanAxis {
    #[must_use]
    pub fn as_semantic_reference(&self) -> ResolvedSemanticReference {
        ResolvedSemanticReference::LogicalIndex(ResolvedLogicalIndexReference {
            id: self.id.clone(),
            name: self.name.clone(),
            axis_ordinal: self.axis_ordinal,
            label: self.label.clone(),
            hidden_by_default: false,
            numeric_axis: self.numeric_axis,
            is_compatibility: false,
        })
    }
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

#[must_use]
pub fn physical_column_id(name: &str) -> String {
    format!("{COLUMN_ID_PREFIX}{name}")
}

#[must_use]
pub fn logical_index_id(name: &str) -> String {
    format!("{LOGICAL_INDEX_ID_PREFIX}{name}")
}

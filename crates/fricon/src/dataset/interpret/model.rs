use crate::dataset::semantics::DatasetDType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetInterpretation {
    pub columns: Vec<ResolvedColumn>,
    pub value_columns: Vec<usize>,
    pub logical_index_columns: Vec<usize>,
    pub chart_axis_candidate_columns: Vec<usize>,
    pub duplicate_policy: ResolvedDuplicatePolicy,
    pub source: InterpretationSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedColumn {
    pub name: String,
    pub ordinal: usize,
    pub dtype: DatasetDType,
    pub meaning: ColumnMeaning,
    pub is_index: bool,
    pub is_system: bool,
    pub hidden_by_default: bool,
    pub is_chart_axis_candidate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnMeaning {
    UserValue,
    CompatibilityIndex,
    SystemRecordId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretationSource {
    Manifest,
    CompatibilityInference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedDuplicatePolicy {
    LatestByRecordIdPlaceholder,
    CompatibilityRowOrderPlaceholder,
}

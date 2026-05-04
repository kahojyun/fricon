mod model;

use std::collections::{HashMap, HashSet};

use arrow_schema::Schema;

pub use self::model::{
    ColumnMeaning, DatasetInterpretation, InterpretationSource, PhysicalColumnOrdinal,
    ResolvedColumn, ResolvedDuplicatePolicy, ResolvedIndexRealization, ResolvedLogicalIndexPoint,
    ResolvedLogicalIndexReference, ResolvedPhysicalColumnReference, ResolvedScanAxis,
    ResolvedScanAxisMode, ResolvedSemanticReference, VisibleColumnOrdinal, logical_index_id,
    physical_column_id,
};
use crate::dataset::{
    schema::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind},
    semantics::{
        DatasetDType, DatasetSemanticManifest, DuplicateResolutionDefault, IndexRealization,
        ScanAxisMode, ScanAxisValue, SystemColumn, TraceAxisDType, TraceLayout, TraceValueDType,
    },
};

pub(crate) fn resolve_from_manifest(
    arrow_schema: &Schema,
    manifest: &DatasetSemanticManifest,
    visible_columns: &[usize],
) -> DatasetInterpretation {
    let visible_ordinals: HashMap<_, _> = visible_columns
        .iter()
        .copied()
        .enumerate()
        .map(|(visible_ordinal, physical_ordinal)| {
            (physical_ordinal, VisibleColumnOrdinal(visible_ordinal))
        })
        .collect();
    let columns: Vec<_> = arrow_schema
        .fields()
        .iter()
        .enumerate()
        .map(|(physical_ordinal, field)| {
            let name = field.name().to_owned();
            let column = manifest
                .columns
                .get(&name)
                .expect("manifest should be validated against arrow schema");
            let is_record_id = column.system == Some(SystemColumn::RecordId);
            let dtype = column.dtype.clone();
            ResolvedColumn {
                id: physical_column_id(&name),
                name,
                physical_ordinal: PhysicalColumnOrdinal(physical_ordinal),
                visible_ordinal: visible_ordinals.get(&physical_ordinal).copied(),
                dtype: dtype.clone(),
                meaning: if is_record_id {
                    ColumnMeaning::SystemRecordId
                } else {
                    ColumnMeaning::UserValue
                },
                is_index: false,
                is_system: is_record_id,
                hidden_by_default: is_record_id || column.hidden_by_default,
                is_chart_axis_candidate: column.chart_axis,
                unit: column.unit.clone(),
                label: column.label.clone(),
                is_complex: dtype_is_complex(&dtype),
                is_trace: dtype_is_trace(&dtype),
                is_numeric_axis_candidate: dtype_is_chart_axis_numeric(&dtype),
            }
        })
        .collect();

    let value_columns = columns
        .iter()
        .filter(|column| column.meaning == ColumnMeaning::UserValue)
        .filter_map(|column| column.visible_ordinal)
        .collect();
    let chart_axis_candidate_columns = columns
        .iter()
        .filter(|column| column.is_chart_axis_candidate)
        .filter_map(|column| column.visible_ordinal)
        .collect();
    let scan_axes = resolve_scan_axes(manifest);
    let role_projections = manifest_role_projections(&columns, &scan_axes);

    DatasetInterpretation {
        columns,
        semantic_references: role_projections.semantic_references,
        value_references: role_projections.value_references,
        plotted_coordinates: role_projections.plotted_coordinates,
        sweep_axes: role_projections.sweep_axes,
        group_axes: role_projections.group_axes,
        filter_axes: role_projections.filter_axes,
        chart_axis_candidates: role_projections.chart_axis_candidates,
        value_columns,
        logical_index_columns: Vec::new(),
        chart_axis_candidate_columns,
        duplicate_policy: match manifest.realization.duplicate_resolution_default {
            DuplicateResolutionDefault::LatestByRecordId => {
                ResolvedDuplicatePolicy::LatestByRecordId
            }
        },
        index_realization: match manifest.realization.index_realization {
            IndexRealization::None => ResolvedIndexRealization::None,
            IndexRealization::Implicit => ResolvedIndexRealization::Implicit,
            IndexRealization::Sidecar => ResolvedIndexRealization::Sidecar,
        },
        scan_axes,
        source: InterpretationSource::Manifest,
    }
}

pub(crate) fn resolve_from_manifest_with_compatibility_inference(
    arrow_schema: &Schema,
    manifest: &DatasetSemanticManifest,
    visible_columns: &[usize],
    index_columns: Option<Vec<usize>>,
) -> DatasetInterpretation {
    let mut interpretation = resolve_from_manifest(arrow_schema, manifest, visible_columns);
    if manifest_allows_compatibility_index_projection(manifest) {
        apply_compatibility_index_projection(
            &mut interpretation,
            index_columns.unwrap_or_default(),
        );
    }
    interpretation
}

pub(crate) fn manifest_allows_compatibility_index_projection(
    manifest: &DatasetSemanticManifest,
) -> bool {
    manifest.compatibility.allow_inference
        && manifest.scan_plan.is_none()
        && manifest.columns.values().all(|column| {
            column.system.is_some()
                || (column.unit.is_none()
                    && column.label.is_none()
                    && !column.hidden_by_default
                    && !column.chart_axis)
        })
}

struct RoleProjections {
    semantic_references: Vec<ResolvedSemanticReference>,
    value_references: Vec<ResolvedSemanticReference>,
    plotted_coordinates: Vec<ResolvedSemanticReference>,
    sweep_axes: Vec<ResolvedSemanticReference>,
    group_axes: Vec<ResolvedSemanticReference>,
    filter_axes: Vec<ResolvedSemanticReference>,
    chart_axis_candidates: Vec<ResolvedSemanticReference>,
}

fn manifest_role_projections(
    columns: &[ResolvedColumn],
    scan_axes: &[ResolvedScanAxis],
) -> RoleProjections {
    let logical_index_references = scan_axes
        .iter()
        .map(ResolvedScanAxis::as_semantic_reference)
        .collect::<Vec<_>>();
    let value_references = physical_column_references(columns, false, |column| {
        column.meaning == ColumnMeaning::UserValue
    });
    let chart_axis_candidates = physical_column_references(columns, false, |column| {
        column.is_chart_axis_candidate && column.visible_ordinal.is_some()
    });
    let mut semantic_references = physical_column_references(columns, false, |_| true);
    semantic_references.extend(logical_index_references.clone());
    let mut plotted_coordinates = logical_index_references.clone();
    plotted_coordinates.extend(chart_axis_candidates.clone());
    let mut filter_axes = logical_index_references.clone();
    filter_axes.extend(chart_axis_candidates.clone());

    RoleProjections {
        semantic_references,
        value_references,
        plotted_coordinates,
        sweep_axes: logical_index_references.clone(),
        group_axes: logical_index_references,
        filter_axes,
        chart_axis_candidates,
    }
}

fn apply_compatibility_index_projection(
    interpretation: &mut DatasetInterpretation,
    index_columns: Vec<usize>,
) {
    let index_columns = index_columns
        .into_iter()
        .map(VisibleColumnOrdinal)
        .collect::<Vec<_>>();
    let index_column_set: HashSet<_> = index_columns.iter().copied().collect();

    for column in &mut interpretation.columns {
        if column.meaning == ColumnMeaning::UserValue
            && column
                .visible_ordinal
                .is_some_and(|ordinal| index_column_set.contains(&ordinal))
        {
            column.meaning = ColumnMeaning::CompatibilityIndex;
            column.is_index = true;
            column.is_chart_axis_candidate = true;
        }
    }

    let role_projections = manifest_with_compatibility_role_projections(&interpretation.columns);
    interpretation.semantic_references = role_projections.semantic_references;
    interpretation.value_references = role_projections.value_references;
    interpretation.plotted_coordinates = role_projections.plotted_coordinates;
    interpretation.sweep_axes = role_projections.sweep_axes;
    interpretation.group_axes = role_projections.group_axes;
    interpretation.filter_axes = role_projections.filter_axes;
    interpretation.chart_axis_candidates = role_projections.chart_axis_candidates;
    interpretation.value_columns = interpretation
        .columns
        .iter()
        .filter(|column| column.meaning == ColumnMeaning::UserValue)
        .filter_map(|column| column.visible_ordinal)
        .collect();
    interpretation.logical_index_columns = index_columns.clone();
    interpretation.chart_axis_candidate_columns = interpretation
        .columns
        .iter()
        .filter(|column| column.is_chart_axis_candidate)
        .filter_map(|column| column.visible_ordinal)
        .collect();
    if !index_columns.is_empty() {
        interpretation.duplicate_policy = ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder;
    }
}

fn manifest_with_compatibility_role_projections(columns: &[ResolvedColumn]) -> RoleProjections {
    let value_references = physical_column_references(columns, false, |column| {
        column.meaning == ColumnMeaning::UserValue
    });
    let compatibility_axes = physical_column_references(columns, true, |column| {
        column.meaning == ColumnMeaning::CompatibilityIndex
    });
    let chart_axis_candidates = physical_column_references(columns, false, |column| {
        column.meaning == ColumnMeaning::UserValue
            && column.is_chart_axis_candidate
            && column.visible_ordinal.is_some()
    });
    let semantic_references = physical_column_references(columns, false, |_| true);
    let mut plotted_coordinates = compatibility_axes.clone();
    plotted_coordinates.extend(chart_axis_candidates.clone());
    let mut filter_axes = compatibility_axes.clone();
    filter_axes.extend(chart_axis_candidates.clone());
    let mut all_chart_axis_candidates = compatibility_axes.clone();
    all_chart_axis_candidates.extend(chart_axis_candidates);

    RoleProjections {
        semantic_references,
        value_references,
        plotted_coordinates,
        sweep_axes: compatibility_axes.clone(),
        group_axes: compatibility_axes,
        filter_axes,
        chart_axis_candidates: all_chart_axis_candidates,
    }
}

fn physical_column_references(
    columns: &[ResolvedColumn],
    is_compatibility: bool,
    predicate: impl Fn(&ResolvedColumn) -> bool,
) -> Vec<ResolvedSemanticReference> {
    columns
        .iter()
        .filter(|column| predicate(column))
        .map(|column| column.as_semantic_reference(is_compatibility))
        .collect()
}

fn resolve_scan_axes(manifest: &DatasetSemanticManifest) -> Vec<ResolvedScanAxis> {
    manifest
        .scan_plan
        .as_ref()
        .map_or_else(Vec::new, |scan_plan| {
            scan_plan
                .axes
                .iter()
                .enumerate()
                .map(|(axis_ordinal, axis)| {
                    let mode = match &axis.mode {
                        ScanAxisMode::Static { values } => ResolvedScanAxisMode::Static {
                            values: values.clone(),
                        },
                        ScanAxisMode::ImplicitIndex => ResolvedScanAxisMode::ImplicitIndex,
                    };
                    ResolvedScanAxis {
                        id: logical_index_id(&axis.name),
                        axis_ordinal,
                        name: axis.name.clone(),
                        label: axis.label.clone(),
                        numeric_axis: scan_axis_mode_is_numeric(&mode),
                        mode,
                    }
                })
                .collect()
        })
}

pub(crate) fn resolve_logical_index_points(
    manifest: &DatasetSemanticManifest,
    record_ids: &[u64],
) -> Vec<ResolvedLogicalIndexPoint> {
    let Some(scan_plan) = &manifest.scan_plan else {
        return Vec::new();
    };
    if scan_plan.axes.is_empty() {
        return Vec::new();
    }
    if scan_plan
        .axes
        .iter()
        .any(|axis| matches!(axis.mode, ScanAxisMode::ImplicitIndex))
    {
        return record_ids
            .iter()
            .copied()
            .map(|record_id| ResolvedLogicalIndexPoint {
                record_id,
                indices: vec![record_id],
                coordinates: vec![ScanAxisValue::Int(
                    i64::try_from(record_id).unwrap_or(i64::MAX),
                )],
            })
            .collect();
    }

    let shapes: Vec<_> = scan_plan
        .axes
        .iter()
        .map(|axis| match &axis.mode {
            ScanAxisMode::Static { values } => values.len() as u64,
            ScanAxisMode::ImplicitIndex => 0,
        })
        .collect();
    let planned_len = shapes
        .iter()
        .try_fold(1_u64, |acc, len| acc.checked_mul(*len));
    if planned_len == Some(0) {
        return Vec::new();
    }

    record_ids
        .iter()
        .copied()
        .map(|record_id| {
            let mut remainder =
                planned_len.map_or(record_id, |planned_len| record_id % planned_len);
            let mut indices = vec![0; shapes.len()];
            for axis_index in (0..shapes.len()).rev() {
                let len = shapes[axis_index];
                indices[axis_index] = remainder % len;
                remainder /= len;
            }
            let coordinates = indices
                .iter()
                .zip(&scan_plan.axes)
                .map(|(index, axis)| {
                    let ScanAxisMode::Static { values } = &axis.mode else {
                        unreachable!("implicit axes handled earlier")
                    };
                    let index = usize::try_from(*index).expect("scan axis index should fit usize");
                    values[index].clone()
                })
                .collect();
            ResolvedLogicalIndexPoint {
                record_id,
                indices,
                coordinates,
            }
        })
        .collect()
}

pub(crate) fn resolve_from_compatibility_inference(
    schema: &DatasetSchema,
    index_columns: Option<Vec<usize>>,
) -> DatasetInterpretation {
    let index_columns = index_columns.unwrap_or_default();
    let index_columns: Vec<_> = index_columns
        .into_iter()
        .map(VisibleColumnOrdinal)
        .collect();
    let index_column_set: HashSet<_> = index_columns.iter().copied().collect();

    let columns: Vec<_> = schema
        .columns()
        .iter()
        .enumerate()
        .map(|(ordinal, (name, data_type))| {
            let visible_ordinal = VisibleColumnOrdinal(ordinal);
            let is_index = index_column_set.contains(&visible_ordinal);
            let dtype = dataset_dtype_from_compatibility_type(*data_type);
            ResolvedColumn {
                id: physical_column_id(name),
                name: name.to_owned(),
                physical_ordinal: PhysicalColumnOrdinal(ordinal),
                visible_ordinal: Some(visible_ordinal),
                dtype: dtype.clone(),
                meaning: if is_index {
                    ColumnMeaning::CompatibilityIndex
                } else {
                    ColumnMeaning::UserValue
                },
                is_index,
                is_system: false,
                hidden_by_default: false,
                is_chart_axis_candidate: is_index,
                unit: None,
                label: None,
                is_complex: dtype_is_complex(&dtype),
                is_trace: dtype_is_trace(&dtype),
                is_numeric_axis_candidate: dtype_is_chart_axis_numeric(&dtype),
            }
        })
        .collect();

    let value_columns = columns
        .iter()
        .filter(|column| column.meaning == ColumnMeaning::UserValue)
        .filter_map(|column| column.visible_ordinal)
        .collect();
    let role_projections = compatibility_role_projections(&columns);

    DatasetInterpretation {
        columns,
        semantic_references: role_projections.semantic_references,
        value_references: role_projections.value_references,
        plotted_coordinates: role_projections.plotted_coordinates,
        sweep_axes: role_projections.sweep_axes,
        group_axes: role_projections.group_axes,
        filter_axes: role_projections.filter_axes,
        chart_axis_candidates: role_projections.chart_axis_candidates,
        value_columns,
        logical_index_columns: index_columns.clone(),
        chart_axis_candidate_columns: index_columns,
        duplicate_policy: ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder,
        index_realization: ResolvedIndexRealization::None,
        scan_axes: Vec::new(),
        source: InterpretationSource::CompatibilityInference,
    }
}

fn compatibility_role_projections(columns: &[ResolvedColumn]) -> RoleProjections {
    let value_references = physical_column_references(columns, true, |column| {
        column.meaning == ColumnMeaning::UserValue
    });
    let compatibility_axes = physical_column_references(columns, true, |column| {
        column.meaning == ColumnMeaning::CompatibilityIndex
    });
    let semantic_references = physical_column_references(columns, true, |_| true);

    RoleProjections {
        semantic_references,
        value_references,
        plotted_coordinates: compatibility_axes.clone(),
        sweep_axes: compatibility_axes.clone(),
        group_axes: compatibility_axes.clone(),
        filter_axes: compatibility_axes.clone(),
        chart_axis_candidates: compatibility_axes,
    }
}

fn dataset_dtype_from_compatibility_type(data_type: DatasetDataType) -> DatasetDType {
    match data_type {
        DatasetDataType::Scalar(ScalarKind::Numeric) => DatasetDType::Float64,
        DatasetDataType::Scalar(ScalarKind::Complex) => DatasetDType::Complex128,
        DatasetDataType::Trace(trace_kind, scalar_kind) => DatasetDType::Trace {
            layout: trace_layout_from_compatibility_type(trace_kind),
            axis: TraceAxisDType::Float64,
            value: trace_value_from_compatibility_type(scalar_kind),
        },
    }
}

fn trace_layout_from_compatibility_type(trace_kind: TraceKind) -> TraceLayout {
    match trace_kind {
        TraceKind::Simple => TraceLayout::Simple,
        TraceKind::FixedStep => TraceLayout::FixedStep,
        TraceKind::VariableStep => TraceLayout::VariableStep,
    }
}

fn trace_value_from_compatibility_type(scalar_kind: ScalarKind) -> TraceValueDType {
    match scalar_kind {
        ScalarKind::Numeric => TraceValueDType::Float64,
        ScalarKind::Complex => TraceValueDType::Complex128,
    }
}

fn dtype_is_complex(dtype: &DatasetDType) -> bool {
    matches!(
        dtype,
        DatasetDType::Complex128
            | DatasetDType::Trace {
                value: TraceValueDType::Complex128,
                ..
            }
    )
}

fn dtype_is_trace(dtype: &DatasetDType) -> bool {
    matches!(dtype, DatasetDType::Trace { .. })
}

fn dtype_is_chart_axis_numeric(dtype: &DatasetDType) -> bool {
    matches!(
        dtype,
        DatasetDType::Float64 | DatasetDType::Float32 | DatasetDType::Int64 | DatasetDType::UInt64
    )
}

fn scan_axis_mode_is_numeric(mode: &ResolvedScanAxisMode) -> bool {
    match mode {
        ResolvedScanAxisMode::ImplicitIndex => true,
        ResolvedScanAxisMode::Static { values } => values
            .iter()
            .all(|value| matches!(value, ScanAxisValue::Int(_) | ScanAxisValue::Float(_))),
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Bound, sync::Arc};

    use arrow_array::{Float64Array, Int64Array, RecordBatch, StringArray, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};

    use super::{
        ColumnMeaning, InterpretationSource, PhysicalColumnOrdinal, ResolvedDuplicatePolicy,
        ResolvedIndexRealization, ResolvedLogicalIndexPoint, ResolvedScanAxisMode,
        ResolvedSemanticReference, VisibleColumnOrdinal, resolve_from_manifest,
        resolve_logical_index_points,
    };
    use crate::dataset::{
        DatasetReader,
        ingest::WriteSessionRegistry,
        interpret::resolve_from_compatibility_inference,
        read::{ReadError, SelectOptions},
        schema::DatasetSchema,
        semantics::{
            DatasetDType, DatasetSemanticManifest, IndexRealization, ManifestColumn,
            RECORD_ID_COLUMN, ScanAxis, ScanAxisValue, ScanPlan, write_manifest,
        },
        storage::{ChunkWriter, layout::ChunkKind, logical_index::logical_index_schema},
    };

    fn visible(indices: &[usize]) -> Vec<VisibleColumnOrdinal> {
        indices.iter().copied().map(VisibleColumnOrdinal).collect()
    }

    #[test]
    fn manifest_interpretation_marks_record_id_system_and_hidden() {
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )]);
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]);

        let interpretation = resolve_from_manifest(&schema, &manifest, &[1]);

        assert_eq!(interpretation.source, InterpretationSource::Manifest);
        assert_eq!(
            interpretation.duplicate_policy,
            ResolvedDuplicatePolicy::LatestByRecordId
        );
        assert_eq!(interpretation.value_columns, visible(&[0]));
        assert_eq!(
            interpretation.logical_index_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(
            interpretation.chart_axis_candidate_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );

        let record_id = &interpretation.columns[0];
        assert_eq!(record_id.id, "column:__ds_record_id");
        assert_eq!(record_id.name, RECORD_ID_COLUMN);
        assert_eq!(record_id.physical_ordinal, PhysicalColumnOrdinal(0));
        assert_eq!(record_id.visible_ordinal, None);
        assert_eq!(record_id.meaning, ColumnMeaning::SystemRecordId);
        assert_eq!(record_id.dtype, DatasetDType::UInt64);
        assert!(!record_id.is_index);
        assert!(record_id.is_system);
        assert!(record_id.hidden_by_default);
        assert!(!record_id.is_chart_axis_candidate);

        let signal = &interpretation.columns[1];
        assert_eq!(signal.id, "column:signal");
        assert_eq!(signal.physical_ordinal, PhysicalColumnOrdinal(1));
        assert_eq!(signal.visible_ordinal, Some(VisibleColumnOrdinal(0)));
        assert_eq!(signal.meaning, ColumnMeaning::UserValue);
        assert_eq!(signal.dtype, DatasetDType::Float64);
        assert!(!signal.is_system);
        assert!(!signal.hidden_by_default);
        assert_eq!(interpretation.value_references.len(), 1);
        assert_eq!(interpretation.value_references[0].id(), "column:signal");
        assert_eq!(
            interpretation
                .physical_column_for_id("column:signal")
                .expect("physical semantic reference")
                .name,
            "signal"
        );
    }

    #[test]
    fn manifest_interpretation_resolves_column_metadata() {
        let mut signal = ManifestColumn::new(DatasetDType::Float64);
        signal.unit = Some("V".to_string());
        signal.label = Some("Voltage".to_string());
        signal.hidden_by_default = true;
        signal.chart_axis = true;
        let manifest = DatasetSemanticManifest::minimal([("signal".to_string(), signal)]);
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]);

        let interpretation = resolve_from_manifest(&schema, &manifest, &[1]);
        let signal = &interpretation.columns[1];

        assert_eq!(signal.unit.as_deref(), Some("V"));
        assert_eq!(signal.label.as_deref(), Some("Voltage"));
        assert!(signal.hidden_by_default);
        assert!(signal.is_chart_axis_candidate);
        assert_eq!(interpretation.chart_axis_candidate_columns, visible(&[0]));
        assert_eq!(interpretation.chart_axis_candidates.len(), 1);
        assert_eq!(
            interpretation.chart_axis_candidates[0].id(),
            "column:signal"
        );
        assert!(interpretation.chart_axis_candidates[0].hidden_by_default());
    }

    #[test]
    fn manifest_interpretation_derives_regular_scan_indices_from_record_ids() {
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![
            ScanAxis::static_values(
                "gate",
                vec![ScanAxisValue::Float(-0.2), ScanAxisValue::Float(-0.1)],
            ),
            ScanAxis::static_values("bias", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
        ])));
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]);

        let interpretation = resolve_from_manifest(&schema, &manifest, &[1]);

        assert_eq!(
            interpretation.index_realization,
            ResolvedIndexRealization::Implicit
        );
        assert_eq!(interpretation.scan_axes.len(), 2);
        assert_eq!(interpretation.scan_axes[0].id, "logicalIndex:gate");
        assert_eq!(interpretation.sweep_axes.len(), 2);
        assert_eq!(interpretation.group_axes[0].id(), "logicalIndex:gate");
        assert_eq!(interpretation.filter_axes[1].id(), "logicalIndex:bias");
        assert_eq!(
            interpretation.plotted_coordinates[0].id(),
            "logicalIndex:gate"
        );
        assert!(
            interpretation
                .logical_axis_for_id("logicalIndex:gate")
                .expect("logical semantic reference")
                .numeric_axis
        );
        assert!(matches!(
            interpretation.scan_axes[0].mode,
            ResolvedScanAxisMode::Static { .. }
        ));
        let logical_index_points = resolve_logical_index_points(&manifest, &[0, 1, 2, 3, 4]);
        assert_eq!(
            logical_index_points,
            vec![
                ResolvedLogicalIndexPoint {
                    record_id: 0,
                    indices: vec![0, 0],
                    coordinates: vec![ScanAxisValue::Float(-0.2), ScanAxisValue::Int(0)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 1,
                    indices: vec![0, 1],
                    coordinates: vec![ScanAxisValue::Float(-0.2), ScanAxisValue::Int(1)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 2,
                    indices: vec![1, 0],
                    coordinates: vec![ScanAxisValue::Float(-0.1), ScanAxisValue::Int(0)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 3,
                    indices: vec![1, 1],
                    coordinates: vec![ScanAxisValue::Float(-0.1), ScanAxisValue::Int(1)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 4,
                    indices: vec![0, 0],
                    coordinates: vec![ScanAxisValue::Float(-0.2), ScanAxisValue::Int(0)],
                },
            ]
        );
    }

    #[test]
    fn implicit_scan_index_derivation_handles_shape_larger_than_u64() {
        let axes = (0..64)
            .map(|index| {
                ScanAxis::static_values(
                    format!("axis_{index}"),
                    vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)],
                )
            })
            .collect();
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(axes)));

        let points = resolve_logical_index_points(&manifest, &[u64::MAX]);

        assert_eq!(points.len(), 1);
        assert_eq!(points[0].indices, vec![1; 64]);
    }

    #[test]
    fn manifest_interpretation_derives_unknown_length_index_from_record_ids() {
        let manifest = DatasetSemanticManifest::minimal([(
            "loss".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![ScanAxis::implicit_index(
            "step", None,
        )])));
        let schema = Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("loss", DataType::Float64, false),
        ]);

        let interpretation = resolve_from_manifest(&schema, &manifest, &[1]);

        assert_eq!(interpretation.scan_axes[0].name, "step");
        assert!(matches!(
            interpretation.scan_axes[0].mode,
            ResolvedScanAxisMode::ImplicitIndex
        ));
        let logical_index_points = resolve_logical_index_points(&manifest, &[0, 1, 2]);
        assert_eq!(
            logical_index_points,
            vec![
                ResolvedLogicalIndexPoint {
                    record_id: 0,
                    indices: vec![0],
                    coordinates: vec![ScanAxisValue::Int(0)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 1,
                    indices: vec![1],
                    coordinates: vec![ScanAxisValue::Int(1)],
                },
                ResolvedLogicalIndexPoint {
                    record_id: 2,
                    indices: vec![2],
                    coordinates: vec![ScanAxisValue::Int(2)],
                },
            ]
        );
    }

    #[test]
    fn compatibility_interpretation_uses_legacy_index_columns() {
        let schema = DatasetSchema::try_from(&Schema::new(vec![
            Field::new("run", DataType::Float64, false),
            Field::new("step", DataType::Float64, false),
            Field::new("signal", DataType::Float64, false),
        ]))
        .expect("schema");

        let interpretation = resolve_from_compatibility_inference(&schema, Some(vec![0, 1]));

        assert_eq!(
            interpretation.source,
            InterpretationSource::CompatibilityInference
        );
        assert_eq!(
            interpretation.duplicate_policy,
            ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder
        );
        assert_eq!(interpretation.logical_index_columns, visible(&[0, 1]));
        assert_eq!(
            interpretation.chart_axis_candidate_columns,
            visible(&[0, 1])
        );
        assert_eq!(
            interpretation
                .sweep_axes
                .iter()
                .map(ResolvedSemanticReference::id)
                .collect::<Vec<_>>(),
            vec!["column:run", "column:step"]
        );
        assert_eq!(
            interpretation
                .chart_axis_candidates
                .iter()
                .map(ResolvedSemanticReference::id)
                .collect::<Vec<_>>(),
            vec!["column:run", "column:step"]
        );
        assert!(
            interpretation
                .sweep_axes
                .iter()
                .all(ResolvedSemanticReference::is_compatibility)
        );
        assert_eq!(interpretation.value_columns, visible(&[2]));
        assert_eq!(
            interpretation.columns[0].meaning,
            ColumnMeaning::CompatibilityIndex
        );
        assert!(interpretation.columns[0].is_index);
        assert!(interpretation.columns[0].is_chart_axis_candidate);
        assert_eq!(interpretation.columns[2].meaning, ColumnMeaning::UserValue);
        assert!(!interpretation.columns[2].is_index);
    }

    #[test]
    fn interpretation_ids_preserve_prefixed_physical_column_names() {
        let schema = DatasetSchema::try_from(&Schema::new(vec![
            Field::new("logicalIndex:gate", DataType::Float64, false),
            Field::new("column:value", DataType::Float64, false),
        ]))
        .expect("schema");

        let interpretation = resolve_from_compatibility_inference(&schema, Some(vec![0]));

        assert_eq!(interpretation.columns[0].id, "column:logicalIndex:gate");
        assert_eq!(interpretation.columns[1].id, "column:column:value");
        assert_eq!(
            interpretation
                .physical_column_for_id("column:logicalIndex:gate")
                .expect("prefixed physical column")
                .name,
            "logicalIndex:gate"
        );
        assert!(
            matches!(
                interpretation.semantic_reference("logicalIndex:gate"),
                None | Some(ResolvedSemanticReference::LogicalIndex(_))
            ),
            "a physical column whose name starts with logicalIndex: must stay in column namespace"
        );
    }

    #[test]
    fn compatibility_interpretation_handles_missing_inferred_indexes() {
        let schema = DatasetSchema::try_from(&Schema::new(vec![
            Field::new("run", DataType::Float64, false),
            Field::new("signal", DataType::Float64, false),
        ]))
        .expect("schema");

        let interpretation = resolve_from_compatibility_inference(&schema, None);

        assert_eq!(
            interpretation.logical_index_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(
            interpretation.chart_axis_candidate_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(interpretation.value_columns, visible(&[0, 1]));
        assert!(interpretation.columns.iter().all(|column| !column.is_index));
    }

    #[test]
    fn reader_interpretation_reads_manifest_when_present() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("run", DataType::Float64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1])),
                        Arc::new(Float64Array::from(vec![1.0, 2.0])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        write_manifest(
            dir.path(),
            &DatasetSemanticManifest::minimal([
                (
                    "run".to_string(),
                    ManifestColumn::new(DatasetDType::Float64),
                ),
                (
                    "signal".to_string(),
                    ManifestColumn::new(DatasetDType::Float64),
                ),
            ]),
        )
        .expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        assert_eq!(reader.schema().expect("visible schema").columns().len(), 2);
        assert_eq!(reader.arrow_schema().fields().len(), 2);
        assert_eq!(reader.batches()[0].num_columns(), 2);
        assert_eq!(
            reader.try_index_columns().expect("index columns"),
            Some(vec![0])
        );
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(interpretation.source, InterpretationSource::Manifest);
        assert_eq!(
            interpretation.duplicate_policy,
            ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder
        );
        assert_eq!(
            interpretation.columns[0].meaning,
            ColumnMeaning::SystemRecordId
        );
        assert_eq!(interpretation.logical_index_columns, visible(&[0]));
        assert_eq!(interpretation.value_columns, visible(&[1]));
        assert_eq!(
            interpretation.columns[1].physical_ordinal,
            PhysicalColumnOrdinal(1)
        );
        assert_eq!(
            interpretation.columns[1].visible_ordinal,
            Some(VisibleColumnOrdinal(0))
        );
        assert_eq!(
            interpretation.columns[1].meaning,
            ColumnMeaning::CompatibilityIndex
        );
        assert!(interpretation.columns[1].is_index);
        assert_eq!(
            interpretation
                .group_axes
                .iter()
                .map(ResolvedSemanticReference::id)
                .collect::<Vec<_>>(),
            vec!["column:run"]
        );
        assert!(interpretation.group_axes[0].is_compatibility());
        let selected_columns = interpretation
            .value_columns
            .iter()
            .map(|ordinal| ordinal.0)
            .collect();
        let (selected_schema, selected_batches) = reader
            .select_data(&SelectOptions {
                start: Bound::Unbounded,
                end: Bound::Unbounded,
                index_filters: None,
                selected_columns: Some(selected_columns),
            })
            .expect("select value columns from interpretation");
        assert_eq!(selected_schema.fields().len(), 1);
        assert_eq!(selected_batches[0].num_columns(), 1);
    }

    #[test]
    fn reader_interpretation_does_not_infer_indexes_for_metadata_manifest() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("run", DataType::Float64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1])),
                        Arc::new(Float64Array::from(vec![1.0, 2.0])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let mut run = ManifestColumn::new(DatasetDType::Float64);
        run.label = Some("Run".to_string());
        write_manifest(
            dir.path(),
            &DatasetSemanticManifest::minimal([
                ("run".to_string(), run),
                (
                    "signal".to_string(),
                    ManifestColumn::new(DatasetDType::Float64),
                ),
            ]),
        )
        .expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(
            interpretation.duplicate_policy,
            ResolvedDuplicatePolicy::LatestByRecordId
        );
        assert_eq!(
            interpretation.logical_index_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(interpretation.value_columns, visible(&[0, 1]));
        assert!(interpretation.columns.iter().all(|column| !column.is_index));
    }

    #[test]
    fn reader_interpretation_resolves_implicit_regular_scan_after_reopen() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1, 2, 3, 4])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0, 40.0, 50.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![
            ScanAxis::static_values(
                "gate",
                vec![
                    ScanAxisValue::String("low".to_string()),
                    ScanAxisValue::String("high".to_string()),
                ],
            ),
            ScanAxis::static_values("bias", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
        ])));
        write_manifest(dir.path(), &manifest).expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(
            interpretation.duplicate_policy,
            ResolvedDuplicatePolicy::LatestByRecordId
        );
        let logical_index_points = reader.logical_index_points().expect("logical index points");
        assert_eq!(
            logical_index_points
                .iter()
                .map(|point| point.indices.clone())
                .collect::<Vec<_>>(),
            vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1], vec![0, 0]]
        );
    }

    #[test]
    fn reader_interpretation_resolves_unknown_length_scan_after_reopen() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("loss", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1, 2])),
                        Arc::new(Float64Array::from(vec![3.0, 2.0, 1.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let manifest = DatasetSemanticManifest::minimal([(
            "loss".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(ScanPlan::new(vec![ScanAxis::implicit_index(
            "step", None,
        )])));
        write_manifest(dir.path(), &manifest).expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(interpretation.scan_axes[0].name, "step");
        let logical_index_points = reader.logical_index_points().expect("logical index points");
        assert_eq!(
            logical_index_points
                .iter()
                .map(|point| point.coordinates.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![ScanAxisValue::Int(0)],
                vec![ScanAxisValue::Int(1)],
                vec![ScanAxisValue::Int(2)]
            ]
        );
    }

    #[test]
    fn reader_interpretation_resolves_sidecar_indices_and_latest_duplicates() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1, 2])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let scan_plan = ScanPlan::new(vec![
            ScanAxis::static_values(
                "gate",
                vec![
                    ScanAxisValue::String("low".to_string()),
                    ScanAxisValue::String("high".to_string()),
                ],
            ),
            ScanAxis::static_values("bias", vec![ScanAxisValue::Int(0), ScanAxisValue::Int(1)]),
        ]);
        let mut manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(scan_plan.clone()));
        manifest.realization.index_realization = IndexRealization::Sidecar;
        write_manifest(dir.path(), &manifest).expect("write manifest");

        let mut sidecar_writer = ChunkWriter::new_with_kind(
            logical_index_schema(&scan_plan),
            dir.path().to_owned(),
            ChunkKind::LogicalIndex,
        );
        sidecar_writer
            .write(
                RecordBatch::try_new(
                    logical_index_schema(&scan_plan),
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1, 2])),
                        Arc::new(Int64Array::from(vec![1, 0, 1])),
                        Arc::new(Int64Array::from(vec![0, 1, 0])),
                    ],
                )
                .expect("sidecar batch"),
            )
            .expect("write sidecar");
        sidecar_writer.finish().expect("finish sidecar");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");
        assert_eq!(
            interpretation.index_realization,
            ResolvedIndexRealization::Sidecar
        );

        let logical_index_points = reader.logical_index_points().expect("logical index points");
        assert_eq!(
            logical_index_points
                .iter()
                .map(|point| (point.record_id, point.indices.clone()))
                .collect::<Vec<_>>(),
            vec![(0, vec![1, 0]), (1, vec![0, 1]), (2, vec![1, 0])]
        );
        assert_eq!(
            logical_index_points[2].coordinates,
            vec![
                ScanAxisValue::String("high".to_string()),
                ScanAxisValue::Int(0)
            ]
        );
    }

    #[test]
    fn reader_interpretation_rejects_missing_sidecar_indices() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let scan_plan = ScanPlan::new(vec![ScanAxis::static_values(
            "gate",
            vec![
                ScanAxisValue::String("low".to_string()),
                ScanAxisValue::String("high".to_string()),
            ],
        )]);
        let mut manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(scan_plan));
        manifest.realization.index_realization = IndexRealization::Sidecar;
        write_manifest(dir.path(), &manifest).expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let error = reader
            .logical_index_points()
            .expect_err("missing sidecar chunks should fail");

        assert!(matches!(error, ReadError::DatasetFs(_)));
    }

    #[test]
    fn reader_interpretation_rejects_partial_sidecar_record_ids() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1, 2])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        let scan_plan = ScanPlan::new(vec![ScanAxis::static_values(
            "gate",
            vec![
                ScanAxisValue::String("low".to_string()),
                ScanAxisValue::String("high".to_string()),
            ],
        )]);
        let mut manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )])
        .with_scan_plan(Some(scan_plan.clone()));
        manifest.realization.index_realization = IndexRealization::Sidecar;
        write_manifest(dir.path(), &manifest).expect("write manifest");

        let mut sidecar_writer = ChunkWriter::new_with_kind(
            logical_index_schema(&scan_plan),
            dir.path().to_owned(),
            ChunkKind::LogicalIndex,
        );
        sidecar_writer
            .write(
                RecordBatch::try_new(
                    logical_index_schema(&scan_plan),
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 2])),
                        Arc::new(Int64Array::from(vec![0, 1])),
                    ],
                )
                .expect("sidecar batch"),
            )
            .expect("write sidecar");
        sidecar_writer.finish().expect("finish sidecar");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let error = reader
            .logical_index_points()
            .expect_err("partial sidecar rows should fail");

        assert!(matches!(error, ReadError::Dataset(_)));
    }

    #[test]
    fn live_reader_interpretation_reads_manifest_when_present() {
        let dir = tempfile::tempdir().expect("temp dir");
        let user_schema = Arc::new(Schema::new(vec![Field::new(
            "signal",
            DataType::Float64,
            false,
        )]));
        let manifest = DatasetSemanticManifest::minimal([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )]);
        let registry = WriteSessionRegistry::new();
        let mut guard = registry.start_session(7, dir.path().to_owned(), &user_schema, None, false);
        guard
            .write_batch(
                &RecordBatch::try_new(user_schema, vec![Arc::new(Float64Array::from(vec![10.0]))])
                    .expect("batch"),
                None,
            )
            .expect("write batch");
        let handle = registry.get(7).expect("active handle");

        let reader =
            DatasetReader::from_handle(handle, Some(manifest), Some(dir.path().to_owned()))
                .expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(interpretation.source, InterpretationSource::Manifest);
        assert_eq!(
            interpretation.columns[0].meaning,
            ColumnMeaning::SystemRecordId
        );
        assert_eq!(interpretation.value_columns, visible(&[0]));
    }

    #[test]
    fn reader_interpretation_rejects_manifest_before_record_id_materialization() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![Field::new(
            "signal",
            DataType::Float64,
            false,
        )]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(schema, vec![Arc::new(Float64Array::from(vec![10.0, 20.0]))])
                    .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        write_manifest(
            dir.path(),
            &DatasetSemanticManifest::minimal([(
                "signal".to_string(),
                ManifestColumn::new(DatasetDType::Float64),
            )]),
        )
        .expect("write manifest");

        let Err(error) = DatasetReader::open_dir(dir.path()) else {
            panic!("reader should reject manifest-only record ids");
        };
        assert!(matches!(error, ReadError::Manifest(_)));
    }

    #[test]
    fn reader_interpretation_supports_manifest_dtypes_outside_legacy_schema() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("label", DataType::Utf8, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1])),
                        Arc::new(StringArray::from(vec!["first", "second"])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        write_manifest(
            dir.path(),
            &DatasetSemanticManifest::minimal([(
                "label".to_string(),
                ManifestColumn::new(DatasetDType::Utf8),
            )]),
        )
        .expect("write manifest");

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(interpretation.source, InterpretationSource::Manifest);
        assert_eq!(interpretation.value_columns, visible(&[0]));
        assert_eq!(interpretation.columns[1].dtype, DatasetDType::Utf8);
        assert_eq!(interpretation.columns[1].meaning, ColumnMeaning::UserValue);
    }

    #[test]
    fn reader_interpretation_reports_manifest_schema_mismatch() {
        let dir = tempfile::tempdir().expect("temp dir");
        let schema = Arc::new(Schema::new(vec![
            Field::new(RECORD_ID_COLUMN, DataType::UInt64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.path().to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(UInt64Array::from(vec![0, 1])),
                        Arc::new(Float64Array::from(vec![10.0, 20.0])),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
        write_manifest(
            dir.path(),
            &DatasetSemanticManifest::minimal([(
                "signal".to_string(),
                ManifestColumn::new(DatasetDType::Utf8),
            )]),
        )
        .expect("write manifest");

        let Err(error) = DatasetReader::open_dir(dir.path()) else {
            panic!("reader should reject manifest schema mismatch");
        };

        assert!(matches!(error, ReadError::Manifest(_)));
    }

    #[test]
    fn reader_interpretation_falls_back_to_legacy_index_inference() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_legacy_numeric_dataset(dir.path(), vec![1.0, 1.0], vec![0.0, 1.0]);

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(
            interpretation.source,
            InterpretationSource::CompatibilityInference
        );
        assert_eq!(interpretation.logical_index_columns, visible(&[0, 1]));
        assert_eq!(
            interpretation.chart_axis_candidate_columns,
            visible(&[0, 1])
        );
        assert_eq!(interpretation.value_columns, visible(&[2]));
        assert_eq!(
            interpretation.columns[0].meaning,
            ColumnMeaning::CompatibilityIndex
        );
        assert_eq!(
            interpretation.columns[1].meaning,
            ColumnMeaning::CompatibilityIndex
        );
        assert_eq!(interpretation.columns[2].meaning, ColumnMeaning::UserValue);
    }

    #[test]
    fn reader_interpretation_with_less_than_two_rows_has_no_legacy_indexes() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_legacy_numeric_dataset(dir.path(), vec![1.0], vec![0.0]);

        let reader = DatasetReader::open_dir(dir.path()).expect("reader");
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(
            interpretation.source,
            InterpretationSource::CompatibilityInference
        );
        assert_eq!(
            interpretation.logical_index_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(
            interpretation.chart_axis_candidate_columns,
            Vec::<VisibleColumnOrdinal>::new()
        );
        assert_eq!(interpretation.value_columns, visible(&[0, 1, 2]));
        assert!(interpretation.columns.iter().all(|column| !column.is_index));
    }

    fn write_legacy_numeric_dataset(dir: &std::path::Path, run: Vec<f64>, step: Vec<f64>) {
        let signal: Vec<f64> = step.iter().map(|value| value + 10.0).collect();
        let schema = Arc::new(Schema::new(vec![
            Field::new("run", DataType::Float64, false),
            Field::new("step", DataType::Float64, false),
            Field::new("signal", DataType::Float64, false),
        ]));
        let mut writer = ChunkWriter::new(schema.clone(), dir.to_owned());
        writer
            .write(
                RecordBatch::try_new(
                    schema,
                    vec![
                        Arc::new(Float64Array::from(run)),
                        Arc::new(Float64Array::from(step)),
                        Arc::new(Float64Array::from(signal)),
                    ],
                )
                .expect("batch"),
            )
            .expect("write batch");
        writer.finish().expect("finish writer");
    }
}

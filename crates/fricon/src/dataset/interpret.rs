mod model;

use std::collections::{HashMap, HashSet};

use arrow_schema::Schema;

pub use self::model::{
    ColumnMeaning, DatasetInterpretation, InterpretationSource, PhysicalColumnOrdinal,
    ResolvedColumn, ResolvedDuplicatePolicy, VisibleColumnOrdinal,
};
use crate::dataset::{
    schema::{DatasetDataType, DatasetSchema, ScalarKind, TraceKind},
    semantics::{
        DatasetDType, DatasetSemanticManifest, DuplicateResolutionDefault, SystemColumn,
        TraceAxisDType, TraceLayout, TraceValueDType,
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
            ResolvedColumn {
                name,
                physical_ordinal: PhysicalColumnOrdinal(physical_ordinal),
                visible_ordinal: visible_ordinals.get(&physical_ordinal).copied(),
                dtype: column.dtype.clone(),
                meaning: if is_record_id {
                    ColumnMeaning::SystemRecordId
                } else {
                    ColumnMeaning::UserValue
                },
                is_index: false,
                is_system: is_record_id,
                hidden_by_default: is_record_id,
                is_chart_axis_candidate: false,
            }
        })
        .collect();

    let value_columns = columns
        .iter()
        .filter(|column| column.meaning == ColumnMeaning::UserValue)
        .filter_map(|column| column.visible_ordinal)
        .collect();

    DatasetInterpretation {
        columns,
        value_columns,
        logical_index_columns: Vec::new(),
        chart_axis_candidate_columns: Vec::new(),
        duplicate_policy: match manifest.realization.duplicate_resolution_default {
            DuplicateResolutionDefault::LatestByRecordId => {
                ResolvedDuplicatePolicy::LatestByRecordIdPlaceholder
            }
        },
        source: InterpretationSource::Manifest,
    }
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
            ResolvedColumn {
                name: name.to_owned(),
                physical_ordinal: PhysicalColumnOrdinal(ordinal),
                visible_ordinal: Some(visible_ordinal),
                dtype: dataset_dtype_from_compatibility_type(*data_type),
                meaning: if is_index {
                    ColumnMeaning::CompatibilityIndex
                } else {
                    ColumnMeaning::UserValue
                },
                is_index,
                is_system: false,
                hidden_by_default: false,
                is_chart_axis_candidate: is_index,
            }
        })
        .collect();

    let value_columns = columns
        .iter()
        .filter(|column| column.meaning == ColumnMeaning::UserValue)
        .filter_map(|column| column.visible_ordinal)
        .collect();

    DatasetInterpretation {
        columns,
        value_columns,
        logical_index_columns: index_columns.clone(),
        chart_axis_candidate_columns: index_columns,
        duplicate_policy: ResolvedDuplicatePolicy::CompatibilityRowOrderPlaceholder,
        source: InterpretationSource::CompatibilityInference,
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

#[cfg(test)]
mod tests {
    use std::{ops::Bound, sync::Arc};

    use arrow_array::{Float64Array, RecordBatch, StringArray, UInt64Array};
    use arrow_schema::{DataType, Field, Schema};

    use super::{
        ColumnMeaning, InterpretationSource, PhysicalColumnOrdinal, ResolvedDuplicatePolicy,
        VisibleColumnOrdinal, resolve_from_manifest,
    };
    use crate::dataset::{
        DatasetReader,
        ingest::WriteSessionRegistry,
        interpret::resolve_from_compatibility_inference,
        read::{ReadError, SelectOptions},
        schema::DatasetSchema,
        semantics::{
            DatasetDType, DatasetSemanticManifest, ManifestColumn, RECORD_ID_COLUMN, write_manifest,
        },
        storage::ChunkWriter,
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
            ResolvedDuplicatePolicy::LatestByRecordIdPlaceholder
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
        assert_eq!(signal.physical_ordinal, PhysicalColumnOrdinal(1));
        assert_eq!(signal.visible_ordinal, Some(VisibleColumnOrdinal(0)));
        assert_eq!(signal.meaning, ColumnMeaning::UserValue);
        assert_eq!(signal.dtype, DatasetDType::Float64);
        assert!(!signal.is_system);
        assert!(!signal.hidden_by_default);
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
                        Arc::new(Float64Array::from(vec![1.0, 1.0])),
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
            Some(vec![0, 1])
        );
        let interpretation = reader.interpret().expect("interpretation");

        assert_eq!(interpretation.source, InterpretationSource::Manifest);
        assert_eq!(
            interpretation.columns[0].meaning,
            ColumnMeaning::SystemRecordId
        );
        assert_eq!(interpretation.value_columns, visible(&[0, 1]));
        assert_eq!(
            interpretation.columns[1].physical_ordinal,
            PhysicalColumnOrdinal(1)
        );
        assert_eq!(
            interpretation.columns[1].visible_ordinal,
            Some(VisibleColumnOrdinal(0))
        );
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
        assert_eq!(selected_schema.fields().len(), 2);
        assert_eq!(selected_batches[0].num_columns(), 2);
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
        let mut guard = registry.start_session(7, dir.path().to_owned(), &user_schema);
        guard
            .write_batch(
                &RecordBatch::try_new(user_schema, vec![Arc::new(Float64Array::from(vec![10.0]))])
                    .expect("batch"),
            )
            .expect("write batch");
        let handle = registry.get(7).expect("active handle");

        let reader = DatasetReader::from_handle(handle, Some(manifest)).expect("reader");
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

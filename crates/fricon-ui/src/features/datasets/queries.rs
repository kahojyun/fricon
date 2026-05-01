use fricon::{
    DatasetListQuery, InterpretationSource, ReadAppError, ResolvedColumn,
    dataset::{
        model::DatasetId,
        semantics::{DatasetDType, TraceValueDType},
    },
};

use super::{
    error::UiDatasetError,
    types::{ColumnInfo, DatasetDetail, DatasetInfo, DatasetWriteStatus},
};
use crate::desktop_runtime::session::WorkspaceSession;

pub(crate) fn validate_non_negative(
    value: Option<i64>,
    field_name: &str,
) -> Result<Option<i64>, UiDatasetError> {
    match value {
        Some(v) if v < 0 => Err(UiDatasetError::validation(format!(
            "{field_name} must be non-negative"
        ))),
        _ => Ok(value),
    }
}

pub(crate) async fn list_datasets(
    session: &WorkspaceSession,
    query: DatasetListQuery,
) -> Result<Vec<DatasetInfo>, UiDatasetError> {
    Ok(session
        .app()
        .list_datasets(query)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub(crate) async fn list_dataset_tags(
    session: &WorkspaceSession,
) -> Result<Vec<String>, UiDatasetError> {
    Ok(session.app().list_dataset_tags().await?)
}

pub(crate) async fn get_dataset_detail(
    session: &WorkspaceSession,
    id: i32,
) -> Result<DatasetDetail, UiDatasetError> {
    let record = session
        .app()
        .get_dataset_including_deleted(DatasetId::Id(id))
        .await?;
    let payload_available = record.metadata.deleted_at.is_none();
    let columns = if payload_available {
        let reader = session.dataset(id).await?;
        let interpretation = reader.interpret().map_err(ReadAppError::from)?;
        let expose_manifest_hints = interpretation.source == InterpretationSource::Manifest;
        interpretation
            .columns
            .iter()
            .filter_map(|column| column_info_from_resolved_column(column, expose_manifest_hints))
            .collect()
    } else {
        Vec::new()
    };

    Ok(DatasetDetail {
        id: record.id,
        name: record.metadata.name,
        description: record.metadata.description,
        favorite: record.metadata.favorite,
        tags: record.metadata.tags,
        status: record.metadata.status.into(),
        created_at: record.metadata.created_at,
        trashed_at: record.metadata.trashed_at,
        deleted_at: record.metadata.deleted_at,
        payload_available,
        columns,
    })
}

fn column_info_from_resolved_column(
    column: &ResolvedColumn,
    expose_manifest_hints: bool,
) -> Option<ColumnInfo> {
    column.visible_ordinal?;
    Some(ColumnInfo {
        name: column.name.clone(),
        label: column.label.clone(),
        unit: column.unit.clone(),
        is_complex: dtype_is_complex(&column.dtype),
        is_trace: matches!(column.dtype, DatasetDType::Trace { .. }),
        is_index: column.is_index,
        hidden_by_default: column.hidden_by_default,
        is_chart_axis_candidate: expose_manifest_hints && column.is_chart_axis_candidate,
    })
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

pub(crate) async fn get_dataset_write_status(
    session: &WorkspaceSession,
    id: i32,
) -> Result<DatasetWriteStatus, UiDatasetError> {
    let dataset = session.dataset(id).await?;
    let row_count = dataset.write_status();
    Ok(DatasetWriteStatus { row_count })
}

#[cfg(test)]
mod tests {
    use std::{fs::File, sync::Arc};

    use arrow_array::{Float64Array, RecordBatch};
    use arrow_ipc::writer::FileWriter;
    use arrow_schema::{DataType, Field, Schema};
    use fricon::{
        AppManager, Client, DatasetRow, DatasetScalar, ScalarArray, WorkspaceRoot,
        dataset::semantics::ColumnMetadata, workspace::WorkspacePaths,
    };
    use indexmap::IndexMap;
    use num::complex::Complex64;
    use tempfile::TempDir;

    use super::{get_dataset_detail, validate_non_negative};
    use crate::desktop_runtime::session::WorkspaceSession;

    #[test]
    fn validate_non_negative_rejects_negative_values() {
        let error = validate_non_negative(Some(-1), "limit").expect_err("expected error");
        assert_eq!(error.to_string(), "limit must be non-negative");
    }

    #[tokio::test]
    async fn get_dataset_detail_uses_manifest_interpretation_without_exposing_record_id()
    -> anyhow::Result<()> {
        let (_temp_dir, _app_manager, session, dataset_id) =
            create_manifest_dataset(manifest_rows()).await?;

        let detail = get_dataset_detail(&session, dataset_id).await?;

        assert_eq!(detail.columns.len(), 3);
        assert_eq!(detail.columns[0].name, "signal");
        assert_eq!(detail.columns[0].label.as_deref(), Some("Signal"));
        assert_eq!(detail.columns[0].unit.as_deref(), Some("V"));
        assert!(!detail.columns[0].is_index);
        assert!(!detail.columns[0].is_trace);
        assert!(!detail.columns[0].is_complex);
        assert!(detail.columns[0].hidden_by_default);
        assert!(detail.columns[0].is_chart_axis_candidate);
        assert_eq!(detail.columns[1].name, "trace");
        assert_eq!(detail.columns[1].label, None);
        assert_eq!(detail.columns[1].unit, None);
        assert!(!detail.columns[1].is_index);
        assert!(detail.columns[1].is_trace);
        assert!(!detail.columns[1].is_complex);
        assert!(!detail.columns[1].hidden_by_default);
        assert!(!detail.columns[1].is_chart_axis_candidate);
        assert_eq!(detail.columns[2].name, "complex");
        assert_eq!(detail.columns[2].label, None);
        assert_eq!(detail.columns[2].unit, None);
        assert!(!detail.columns[2].is_index);
        assert!(!detail.columns[2].is_trace);
        assert!(detail.columns[2].is_complex);
        assert!(!detail.columns[2].hidden_by_default);
        assert!(!detail.columns[2].is_chart_axis_candidate);
        assert!(
            !detail
                .columns
                .iter()
                .any(|column| column.name == "__ds_record_id")
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_dataset_detail_uses_legacy_interpretation_when_manifest_is_absent()
    -> anyhow::Result<()> {
        let (_temp_dir, _app_manager, session, dataset_id) = create_legacy_dataset().await?;

        let detail = get_dataset_detail(&session, dataset_id).await?;

        assert_eq!(detail.columns.len(), 3);
        assert_eq!(detail.columns[0].name, "run");
        assert_eq!(detail.columns[0].label, None);
        assert_eq!(detail.columns[0].unit, None);
        assert!(detail.columns[0].is_index);
        assert!(!detail.columns[0].is_trace);
        assert!(!detail.columns[0].is_complex);
        assert!(!detail.columns[0].hidden_by_default);
        assert!(!detail.columns[0].is_chart_axis_candidate);
        assert_eq!(detail.columns[1].name, "step");
        assert_eq!(detail.columns[1].label, None);
        assert_eq!(detail.columns[1].unit, None);
        assert!(detail.columns[1].is_index);
        assert!(!detail.columns[1].is_trace);
        assert!(!detail.columns[1].is_complex);
        assert!(!detail.columns[1].hidden_by_default);
        assert!(!detail.columns[1].is_chart_axis_candidate);
        assert_eq!(detail.columns[2].name, "value");
        assert_eq!(detail.columns[2].label, None);
        assert_eq!(detail.columns[2].unit, None);
        assert!(!detail.columns[2].is_index);
        assert!(!detail.columns[2].is_trace);
        assert!(!detail.columns[2].is_complex);
        assert!(!detail.columns[2].hidden_by_default);
        assert!(!detail.columns[2].is_chart_axis_candidate);

        Ok(())
    }

    async fn create_manifest_dataset(
        rows: Vec<DatasetRow>,
    ) -> anyhow::Result<(TempDir, AppManager, WorkspaceSession, i32)> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;

        let app_manager =
            AppManager::new_with_path(temp_dir.path())?.start(&tokio::runtime::Handle::current())?;
        let client = Client::connect(temp_dir.path()).await?;

        let schema = rows[0].to_schema();
        let mut writer = client
            .create_dataset(
                "manifest-detail-test".to_string(),
                String::new(),
                vec!["test".to_string()],
                schema,
                vec![ColumnMetadata {
                    name: "signal".to_string(),
                    label: Some("Signal".to_string()),
                    unit: Some("V".to_string()),
                    hidden_by_default: true,
                    chart_axis: true,
                }],
                None,
                false,
            )
            .await?;
        for row in rows {
            writer.write(row).await?;
        }
        let dataset = writer.finish().await?;
        let session = WorkspaceSession::new(app_manager.handle().clone());
        Ok((temp_dir, app_manager, session, dataset.id()))
    }

    async fn create_legacy_dataset() -> anyhow::Result<(TempDir, AppManager, WorkspaceSession, i32)>
    {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let record = session
            .app()
            .create_empty_dataset("legacy-detail-test".to_string(), String::new(), vec![])
            .await?;
        let dataset_dir =
            WorkspacePaths::new(temp_dir.path()).dataset_path_from_uid(record.metadata.uid);
        std::fs::remove_file(dataset_dir.join("dataset_manifest.json"))?;
        write_legacy_arrow_chunk(&dataset_dir)?;

        Ok((temp_dir, app_manager, session, record.id))
    }

    fn write_legacy_arrow_chunk(dataset_dir: &std::path::Path) -> anyhow::Result<()> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("run", DataType::Float64, false),
            Field::new("step", DataType::Float64, false),
            Field::new("value", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Float64Array::from(vec![1.0, 1.0])),
                Arc::new(Float64Array::from(vec![0.0, 1.0])),
                Arc::new(Float64Array::from(vec![10.0, 20.0])),
            ],
        )?;
        let mut writer = FileWriter::try_new(
            File::create(dataset_dir.join("data_chunk_0.arrow"))?,
            &schema,
        )?;
        writer.write(&batch)?;
        writer.finish()?;
        Ok(())
    }

    fn manifest_rows() -> Vec<DatasetRow> {
        vec![
            manifest_row(1.0, &[1.0, 2.0], Complex64::new(1.0, 2.0)),
            manifest_row(2.0, &[3.0, 4.0], Complex64::new(3.0, 4.0)),
        ]
    }

    fn manifest_row(signal: f64, trace: &[f64], complex: Complex64) -> DatasetRow {
        DatasetRow(IndexMap::from([
            ("signal".to_string(), DatasetScalar::Numeric(signal)),
            (
                "trace".to_string(),
                DatasetScalar::SimpleTrace(trace.iter().copied().collect::<ScalarArray>()),
            ),
            ("complex".to_string(), DatasetScalar::Complex(complex)),
        ]))
    }
}

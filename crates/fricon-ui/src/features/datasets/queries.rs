use anyhow::Context;
use fricon::{DatasetDataType, DatasetListQuery, dataset::model::DatasetId};

use super::types::{ColumnInfo, DatasetDetail, DatasetInfo, DatasetWriteStatus};
use crate::desktop_runtime::session::WorkspaceSession;

pub(crate) fn validate_non_negative(
    value: Option<i64>,
    field_name: &str,
) -> anyhow::Result<Option<i64>> {
    match value {
        Some(v) if v < 0 => anyhow::bail!("{field_name} must be non-negative"),
        _ => Ok(value),
    }
}

pub(crate) async fn list_datasets(
    session: &WorkspaceSession,
    query: DatasetListQuery,
) -> anyhow::Result<Vec<DatasetInfo>> {
    session
        .app()
        .list_datasets(query)
        .await
        .context("Failed to list datasets.")
        .map(|records| records.into_iter().map(Into::into).collect())
}

pub(crate) async fn list_dataset_tags(session: &WorkspaceSession) -> anyhow::Result<Vec<String>> {
    session
        .app()
        .list_dataset_tags()
        .await
        .context("Failed to list dataset tags.")
}

pub(crate) async fn get_dataset_detail(
    session: &WorkspaceSession,
    id: i32,
) -> anyhow::Result<DatasetDetail> {
    let record = session
        .app()
        .get_dataset(DatasetId::Id(id))
        .await
        .context("Failed to load dataset metadata.")?;
    let payload_available = record.metadata.deleted_at.is_none();
    let columns = if payload_available {
        let reader = session.dataset(id).await?;
        let schema = reader.schema();
        let index = reader.index_columns();
        schema
            .columns()
            .iter()
            .enumerate()
            .map(|(i, (name, data_type))| ColumnInfo {
                name: name.to_owned(),
                is_complex: data_type.is_complex(),
                is_trace: matches!(data_type, DatasetDataType::Trace(_, _)),
                is_index: index.as_ref().is_some_and(|index| index.contains(&i)),
            })
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

pub(crate) async fn get_dataset_write_status(
    session: &WorkspaceSession,
    id: i32,
) -> anyhow::Result<DatasetWriteStatus> {
    let dataset = session.dataset(id).await?;
    let (row_count, is_complete) = dataset.write_status();
    Ok(DatasetWriteStatus {
        row_count,
        is_complete,
    })
}

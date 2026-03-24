use fricon::{DatasetDataType, DatasetListQuery, dataset::model::DatasetId};

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
) -> Result<DatasetWriteStatus, UiDatasetError> {
    let dataset = session.dataset(id).await?;
    let (row_count, is_complete) = dataset.write_status();
    Ok(DatasetWriteStatus {
        row_count,
        is_complete,
    })
}

#[cfg(test)]
mod tests {
    use super::validate_non_negative;

    #[test]
    fn validate_non_negative_rejects_negative_values() {
        let error = validate_non_negative(Some(-1), "limit").expect_err("expected error");
        assert_eq!(error.to_string(), "limit must be non-negative");
    }
}

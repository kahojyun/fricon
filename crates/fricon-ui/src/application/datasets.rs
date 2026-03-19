use std::collections::BTreeSet;

use anyhow::Context;
use chrono::{DateTime, Utc};
use fricon::{
    DatasetDataType, DatasetListQuery, DatasetRecord, DatasetStatus, DatasetUpdate,
    dataset::model::DatasetId,
};
use serde::Serialize;

use crate::application::session::WorkspaceSession;

#[derive(Debug, Clone)]
pub(crate) struct ColumnInfo {
    pub(crate) name: String,
    pub(crate) is_complex: bool,
    pub(crate) is_trace: bool,
    pub(crate) is_index: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DatasetDetail {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) favorite: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) status: DatasetStatus,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DatasetWriteStatus {
    pub(crate) row_count: usize,
    pub(crate) is_complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DatasetDeleteResult {
    pub(crate) id: i32,
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DatasetInfoUpdate {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) favorite: Option<bool>,
    pub(crate) tags: Option<Vec<String>>,
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut unique = BTreeSet::new();
    for tag in tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_string());
        }
    }
    unique.into_iter().collect()
}

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
) -> anyhow::Result<Vec<DatasetRecord>> {
    session
        .app()
        .list_datasets(query)
        .await
        .context("Failed to list datasets.")
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
    let reader = session.dataset(id).await?;
    let schema = reader.schema();
    let index = reader.index_columns();
    let columns = schema
        .columns()
        .iter()
        .enumerate()
        .map(|(i, (name, data_type))| ColumnInfo {
            name: name.to_owned(),
            is_complex: data_type.is_complex(),
            is_trace: matches!(data_type, DatasetDataType::Trace(_, _)),
            is_index: index.as_ref().is_some_and(|index| index.contains(&i)),
        })
        .collect();

    Ok(DatasetDetail {
        id: record.id,
        name: record.metadata.name,
        description: record.metadata.description,
        favorite: record.metadata.favorite,
        tags: record.metadata.tags,
        status: record.metadata.status,
        created_at: record.metadata.created_at,
        columns,
    })
}

pub(crate) async fn update_dataset_favorite(
    session: &WorkspaceSession,
    id: i32,
    favorite: bool,
) -> anyhow::Result<()> {
    session
        .app()
        .update_dataset(
            id,
            DatasetUpdate {
                name: None,
                description: None,
                favorite: Some(favorite),
            },
        )
        .await
        .context("Failed to update dataset favorite status.")
}

pub(crate) async fn update_dataset_info(
    session: &WorkspaceSession,
    id: i32,
    update: DatasetInfoUpdate,
) -> anyhow::Result<()> {
    let app = session.app();

    let current = app
        .get_dataset(DatasetId::Id(id))
        .await
        .context("Failed to load current dataset metadata.")?;

    app.update_dataset(
        id,
        DatasetUpdate {
            name: update.name,
            description: update.description,
            favorite: update.favorite,
        },
    )
    .await
    .context("Failed to update dataset metadata.")?;

    if let Some(next_tags_raw) = update.tags {
        let next_tags = normalize_tags(next_tags_raw);
        let current_tags: BTreeSet<_> = current.metadata.tags.into_iter().collect();
        let next_tags_set: BTreeSet<_> = next_tags.into_iter().collect();

        let to_add: Vec<String> = next_tags_set.difference(&current_tags).cloned().collect();
        let to_remove: Vec<String> = current_tags.difference(&next_tags_set).cloned().collect();

        if !to_add.is_empty() {
            app.add_dataset_tags(id, to_add)
                .await
                .context("Failed to add dataset tags.")?;
        }

        if !to_remove.is_empty() {
            app.remove_dataset_tags(id, to_remove)
                .await
                .context("Failed to remove dataset tags.")?;
        }
    }

    Ok(())
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

pub(crate) async fn delete_datasets(
    session: &WorkspaceSession,
    ids: Vec<i32>,
) -> Vec<DatasetDeleteResult> {
    let app = session.app();
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        match app.delete_dataset(id).await {
            Ok(()) => results.push(DatasetDeleteResult {
                id,
                success: true,
                error: None,
            }),
            Err(e) => results.push(DatasetDeleteResult {
                id,
                success: false,
                error: Some(e.to_string()),
            }),
        }
    }
    results
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BatchTagUpdate {
    pub(crate) ids: Vec<i32>,
    pub(crate) add: Vec<String>,
    pub(crate) remove: Vec<String>,
}

pub(crate) async fn batch_update_dataset_tags(
    session: &WorkspaceSession,
    update: BatchTagUpdate,
) -> Vec<DatasetDeleteResult> {
    let app = session.app();
    let add = normalize_tags(update.add);
    let remove = normalize_tags(update.remove);
    let mut results = Vec::with_capacity(update.ids.len());
    for id in update.ids {
        let add_result = if add.is_empty() {
            Ok(())
        } else {
            app.add_dataset_tags(id, add.clone()).await
        };
        let remove_result = if remove.is_empty() {
            Ok(())
        } else {
            app.remove_dataset_tags(id, remove.clone()).await
        };
        let error = match (add_result, remove_result) {
            (Ok(()), Ok(())) => None,
            (Err(e), Ok(())) => Some(format!("add tags failed: {e}")),
            (Ok(()), Err(e)) => Some(format!("remove tags failed: {e}")),
            (Err(e1), Err(e2)) => Some(format!("add tags failed: {e1}; remove tags failed: {e2}")),
        };
        results.push(DatasetDeleteResult {
            id,
            success: error.is_none(),
            error,
        });
    }
    results
}

pub(crate) async fn delete_tag(session: &WorkspaceSession, tag: String) -> anyhow::Result<()> {
    session
        .app()
        .delete_tag(tag)
        .await
        .context("Failed to delete tag.")
}

pub(crate) async fn rename_tag(
    session: &WorkspaceSession,
    old_name: String,
    new_name: String,
) -> anyhow::Result<()> {
    session
        .app()
        .rename_tag(old_name, new_name)
        .await
        .context("Failed to rename tag.")
}

pub(crate) async fn merge_tag(
    session: &WorkspaceSession,
    source: String,
    target: String,
) -> anyhow::Result<()> {
    session
        .app()
        .merge_tag(source, target)
        .await
        .context("Failed to merge tag.")
}

#[cfg(test)]
mod tests {
    use fricon::{AppManager, DatasetListQuery, WorkspaceRoot};
    use tempfile::TempDir;

    use super::delete_datasets;
    use crate::application::session::WorkspaceSession;

    async fn create_completed_dataset(
        session: &WorkspaceSession,
        name: &str,
    ) -> anyhow::Result<i32> {
        let dataset = session
            .app()
            .create_empty_dataset(
                name.to_string(),
                "test dataset".to_string(),
                vec!["test".to_string()],
            )
            .await?;

        Ok(dataset.id)
    }

    #[tokio::test]
    async fn delete_datasets_reports_partial_success() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let existing_id = create_completed_dataset(&session, "delete-me").await?;
        let missing_id = existing_id + 10_000;

        let results = delete_datasets(&session, vec![existing_id, missing_id]).await;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, existing_id);
        assert!(results[0].success);
        assert_eq!(results[0].error, None);
        assert_eq!(results[1].id, missing_id);
        assert!(!results[1].success);
        assert!(results[1].error.is_some());

        let remaining = session
            .app()
            .list_datasets(DatasetListQuery::default())
            .await?;
        assert!(remaining.is_empty());

        Ok(())
    }
}

use std::collections::BTreeSet;

use fricon::{DatasetListQuery, DatasetUpdate, dataset::model::DatasetId};

use super::{
    error::UiDatasetError,
    types::{DatasetDeleteResult, DatasetInfoUpdate, DatasetOperationError, DatasetTagBatchResult},
};
use crate::{desktop_runtime::session::WorkspaceSession, tauri_api::ApiError};

#[derive(Debug, Clone, Default)]
pub(crate) struct BatchTagUpdate {
    pub(crate) ids: Vec<i32>,
    pub(crate) add: Vec<String>,
    pub(crate) remove: Vec<String>,
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

fn dataset_operation_error(error: &UiDatasetError) -> DatasetOperationError {
    ApiError::from_dataset_error(error).into_dataset_operation_error()
}

pub(crate) async fn update_dataset_favorite(
    session: &WorkspaceSession,
    id: i32,
    favorite: bool,
) -> Result<(), UiDatasetError> {
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
        .await?;
    Ok(())
}

pub(crate) async fn update_dataset_info(
    session: &WorkspaceSession,
    id: i32,
    update: DatasetInfoUpdate,
) -> Result<(), UiDatasetError> {
    let app = session.app();

    let current = app.get_dataset(DatasetId::Id(id)).await?;

    app.update_dataset(
        id,
        DatasetUpdate {
            name: update.name,
            description: update.description,
            favorite: update.favorite,
        },
    )
    .await?;

    if let Some(next_tags_raw) = update.tags {
        let next_tags = normalize_tags(next_tags_raw);
        let current_tags: BTreeSet<_> = current.metadata.tags.into_iter().collect();
        let next_tags_set: BTreeSet<_> = next_tags.into_iter().collect();

        let to_add: Vec<String> = next_tags_set.difference(&current_tags).cloned().collect();
        let to_remove: Vec<String> = current_tags.difference(&next_tags_set).cloned().collect();

        if !to_add.is_empty() {
            app.add_dataset_tags(id, to_add).await?;
        }

        if !to_remove.is_empty() {
            app.remove_dataset_tags(id, to_remove).await?;
        }
    }

    Ok(())
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
            Err(error) => {
                let error = UiDatasetError::from(error);
                results.push(DatasetDeleteResult {
                    id,
                    success: false,
                    error: Some(dataset_operation_error(&error)),
                });
            }
        }
    }
    results
}

pub(crate) async fn trash_datasets(
    session: &WorkspaceSession,
    ids: Vec<i32>,
) -> Vec<DatasetDeleteResult> {
    let app = session.app();
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        match app.trash_dataset(id).await {
            Ok(()) => results.push(DatasetDeleteResult {
                id,
                success: true,
                error: None,
            }),
            Err(error) => {
                let error = UiDatasetError::from(error);
                results.push(DatasetDeleteResult {
                    id,
                    success: false,
                    error: Some(dataset_operation_error(&error)),
                });
            }
        }
    }
    results
}

pub(crate) async fn restore_datasets(
    session: &WorkspaceSession,
    ids: Vec<i32>,
) -> Vec<DatasetDeleteResult> {
    let app = session.app();
    let mut results = Vec::with_capacity(ids.len());
    for id in ids {
        match app.restore_dataset(id).await {
            Ok(()) => results.push(DatasetDeleteResult {
                id,
                success: true,
                error: None,
            }),
            Err(error) => {
                let error = UiDatasetError::from(error);
                results.push(DatasetDeleteResult {
                    id,
                    success: false,
                    error: Some(dataset_operation_error(&error)),
                });
            }
        }
    }
    results
}

pub(crate) async fn empty_trash(
    session: &WorkspaceSession,
) -> Result<Vec<DatasetDeleteResult>, UiDatasetError> {
    let ids = session
        .app()
        .list_datasets(DatasetListQuery {
            trashed: Some(true),
            limit: Some(i64::MAX),
            offset: Some(0),
            ..DatasetListQuery::default()
        })
        .await?
        .into_iter()
        .map(|record| record.id)
        .collect();

    Ok(delete_datasets(session, ids).await)
}

pub(crate) async fn batch_update_dataset_tags(
    session: &WorkspaceSession,
    update: BatchTagUpdate,
) -> Vec<DatasetTagBatchResult> {
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
        let add_error = match add_result {
            Ok(()) => None,
            Err(error) => {
                let error = UiDatasetError::from(error);
                Some(dataset_operation_error(&error))
            }
        };
        let remove_error = match remove_result {
            Ok(()) => None,
            Err(error) => {
                let error = UiDatasetError::from(error);
                Some(dataset_operation_error(&error))
            }
        };
        results.push(DatasetTagBatchResult {
            id,
            success: add_error.is_none() && remove_error.is_none(),
            add_error,
            remove_error,
        });
    }
    results
}

pub(crate) async fn delete_tag(
    session: &WorkspaceSession,
    tag: String,
) -> Result<(), UiDatasetError> {
    session.app().delete_tag(tag).await?;
    Ok(())
}

pub(crate) async fn rename_tag(
    session: &WorkspaceSession,
    old_name: String,
    new_name: String,
) -> Result<(), UiDatasetError> {
    session.app().rename_tag(old_name, new_name).await?;
    Ok(())
}

pub(crate) async fn merge_tag(
    session: &WorkspaceSession,
    source: String,
    target: String,
) -> Result<(), UiDatasetError> {
    session.app().merge_tag(source, target).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use fricon::{AppManager, DatasetId, DatasetListQuery, WorkspaceRoot};
    use tempfile::TempDir;

    use super::{
        BatchTagUpdate, batch_update_dataset_tags, delete_datasets, empty_trash, restore_datasets,
        trash_datasets, update_dataset_info,
    };
    use crate::{
        desktop_runtime::session::WorkspaceSession, features::datasets::types::DatasetInfoUpdate,
        tauri_api::ApiErrorCode,
    };

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

        let trash_results = trash_datasets(&session, vec![existing_id]).await;
        assert_eq!(trash_results.len(), 1);
        assert!(trash_results[0].success);

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

        let deleted = session
            .app()
            .get_dataset(DatasetId::Id(existing_id))
            .await?;
        assert!(deleted.metadata.deleted_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn trash_and_restore_datasets_switches_between_active_and_trash_views()
    -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let dataset_id = create_completed_dataset(&session, "trash-me").await?;

        let trash_results = trash_datasets(&session, vec![dataset_id]).await;
        assert_eq!(trash_results.len(), 1);
        assert!(trash_results[0].success);

        let active_after_trash = session
            .app()
            .list_datasets(DatasetListQuery::default())
            .await?;
        assert!(active_after_trash.is_empty());

        let trashed_after_trash = session
            .app()
            .list_datasets(DatasetListQuery {
                trashed: Some(true),
                ..DatasetListQuery::default()
            })
            .await?;
        assert_eq!(trashed_after_trash.len(), 1);
        assert_eq!(trashed_after_trash[0].id, dataset_id);

        let restore_results = restore_datasets(&session, vec![dataset_id]).await;
        assert_eq!(restore_results.len(), 1);
        assert!(restore_results[0].success);

        let active_after_restore = session
            .app()
            .list_datasets(DatasetListQuery::default())
            .await?;
        assert_eq!(active_after_restore.len(), 1);
        assert_eq!(active_after_restore[0].id, dataset_id);

        let trashed_after_restore = session
            .app()
            .list_datasets(DatasetListQuery {
                trashed: Some(true),
                ..DatasetListQuery::default()
            })
            .await?;
        assert!(trashed_after_restore.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn empty_trash_tombstones_trashed_datasets() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let dataset_id = create_completed_dataset(&session, "purge-me").await?;
        let record = session.app().get_dataset(DatasetId::Id(dataset_id)).await?;
        let dataset_path = session
            .app()
            .paths()?
            .dataset_path_from_uid(record.metadata.uid);
        assert!(dataset_path.exists());

        let trash_results = trash_datasets(&session, vec![dataset_id]).await;
        assert_eq!(trash_results.len(), 1);
        assert!(trash_results[0].success);

        let delete_results = empty_trash(&session).await?;
        assert_eq!(delete_results.len(), 1);
        assert!(delete_results[0].success);
        assert!(!dataset_path.exists());

        let active = session
            .app()
            .list_datasets(DatasetListQuery::default())
            .await?;
        assert!(active.is_empty());

        let trashed = session
            .app()
            .list_datasets(DatasetListQuery {
                trashed: Some(true),
                ..DatasetListQuery::default()
            })
            .await?;
        assert!(trashed.is_empty());

        let tombstone = session.app().get_dataset(DatasetId::Id(dataset_id)).await?;
        assert!(tombstone.metadata.deleted_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn empty_trash_deletes_more_than_default_page_size() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let mut dataset_ids = Vec::new();
        for index in 0..205 {
            let dataset_id = create_completed_dataset(&session, &format!("trash-{index}")).await?;
            dataset_ids.push(dataset_id);
        }

        let trash_results = trash_datasets(&session, dataset_ids.clone()).await;
        assert_eq!(trash_results.len(), dataset_ids.len());
        assert!(trash_results.iter().all(|result| result.success));

        let delete_results = empty_trash(&session).await?;
        assert_eq!(delete_results.len(), dataset_ids.len());
        assert!(delete_results.iter().all(|result| result.success));

        let trashed = session
            .app()
            .list_datasets(DatasetListQuery {
                trashed: Some(true),
                limit: Some(i64::MAX),
                offset: Some(0),
                ..DatasetListQuery::default()
            })
            .await?;
        assert!(trashed.is_empty());

        for dataset_id in dataset_ids {
            let tombstone = session.app().get_dataset(DatasetId::Id(dataset_id)).await?;
            assert!(tombstone.metadata.deleted_at.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn deleted_datasets_still_allow_metadata_mutation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let dataset_id = create_completed_dataset(&session, "before-delete").await?;

        let trash_results = trash_datasets(&session, vec![dataset_id]).await;
        assert_eq!(trash_results.len(), 1);
        assert!(trash_results[0].success);

        let delete_results = delete_datasets(&session, vec![dataset_id]).await;
        assert_eq!(delete_results.len(), 1);
        assert!(delete_results[0].success);

        update_dataset_info(
            &session,
            dataset_id,
            DatasetInfoUpdate {
                name: Some("after-delete".to_string()),
                description: Some("updated tombstone".to_string()),
                favorite: Some(true),
                tags: Some(vec!["updated".to_string(), "retained".to_string()]),
            },
        )
        .await?;

        let add_tag_results = batch_update_dataset_tags(
            &session,
            BatchTagUpdate {
                ids: vec![dataset_id],
                add: vec!["extra".to_string()],
                remove: vec!["updated".to_string()],
            },
        )
        .await;
        assert_eq!(add_tag_results.len(), 1);
        assert!(add_tag_results[0].success);

        let tombstone = session.app().get_dataset(DatasetId::Id(dataset_id)).await?;
        assert_eq!(tombstone.metadata.name, "after-delete");
        assert_eq!(tombstone.metadata.description, "updated tombstone");
        assert!(tombstone.metadata.favorite);
        assert!(tombstone.metadata.deleted_at.is_some());
        assert_eq!(
            tombstone.metadata.tags,
            vec!["retained".to_string(), "extra".to_string()]
        );

        Ok(())
    }

    #[tokio::test]
    async fn deleted_datasets_cannot_be_restored() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());

        let dataset_id = create_completed_dataset(&session, "cannot-restore").await?;

        let trash_results = trash_datasets(&session, vec![dataset_id]).await;
        assert_eq!(trash_results.len(), 1);
        assert!(trash_results[0].success);

        let delete_results = delete_datasets(&session, vec![dataset_id]).await;
        assert_eq!(delete_results.len(), 1);
        assert!(delete_results[0].success);

        let restore_results = restore_datasets(&session, vec![dataset_id]).await;
        assert_eq!(restore_results.len(), 1);
        assert!(!restore_results[0].success);
        assert!(
            restore_results[0]
                .error
                .as_ref()
                .is_some_and(|error| error.code == ApiErrorCode::DatasetDeleted)
        );

        let tombstone = session.app().get_dataset(DatasetId::Id(dataset_id)).await?;
        assert!(tombstone.metadata.deleted_at.is_some());
        assert!(tombstone.metadata.trashed_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn batch_update_dataset_tags_reports_add_and_remove_errors_separately()
    -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        WorkspaceRoot::create_new(temp_dir.path())?;
        let app_manager = AppManager::new_with_path(temp_dir.path())?;
        let session = WorkspaceSession::new(app_manager.handle().clone());
        drop(app_manager);

        let results = batch_update_dataset_tags(
            &session,
            BatchTagUpdate {
                ids: vec![1],
                add: vec!["added".to_string()],
                remove: vec!["removed".to_string()],
            },
        )
        .await;

        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert!(
            results[0]
                .add_error
                .as_ref()
                .is_some_and(|error| error.code == ApiErrorCode::Internal)
        );
        assert!(
            results[0]
                .remove_error
                .as_ref()
                .is_some_and(|error| error.code == ApiErrorCode::Internal)
        );

        Ok(())
    }
}

use tracing::error;

use crate::dataset_catalog::{DatasetCatalogError, DatasetId, DatasetUpdate, tasks};

pub(super) fn update_dataset(
    conn: &mut diesel::SqliteConnection,
    id: i32,
    update: DatasetUpdate,
) -> Result<(), DatasetCatalogError> {
    tasks::do_update_dataset(conn, id, update)
}

pub(super) fn add_tags(
    conn: &mut diesel::SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    tasks::do_add_tags(conn, id, tags)
}

pub(super) fn remove_tags(
    conn: &mut diesel::SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    tasks::do_remove_tags(conn, id, tags)
}

pub(super) fn delete_dataset(
    database: &crate::database::Pool,
    root: &crate::workspace::WorkspaceRoot,
    id: i32,
) -> Result<(), DatasetCatalogError> {
    tasks::do_delete_dataset(database, root, id).inspect_err(|e| {
        error!(error = %e, dataset.id = id, "Dataset deletion failed");
    })
}

pub(super) fn reload_dataset(
    conn: &mut diesel::SqliteConnection,
    id: i32,
) -> Result<crate::dataset_catalog::DatasetRecord, DatasetCatalogError> {
    tasks::do_get_dataset(conn, DatasetId::Id(id))
}

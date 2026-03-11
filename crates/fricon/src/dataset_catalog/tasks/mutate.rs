use diesel::{SqliteConnection, prelude::*};
use tracing::{debug, info, instrument};

use crate::{
    database::{self, Pool, schema},
    dataset_catalog::{DatasetCatalogError, DatasetId, DatasetUpdate},
    storage,
    workspace::WorkspaceRoot,
};

#[instrument(skip(database, root), fields(dataset.id = id))]
pub(crate) fn do_delete_dataset(
    database: &Pool,
    root: &WorkspaceRoot,
    id: i32,
) -> Result<(), DatasetCatalogError> {
    let mut conn = database.get()?;
    let record = super::do_get_dataset(&mut conn, DatasetId::Id(id))?;
    let uid = record.metadata.uid;
    let dataset_path = root.paths().dataset_path_from_uid(uid);
    database::Dataset::delete_from_db(&mut conn, id)?;
    drop(conn);

    storage::delete_dataset(&dataset_path)?;
    info!(dataset.id = id, %uid, "Dataset deleted");

    Ok(())
}

#[instrument(skip(conn, update), fields(dataset.id = id))]
pub(crate) fn do_update_dataset(
    conn: &mut SqliteConnection,
    id: i32,
    update: DatasetUpdate,
) -> Result<(), DatasetCatalogError> {
    let db_update = database::DatasetUpdate {
        name: update.name,
        description: update.description,
        favorite: update.favorite,
        status: None,
    };
    database::Dataset::update_metadata(conn, id, &db_update)?;
    debug!(dataset.id = id, "Dataset metadata updated");
    Ok(())
}

#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_add_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    conn.immediate_transaction(|conn| {
        let created_tags = database::Tag::find_or_create_batch(conn, tags)?;
        let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

        database::DatasetTag::create_associations(conn, id, &tag_ids)?;
        Ok::<(), DatasetCatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags added to dataset");
    Ok(())
}

#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_remove_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), DatasetCatalogError> {
    conn.immediate_transaction(|conn| {
        let tag_ids_to_delete = schema::tags::table
            .filter(schema::tags::name.eq_any(tags))
            .select(schema::tags::id)
            .load::<i32>(conn)?;

        database::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
        Ok::<(), DatasetCatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags removed from dataset");
    Ok(())
}

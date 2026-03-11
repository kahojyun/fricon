use diesel::{SqliteConnection, prelude::*};
use tracing::{debug, info, instrument};

use super::query::do_get_dataset;
use crate::{
    dataset::{
        catalog::CatalogError,
        model::{DatasetId, DatasetUpdate},
        sqlite::{self, Pool, schema},
        storage,
    },
    workspace::WorkspacePaths,
};

#[instrument(skip(database, paths), fields(dataset.id = id))]
pub(crate) fn do_delete_dataset(
    database: &Pool,
    paths: &WorkspacePaths,
    id: i32,
) -> Result<(), CatalogError> {
    let mut conn = database.get()?;
    let record = do_get_dataset(&mut conn, DatasetId::Id(id))?;
    let uid = record.metadata.uid;
    let dataset_path = paths.dataset_path_from_uid(uid);
    sqlite::Dataset::delete_from_db(&mut conn, id)?;
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
) -> Result<(), CatalogError> {
    let db_update = sqlite::DatasetUpdate {
        name: update.name,
        description: update.description,
        favorite: update.favorite,
        status: None,
    };
    sqlite::Dataset::update_metadata(conn, id, &db_update)?;
    debug!(dataset.id = id, "Dataset metadata updated");
    Ok(())
}

#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_add_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), CatalogError> {
    conn.immediate_transaction(|conn| {
        let created_tags = sqlite::Tag::find_or_create_batch(conn, tags)?;
        let tag_ids: Vec<i32> = created_tags.into_iter().map(|tag| tag.id).collect();

        sqlite::DatasetTag::create_associations(conn, id, &tag_ids)?;
        Ok::<(), CatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags added to dataset");
    Ok(())
}

#[instrument(skip(conn, tags), fields(dataset.id = id, tags.count = tags.len()))]
pub(crate) fn do_remove_tags(
    conn: &mut SqliteConnection,
    id: i32,
    tags: &[String],
) -> Result<(), CatalogError> {
    conn.immediate_transaction(|conn| {
        let tag_ids_to_delete = schema::tags::table
            .filter(schema::tags::name.eq_any(tags))
            .select(schema::tags::id)
            .load::<i32>(conn)?;

        sqlite::DatasetTag::remove_associations(conn, id, &tag_ids_to_delete)?;
        Ok::<(), CatalogError>(())
    })?;
    debug!(dataset.id = id, ?tags, "Tags removed from dataset");
    Ok(())
}

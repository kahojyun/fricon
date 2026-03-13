use chrono::NaiveDateTime;
use diesel::{
    prelude::*,
    sqlite::{Sqlite, SqliteConnection},
};
use uuid::Uuid;

use super::types::{DbDatasetStatus, SimpleUuid};
use crate::database::schema;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::datasets, check_for_backend(Sqlite))]
pub(super) struct Dataset {
    pub(super) id: i32,
    pub(super) uid: SimpleUuid,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) favorite: bool,
    pub(super) status: DbDatasetStatus,
    pub(super) created_at: NaiveDateTime,
}

impl Dataset {
    pub(super) fn find_by_id(
        conn: &mut SqliteConnection,
        dataset_id: i32,
    ) -> QueryResult<Option<Self>> {
        use schema::datasets::dsl::datasets;

        datasets
            .find(dataset_id)
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    pub(super) fn find_by_uid(
        conn: &mut SqliteConnection,
        dataset_uid: Uuid,
    ) -> QueryResult<Option<Self>> {
        use schema::datasets::dsl::{datasets, uid};

        datasets
            .filter(uid.eq(SimpleUuid(dataset_uid)))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    pub(super) fn update_status(
        conn: &mut SqliteConnection,
        dataset_id: i32,
        new_status: DbDatasetStatus,
    ) -> QueryResult<usize> {
        use schema::datasets::dsl::{datasets, status};

        diesel::update(datasets.find(dataset_id))
            .set(status.eq(new_status))
            .execute(conn)
    }

    pub(super) fn update_metadata(
        conn: &mut SqliteConnection,
        dataset_id: i32,
        update: &DatasetUpdate,
    ) -> QueryResult<usize> {
        use schema::datasets::dsl::datasets;

        diesel::update(datasets.find(dataset_id))
            .set(update)
            .execute(conn)
    }

    pub(super) fn delete_from_db(
        conn: &mut SqliteConnection,
        dataset_id: i32,
    ) -> QueryResult<usize> {
        use schema::datasets::dsl::datasets;

        diesel::delete(datasets.find(dataset_id)).execute(conn)
    }

    pub(super) fn load_tags(&self, conn: &mut SqliteConnection) -> QueryResult<Vec<Tag>> {
        DatasetTag::belonging_to(self)
            .inner_join(schema::tags::table)
            .select(Tag::as_select())
            .load(conn)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = schema::datasets)]
pub(super) struct DatasetUpdate {
    pub(super) name: Option<String>,
    pub(super) description: Option<String>,
    pub(super) favorite: Option<bool>,
    pub(super) status: Option<DbDatasetStatus>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::datasets)]
pub(super) struct NewDataset<'a> {
    pub(super) uid: SimpleUuid,
    pub(super) name: &'a str,
    pub(super) description: &'a str,
    pub(super) status: DbDatasetStatus,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::tags, check_for_backend(Sqlite))]
pub(super) struct Tag {
    pub(super) id: i32,
    pub(super) name: String,
}

impl Tag {
    pub(super) fn find_or_create_batch(
        conn: &mut SqliteConnection,
        names: &[String],
    ) -> QueryResult<Vec<Self>> {
        use schema::tags::dsl::{name, tags};

        let new_tags: Vec<_> = names
            .iter()
            .map(|tag_name| NewTag { name: tag_name })
            .collect();
        diesel::insert_or_ignore_into(tags)
            .values(new_tags)
            .execute(conn)?;

        tags.filter(name.eq_any(names))
            .select(Self::as_select())
            .load(conn)
    }

    /// Find a tag by name, returning `None` if it does not exist.
    pub(super) fn find_by_name(
        conn: &mut SqliteConnection,
        tag_name: &str,
    ) -> QueryResult<Option<Self>> {
        use schema::tags::dsl::{name, tags};
        tags.filter(name.eq(tag_name))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// Delete all dataset associations for this tag, then delete the tag row
    /// itself.
    pub(super) fn delete_by_name(conn: &mut SqliteConnection, tag_name: &str) -> QueryResult<()> {
        use schema::{
            datasets_tags::dsl::{datasets_tags, tag_id},
            tags::dsl::{name, tags},
        };

        let Some(tag) = Self::find_by_name(conn, tag_name)? else {
            return Ok(());
        };
        // Remove all dataset associations first.
        diesel::delete(datasets_tags.filter(tag_id.eq(tag.id))).execute(conn)?;
        // Then remove the tag row.
        diesel::delete(tags.filter(name.eq(tag_name))).execute(conn)?;
        Ok(())
    }

    /// Rename a tag. Returns an error if `new_name` already exists.
    pub(super) fn rename(
        conn: &mut SqliteConnection,
        old_name: &str,
        new_name: &str,
    ) -> QueryResult<()> {
        use schema::tags::dsl::{name, tags};
        diesel::update(tags.filter(name.eq(old_name)))
            .set(name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    /// Merge `source` tag into `target` tag:
    /// re-points all `datasets_tags` rows from source to target (skipping
    /// duplicates), then deletes the source tag row.
    pub(super) fn merge_into(
        conn: &mut SqliteConnection,
        source_name: &str,
        target_name: &str,
    ) -> QueryResult<()> {
        use schema::{
            datasets_tags::dsl::{dataset_id, datasets_tags, tag_id},
            tags::dsl::{name, tags},
        };

        let Some(source) = Self::find_by_name(conn, source_name)? else {
            return Ok(());
        };
        // Ensure target exists (create if not).
        let target_names = vec![target_name.to_owned()];
        let target_vec = Tag::find_or_create_batch(conn, &target_names)?;
        let target = &target_vec[0];

        // Dataset IDs that already have the target tag — we must not insert duplicates.
        let already_tagged: Vec<i32> = datasets_tags
            .filter(tag_id.eq(target.id))
            .select(dataset_id)
            .load(conn)?;

        // Move source-only rows to target.
        diesel::update(
            datasets_tags
                .filter(tag_id.eq(source.id))
                .filter(dataset_id.ne_all(&already_tagged)),
        )
        .set(tag_id.eq(target.id))
        .execute(conn)?;

        // Delete any remaining source rows (duplicates that already had target).
        diesel::delete(datasets_tags.filter(tag_id.eq(source.id))).execute(conn)?;

        // Delete the source tag row.
        diesel::delete(tags.filter(name.eq(source_name))).execute(conn)?;
        Ok(())
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::tags)]
struct NewTag<'a> {
    name: &'a str,
}

#[derive(Debug, Queryable, Insertable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Dataset), belongs_to(Tag))]
#[diesel(primary_key(dataset_id, tag_id))]
#[diesel(table_name = schema::datasets_tags, check_for_backend(Sqlite))]
pub(super) struct DatasetTag {
    dataset_id: i32,
    tag_id: i32,
}

impl DatasetTag {
    pub(super) fn create_associations(
        conn: &mut SqliteConnection,
        ds_id: i32,
        tag_ids: &[i32],
    ) -> QueryResult<Vec<Self>> {
        use schema::datasets_tags::dsl::datasets_tags;

        let new_associations: Vec<_> = tag_ids
            .iter()
            .map(|&tag_id_val| DatasetTag {
                dataset_id: ds_id,
                tag_id: tag_id_val,
            })
            .collect();

        diesel::insert_or_ignore_into(datasets_tags)
            .values(&new_associations)
            .execute(conn)?;

        Ok(new_associations)
    }

    pub(super) fn remove_associations(
        conn: &mut SqliteConnection,
        ds_id: i32,
        tag_ids: &[i32],
    ) -> QueryResult<usize> {
        use schema::datasets_tags::dsl::{dataset_id, datasets_tags, tag_id};

        diesel::delete(datasets_tags)
            .filter(dataset_id.eq(ds_id))
            .filter(tag_id.eq_any(tag_ids))
            .execute(conn)
    }
}

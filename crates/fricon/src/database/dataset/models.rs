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

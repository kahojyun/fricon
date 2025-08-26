use chrono::NaiveDateTime;
use diesel::{
    prelude::*,
    sqlite::{Sqlite, SqliteConnection},
};
use uuid::Uuid;

use super::{DatasetStatus, JsonValue, SimpleUuid, schema};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::datasets, check_for_backend(Sqlite))]
pub struct Dataset {
    pub id: i32,
    pub uuid: SimpleUuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub status: DatasetStatus,
    pub index_columns: JsonValue<Vec<String>>,
    pub created_at: NaiveDateTime,
}

impl Dataset {
    /// Find dataset by ID
    pub fn find_by_id(conn: &mut SqliteConnection, dataset_id: i32) -> QueryResult<Option<Self>> {
        use schema::datasets::dsl::datasets;

        datasets
            .find(dataset_id)
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// Find dataset by UUID
    pub fn find_by_uuid(
        conn: &mut SqliteConnection,
        dataset_uuid: Uuid,
    ) -> QueryResult<Option<Self>> {
        use schema::datasets::dsl::{datasets, uuid};

        datasets
            .filter(uuid.eq(SimpleUuid(dataset_uuid)))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// List all datasets ordered by ID descending
    pub fn list_all_ordered(conn: &mut SqliteConnection) -> QueryResult<Vec<Self>> {
        use schema::datasets::dsl::{datasets, id};

        datasets
            .order(id.desc())
            .select(Self::as_select())
            .load(conn)
    }

    /// Update dataset status
    pub fn update_status(
        conn: &mut SqliteConnection,
        dataset_id: i32,
        new_status: DatasetStatus,
    ) -> QueryResult<usize> {
        use schema::datasets::dsl::{datasets, status};

        diesel::update(datasets.find(dataset_id))
            .set(status.eq(new_status))
            .execute(conn)
    }

    /// Update dataset metadata
    pub fn update_metadata(
        conn: &mut SqliteConnection,
        dataset_id: i32,
        update: &DatasetUpdate,
    ) -> QueryResult<usize> {
        use schema::datasets::dsl::datasets;

        diesel::update(datasets.find(dataset_id))
            .set(update)
            .execute(conn)
    }

    /// Delete dataset from database
    pub fn delete_from_db(conn: &mut SqliteConnection, dataset_id: i32) -> QueryResult<usize> {
        use schema::datasets::dsl::datasets;

        diesel::delete(datasets.find(dataset_id)).execute(conn)
    }

    /// Load tags associated with this dataset
    pub fn load_tags(&self, conn: &mut SqliteConnection) -> QueryResult<Vec<Tag>> {
        DatasetTag::belonging_to(self)
            .inner_join(schema::tags::table)
            .select(Tag::as_select())
            .load(conn)
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = schema::datasets)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
    pub status: Option<DatasetStatus>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::datasets)]
pub struct NewDataset<'a> {
    pub uuid: SimpleUuid,
    pub name: &'a str,
    pub description: &'a str,
    pub status: DatasetStatus,
    pub index_columns: JsonValue<&'a [String]>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::tags, check_for_backend(Sqlite))]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

impl Tag {
    /// Find tag by name
    pub fn find_by_name(conn: &mut SqliteConnection, tag_name: &str) -> QueryResult<Option<Self>> {
        use schema::tags::dsl::{name, tags};

        tags.filter(name.eq(tag_name))
            .select(Self::as_select())
            .first(conn)
            .optional()
    }

    /// Find or create tags in batch
    pub fn find_or_create_batch(
        conn: &mut SqliteConnection,
        names: &[String],
    ) -> QueryResult<Vec<Self>> {
        use schema::tags::dsl::{name, tags};

        // Insert new tags (ignore duplicates)
        let new_tags: Vec<_> = names
            .iter()
            .map(|tag_name| NewTag { name: tag_name })
            .collect();
        diesel::insert_or_ignore_into(tags)
            .values(new_tags)
            .execute(conn)?;

        // Return all requested tags
        tags.filter(name.eq_any(names))
            .select(Self::as_select())
            .load(conn)
    }

    /// Create a new tag
    pub fn create_new(conn: &mut SqliteConnection, tag_name: &str) -> QueryResult<Self> {
        use schema::tags::dsl::tags;

        let new_tag = NewTag { name: tag_name };
        diesel::insert_into(tags)
            .values(new_tag)
            .returning(Self::as_returning())
            .get_result(conn)
    }

    /// Get all datasets associated with this tag
    pub fn datasets(&self, conn: &mut SqliteConnection) -> QueryResult<Vec<Dataset>> {
        use schema::{datasets, datasets_tags};

        datasets_tags::table
            .filter(datasets_tags::tag_id.eq(self.id))
            .inner_join(datasets::table)
            .select(Dataset::as_select())
            .load(conn)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
}

#[derive(Debug, Queryable, Insertable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Dataset), belongs_to(Tag))]
#[diesel(primary_key(dataset_id, tag_id))]
#[diesel(table_name = schema::datasets_tags, check_for_backend(Sqlite))]
pub struct DatasetTag {
    pub dataset_id: i32,
    pub tag_id: i32,
}

impl DatasetTag {
    /// Create associations between dataset and multiple tags
    pub fn create_associations(
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

        // Return the associations that were requested
        Ok(new_associations)
    }

    /// Remove associations between dataset and tags
    pub fn remove_associations(
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

    /// Find all dataset-tag associations for a given dataset
    pub fn find_by_dataset(conn: &mut SqliteConnection, ds_id: i32) -> QueryResult<Vec<Self>> {
        use schema::datasets_tags::dsl::{dataset_id, datasets_tags};

        datasets_tags
            .filter(dataset_id.eq(ds_id))
            .select(Self::as_select())
            .load(conn)
    }
}

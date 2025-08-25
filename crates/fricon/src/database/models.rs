use chrono::NaiveDateTime;
use diesel::{prelude::*, sqlite::Sqlite};

use super::{JsonValue, SimpleUuid, schema};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::datasets, check_for_backend(Sqlite))]
pub struct Dataset {
    pub id: i32,
    pub uuid: SimpleUuid,
    pub name: String,
    pub description: String,
    pub favorite: bool,
    pub index_columns: JsonValue<Vec<String>>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = schema::datasets)]
pub struct DatasetUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub favorite: Option<bool>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::datasets)]
pub struct NewDataset<'a> {
    pub uuid: SimpleUuid,
    pub name: &'a str,
    pub description: &'a str,
    pub index_columns: JsonValue<&'a [String]>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::tags, check_for_backend(Sqlite))]
pub struct Tag {
    pub id: i32,
    pub name: String,
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

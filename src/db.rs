use chrono::NaiveDateTime;
use sqlx::{migrate::Migrator, SqliteConnection};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Sqlx error: {0}")]
    Other(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug)]
pub struct DatasetRecord {
    pub name: String,
    pub description: String,
    pub path: String,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
}

pub async fn create(
    uid: Uuid,
    name: &str,
    description: &str,
    path: &str,
    tags: &[String],
    pool: &sqlx::SqlitePool,
) -> Result<i64> {
    let mut tx = pool.begin().await?;
    let dataset_id = tx.insert_dataset(uid, name, description, path).await?;
    for tag in tags {
        let tag_id = tx.get_or_insert_tag(tag).await?;
        tx.add_tag_to_dataset(dataset_id, tag_id).await?;
    }
    tx.commit().await?;
    Ok(dataset_id)
}

pub async fn fetch_by_uid(uid: Uuid, pool: &sqlx::SqlitePool) -> Result<DatasetRecord> {
    let mut tx = pool.begin().await?;
    let id = tx.find_dataset_by_uid(uid).await?;
    tx.fetch_dataset_by_id(id).await
}

trait StorageDbExt {
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64>;
    async fn insert_dataset(
        &mut self,
        uid: Uuid,
        name: &str,
        description: &str,
        path: &str,
    ) -> Result<i64>;
    async fn find_dataset_by_uid(&mut self, uid: Uuid) -> Result<i64>;
    async fn add_tag_to_dataset(&mut self, dataset_id: i64, tag_id: i64) -> Result<()>;
    async fn fetch_dataset_by_id(&mut self, id: i64) -> Result<DatasetRecord>;
}

impl StorageDbExt for SqliteConnection {
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64> {
        let res = sqlx::query!("SELECT id FROM tags WHERE name = ?", tag)
            .fetch_optional(&mut *self)
            .await?;
        if let Some(r) = res {
            return Ok(r.id);
        }
        let tag_id = sqlx::query!("INSERT INTO tags (name) VALUES (?) RETURNING id", tag)
            .fetch_one(&mut *self)
            .await?
            .id;
        Ok(tag_id)
    }

    async fn insert_dataset(
        &mut self,
        uid: Uuid,
        name: &str,
        description: &str,
        path: &str,
    ) -> Result<i64> {
        let id = sqlx::query!(
            "INSERT INTO datasets (uid, name, description, path) VALUES (?, ?, ?, ?) RETURNING id",
            uid,
            name,
            description,
            path
        )
        .fetch_one(&mut *self)
        .await?
        .id;
        Ok(id)
    }

    async fn add_tag_to_dataset(&mut self, dataset_id: i64, tag_id: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO dataset_tag (dataset_id, tag_id) VALUES (?, ?)",
            dataset_id,
            tag_id
        )
        .execute(&mut *self)
        .await?;
        Ok(())
    }

    async fn find_dataset_by_uid(&mut self, uid: Uuid) -> Result<i64> {
        sqlx::query!("SELECT id FROM datasets WHERE uid = ?", uid)
            .fetch_optional(&mut *self)
            .await?
            .map(|r| r.id)
            .ok_or(Error::NotFound)
    }

    async fn fetch_dataset_by_id(&mut self, id: i64) -> Result<DatasetRecord> {
        let res = sqlx::query!(
            r#"
            SELECT
            name, description, path, created_at
            FROM
            datasets
            WHERE
            id = ?
            "#,
            id
        )
        .fetch_one(&mut *self)
        .await?;
        let tags = sqlx::query_scalar!(
            r#"
            SELECT
            t.name
            FROM
            tags t
            JOIN dataset_tag dt ON t.id = dt.tag_id
            WHERE
            dt.dataset_id = ?
            "#,
            id
        )
        .fetch_all(&mut *self)
        .await?;
        Ok(DatasetRecord {
            name: res.name,
            description: res.description,
            path: res.path,
            tags,
            created_at: res.created_at,
        })
    }
}

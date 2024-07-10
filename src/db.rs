use sqlx::SqliteConnection;
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

pub async fn create(
    uid: Uuid,
    name: &str,
    description: Option<&str>,
    tags: &[String],
    pool: &sqlx::SqlitePool,
) -> Result<i64> {
    let mut tx = pool.begin().await?;
    let dataset_id = tx.insert_dataset(uid, name, description).await?;
    for tag in tags {
        let tag_id = tx.get_or_insert_tag(tag).await?;
        tx.add_tag_to_dataset(dataset_id, tag_id).await?;
    }
    tx.commit().await?;
    Ok(dataset_id)
}

pub async fn find_by_uid(uid: Uuid, pool: &sqlx::SqlitePool) -> Result<i64> {
    let mut conn = pool.acquire().await?;
    conn.find_dataset_by_uid(uid).await
}

pub async fn update(
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
    pool: &sqlx::SqlitePool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    tx.update_dataset(id, name, description).await?;
    if let Some(tags) = tags {
        tx.clear_dataset_tags(id).await?;
        for tag in tags {
            let tag_id = tx.get_or_insert_tag(tag).await?;
            tx.add_tag_to_dataset(id, tag_id).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

trait StorageDbExt {
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64>;
    async fn insert_dataset(
        &mut self,
        uid: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<i64>;
    async fn find_dataset_by_uid(&mut self, uid: Uuid) -> Result<i64>;
    async fn add_tag_to_dataset(&mut self, dataset_id: i64, tag_id: i64) -> Result<()>;
    async fn clear_dataset_tags(&mut self, dataset_id: i64) -> Result<()>;
    async fn update_dataset(
        &mut self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()>;
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
        description: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query!(
            "INSERT INTO datasets (uid, name, description) VALUES (?, ?, ?) RETURNING id",
            uid,
            name,
            description
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

    async fn clear_dataset_tags(&mut self, dataset_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM dataset_tag WHERE dataset_id = ?", dataset_id)
            .execute(&mut *self)
            .await?;
        Ok(())
    }

    async fn update_dataset(
        &mut self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let res = sqlx::query!(
            "UPDATE datasets SET name = ifnull(?, name), description = ifnull(?, description) WHERE id = ?",
            name,
            description,
            id
        )
        .execute(&mut *self)
        .await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    async fn find_dataset_by_uid(&mut self, uid: Uuid) -> Result<i64> {
        sqlx::query!("SELECT id FROM datasets WHERE uid = ?", uid)
            .fetch_optional(&mut *self)
            .await?
            .map(|r| r.id)
            .ok_or(Error::NotFound)
    }
}

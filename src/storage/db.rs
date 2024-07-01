use sqlx::SqliteConnection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Sqlx error: {0}")]
    Other(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub async fn create(
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
    pool: &sqlx::SqlitePool,
) -> Result<i64> {
    let mut tx = pool.begin().await?;
    let data_index_id = tx.insert_data_index(name, description).await?;
    for tag in tags.into_iter().flatten() {
        let tag_id = tx.get_or_insert_tag(tag).await?;
        tx.add_tag_to_data_index(data_index_id, tag_id).await?;
    }
    tx.commit().await?;
    Ok(data_index_id)
}

pub async fn update(
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
    pool: &sqlx::SqlitePool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    tx.update_data_index(id, name, description).await?;
    if let Some(tags) = tags {
        tx.clear_tag_relations(id).await?;
        for tag in tags {
            let tag_id = tx.get_or_insert_tag(tag).await?;
            tx.add_tag_to_data_index(id, tag_id).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

trait StorageDbExt {
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64>;
    async fn insert_data_index(
        &mut self,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<i64>;
    async fn add_tag_to_data_index(&mut self, data_index_id: i64, tag_id: i64) -> Result<()>;
    async fn clear_tag_relations(&mut self, data_index_id: i64) -> Result<()>;
    async fn update_data_index(
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

    async fn insert_data_index(
        &mut self,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query!(
            "INSERT INTO data_indices (name, description) VALUES (?, ?) RETURNING id",
            name,
            description
        )
        .fetch_one(&mut *self)
        .await?
        .id;
        Ok(id)
    }

    async fn add_tag_to_data_index(&mut self, data_index_id: i64, tag_id: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO data_index_tag (data_index_id, tag_id) VALUES (?, ?)",
            data_index_id,
            tag_id
        )
        .execute(&mut *self)
        .await?;
        Ok(())
    }

    async fn clear_tag_relations(&mut self, data_index_id: i64) -> Result<()> {
        sqlx::query!(
            "DELETE FROM data_index_tag WHERE data_index_id = ?",
            data_index_id
        )
        .execute(&mut *self)
        .await?;
        Ok(())
    }

    async fn update_data_index(
        &mut self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let res = sqlx::query!(
            "UPDATE data_indices SET name = ifnull(?, name), description = ifnull(?, description) WHERE id = ?",
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
}

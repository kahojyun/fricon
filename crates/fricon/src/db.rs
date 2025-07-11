use std::collections::HashSet;

use anyhow::{Context, ensure};
use chrono::{DateTime, Utc};
use futures::prelude::*;
use sqlx::{
    SqliteConnection, SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    types::Json,
};
use thiserror::Error;
use tracing::info;
use uuid::{Uuid, fmt::Simple};

use crate::{dataset::Info, paths::DatabaseFile};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Sqlx error: {0}")]
    Other(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

pub async fn connect(path: &DatabaseFile) -> anyhow::Result<SqlitePool> {
    let path = &path.0;
    info!("Connect to database at {}", path.display());
    let pool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::new().filename(path))
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

pub async fn init(path: &DatabaseFile) -> anyhow::Result<SqlitePool> {
    let path = &path.0;
    ensure!(!path.exists(), "Database already exists.");
    info!("Initialize database at {}", path.display());
    let options = SqliteConnectOptions::new()
        .filename(path)
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .context("Failed to create database.")?;
    MIGRATOR
        .run(&pool)
        .await
        .context("Failed to initialize database schema.")?;
    Ok(pool)
}

pub struct DatasetIndex {
    pub pool: SqlitePool,
}

pub struct DatasetRecord {
    pub id: i64,
    pub info: Info,
}

impl DatasetIndex {
    pub async fn create(&self, info: &Info) -> Result<i64> {
        let mut tx = self.pool.begin().await?;
        let dataset_id = tx.insert_dataset(info).await?;
        for tag in &info.tags {
            let tag_id = tx.get_or_insert_tag(tag).await?;
            tx.add_dataset_tag(dataset_id, tag_id).await?;
        }
        tx.commit().await?;
        Ok(dataset_id)
    }

    pub async fn get_by_uid(&self, uid: Uuid) -> Result<DatasetRecord> {
        let mut tx = self.pool.begin().await?;
        let id = tx.find_dataset_by_uid(uid.simple()).await?;
        tx.get_dataset_by_id(id).await
    }

    pub async fn get_by_id(&self, id: i64) -> Result<DatasetRecord> {
        let mut tx = self.pool.begin().await?;
        tx.get_dataset_by_id(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<DatasetRecord>> {
        let mut tx = self.pool.begin().await?;
        let ids = sqlx::query_scalar!("SELECT id FROM datasets ORDER BY id DESC")
            .fetch_all(&mut *tx)
            .await?;
        let mut datasets = Vec::with_capacity(ids.len());
        for id in ids {
            let record = tx.get_dataset_by_id(id).await?;
            datasets.push(record);
        }
        Ok(datasets)
    }

    pub async fn add_dataset_tags(&self, id: i64, tags: &[String]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        tx.ensure_exist(id).await?;
        for tag in tags {
            let tag_id = tx.get_or_insert_tag(tag).await?;
            tx.add_dataset_tag(id, tag_id).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn remove_dataset_tags(&self, id: i64, tags: &[String]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        tx.ensure_exist(id).await?;
        for tag in tags {
            sqlx::query!(
                r#"
                DELETE FROM dataset_tag
                WHERE dataset_id = ? AND tag_id = (SELECT id FROM tags WHERE name = ?)
                "#,
                id,
                tag
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn replace_dataset_tags(&self, id: i64, tags: &[String]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        tx.ensure_exist(id).await?;
        let current_tag_ids =
            sqlx::query_scalar!("SELECT tag_id FROM dataset_tag WHERE dataset_id = ?", id)
                .fetch(&mut *tx)
                .try_collect::<HashSet<_>>()
                .await?;
        let mut new_tag_ids = HashSet::with_capacity(tags.len());
        for tag in tags {
            let tag_id = tx.get_or_insert_tag(tag).await?;
            new_tag_ids.insert(tag_id);
        }
        for &tag_id in current_tag_ids.difference(&new_tag_ids) {
            tx.remove_dataset_tag(id, tag_id).await?;
        }
        for &tag_id in new_tag_ids.difference(&current_tag_ids) {
            tx.add_dataset_tag(id, tag_id).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn update_dataset_name(&self, id: i64, name: &str) -> Result<()> {
        sqlx::query!("UPDATE datasets SET name = ? WHERE id = ?", name, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_dataset_description(&self, id: i64, description: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE datasets SET description = ? WHERE id = ?",
            description,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_dataset_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        sqlx::query!(
            "UPDATE datasets SET favorite = ? WHERE id = ?",
            favorite,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

trait StorageDbExt {
    async fn ensure_exist(&mut self, id: i64) -> Result<()>;
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64>;
    async fn insert_dataset(&mut self, info: &Info) -> Result<i64>;
    async fn find_dataset_by_uid(&mut self, uid: Simple) -> Result<i64>;
    async fn add_dataset_tag(&mut self, dataset_id: i64, tag_id: i64) -> Result<()>;
    async fn remove_dataset_tag(&mut self, dataset_id: i64, tag_id: i64) -> Result<()>;
    async fn get_dataset_by_id(&mut self, id: i64) -> Result<DatasetRecord>;
}

impl StorageDbExt for SqliteConnection {
    async fn ensure_exist(&mut self, id: i64) -> Result<()> {
        let exist = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM datasets WHERE id = ?) as "exist: bool""#,
            id
        )
        .fetch_one(&mut *self)
        .await?;
        if exist { Ok(()) } else { Err(Error::NotFound) }
    }

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

    async fn insert_dataset(&mut self, info: &Info) -> Result<i64> {
        let index_columns = Json(&info.index_columns);
        let uid = info.uid.simple();
        let path = &info.path.0;
        let id = sqlx::query!(
            r#"
            INSERT INTO datasets
            (uid, name, description, favorite, index_columns, path, created_at)
            VALUES
            (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
            uid,
            info.name,
            info.description,
            info.favorite,
            index_columns,
            path,
            info.created_at,
        )
        .fetch_one(&mut *self)
        .await?
        .id;
        Ok(id)
    }

    async fn add_dataset_tag(&mut self, dataset_id: i64, tag_id: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO dataset_tag (dataset_id, tag_id) VALUES (?, ?)",
            dataset_id,
            tag_id
        )
        .execute(&mut *self)
        .await?;
        Ok(())
    }

    async fn remove_dataset_tag(&mut self, dataset_id: i64, tag_id: i64) -> Result<()> {
        sqlx::query!(
            "DELETE FROM dataset_tag WHERE dataset_id = ? AND tag_id = ?",
            dataset_id,
            tag_id
        )
        .execute(&mut *self)
        .await?;
        Ok(())
    }

    async fn find_dataset_by_uid(&mut self, uid: Simple) -> Result<i64> {
        sqlx::query!("SELECT id FROM datasets WHERE uid = ?", uid)
            .fetch_optional(&mut *self)
            .await?
            .map(|r| r.id)
            .ok_or(Error::NotFound)
    }

    async fn get_dataset_by_id(&mut self, id: i64) -> Result<DatasetRecord> {
        #[derive(Debug)]
        pub struct DatasetRow {
            pub uid: Simple,
            pub name: String,
            pub description: String,
            pub favorite: bool,
            pub index_columns: Json<Vec<String>>,
            pub path: String,
            pub created_at: DateTime<Utc>,
        }
        let res = sqlx::query_as!(
            DatasetRow,
            r#"
            SELECT
            uid as "uid: _", name, description, favorite, index_columns as "index_columns: _",
            path, created_at as "created_at: _"
            FROM datasets WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&mut *self)
        .await?
        .ok_or(Error::NotFound)?;
        let tags = sqlx::query_scalar!(
            r#"
            SELECT t.name FROM tags t
            JOIN dataset_tag dt ON t.id = dt.tag_id
            WHERE dt.dataset_id = ?
            "#,
            id
        )
        .fetch_all(&mut *self)
        .await?;
        let info = Info {
            uid: res.uid.into_uuid(),
            name: res.name,
            description: res.description,
            favorite: res.favorite,
            index_columns: res.index_columns.0,
            path: res.path.into(),
            tags,
            created_at: res.created_at,
        };
        Ok(DatasetRecord { id, info })
    }
}

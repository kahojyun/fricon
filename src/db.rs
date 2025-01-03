use anyhow::{ensure, Context};
use chrono::{DateTime, Utc};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    types::Json,
    SqliteConnection, SqlitePool,
};
use thiserror::Error;
use tracing::info;
use uuid::{fmt::Simple, Uuid};

use crate::{dataset::Info, paths::DatabaseFile};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Sqlx error: {0}")]
    Other(#[from] sqlx::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub static MIGRATOR: Migrator = sqlx::migrate!();

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
            tx.add_tag_to_dataset(dataset_id, tag_id).await?;
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
        let ids = sqlx::query_scalar!("SELECT id FROM datasets")
            .fetch_all(&mut *tx)
            .await?;
        let mut datasets = Vec::with_capacity(ids.len());
        for id in ids {
            let record = tx.get_dataset_by_id(id).await?;
            datasets.push(record);
        }
        Ok(datasets)
    }
}

trait StorageDbExt {
    async fn get_or_insert_tag(&mut self, tag: &str) -> Result<i64>;
    async fn insert_dataset(&mut self, info: &Info) -> Result<i64>;
    async fn find_dataset_by_uid(&mut self, uid: Simple) -> Result<i64>;
    async fn add_tag_to_dataset(&mut self, dataset_id: i64, tag_id: i64) -> Result<()>;
    async fn get_dataset_by_id(&mut self, id: i64) -> Result<DatasetRecord>;
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

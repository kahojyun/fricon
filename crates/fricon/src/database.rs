mod models;
#[rustfmt::skip]
#[allow(clippy::module_name_repetitions)]
pub mod schema;
mod types;

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use deadpool_diesel::{
    Runtime,
    sqlite::{Hook, HookError, Manager, Pool},
};
use diesel::{
    QueryResult, RunQueryDsl, SqliteConnection, connection::SimpleConnection,
    migration::MigrationSource, sqlite::Sqlite,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures::FutureExt;
use thiserror::Error;
use tracing::{error, info};

pub use self::{
    models::{Dataset, DatasetTag, DatasetUpdate, NewDataset, Tag},
    types::{DatasetStatus, SimpleUuid},
};

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error(transparent)]
    Pool(#[from] deadpool_diesel::PoolError),

    #[error(transparent)]
    Migration(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    Query(#[from] diesel::result::Error),

    #[error(transparent)]
    General(#[from] anyhow::Error),
}

pub async fn connect(
    path: impl AsRef<Path>,
    backup_path: impl Into<PathBuf>,
) -> Result<Pool, DatabaseError> {
    let path = path.as_ref();
    let backup_path = backup_path.into();
    info!("Connect to database at {}", path.display());

    let manager = Manager::new(path.display().to_string(), Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(8)
        .post_create(Hook::async_fn(|obj, _| {
            async move {
                obj.interact(initialize_connection)
                    .await
                    .unwrap()
                    .map_err(|e| HookError::message(e.to_string()))
            }
            .boxed()
        }))
        .build()
        .context("Failed to create database pool")?;
    pool.interact(move |conn| run_migrations(conn, &backup_path))
        .await?
        .context("Migration execution failed during connection")?;
    Ok(pool)
}

fn backup_database(conn: &mut SqliteConnection, backup_path: &Path) -> Result<(), DatabaseError> {
    let backup_path_str = backup_path
        .to_str()
        .context("Invalid backup path encoding")?;
    diesel::sql_query("VACUUM INTO ?")
        .bind::<diesel::sql_types::Text, _>(backup_path_str)
        .execute(conn)?;
    Ok(())
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut SqliteConnection, backup_path: &Path) -> Result<(), DatabaseError> {
    let applied_migrations = conn.applied_migrations()?;
    let available_migrations = MigrationSource::<Sqlite>::migrations(&MIGRATIONS)?;

    if applied_migrations.len() > available_migrations.len() {
        return Err(DatabaseError::Migration(
            anyhow!("Migration count mismatch").into(),
        ));
    }

    let has_pending = conn.has_pending_migration(MIGRATIONS)?;

    if has_pending {
        info!("Running pending database migrations");
        backup_database(conn, backup_path)?;
        let _result = conn.run_pending_migrations(MIGRATIONS)?;
        info!("Database migrations completed");
    }

    Ok(())
}

fn initialize_connection(conn: &mut SqliteConnection) -> QueryResult<()> {
    // https://docs.rs/diesel/2.2.12/diesel/sqlite/struct.SqliteConnection.html#concurrency
    conn.batch_execute("PRAGMA busy_timeout = 5000;")?;
    conn.batch_execute("PRAGMA journal_mode = WAL;")?;
    conn.batch_execute("PRAGMA synchronous = NORMAL;")?;
    conn.batch_execute("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

pub trait PoolExt {
    async fn interact<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&mut SqliteConnection) -> R + Send + 'static,
        R: Send + 'static;
}

impl PoolExt for Pool {
    async fn interact<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&mut SqliteConnection) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.get()
            .await?
            .interact(f)
            .await
            .map_err(|e| DatabaseError::General(anyhow!("Interact error: {e}")))
    }
}

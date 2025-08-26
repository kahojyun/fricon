mod models;
#[rustfmt::skip]
#[allow(clippy::module_name_repetitions)]
pub mod schema;
mod types;

pub use self::{
    models::{Dataset, DatasetTag, DatasetUpdate, NewDataset, Tag},
    types::{DatasetStatus, JsonValue, SimpleUuid},
};

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
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
use tracing::info;

pub async fn connect(path: impl AsRef<Path>, backup_path: impl Into<PathBuf>) -> Result<Pool> {
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
        .build()?;
    pool.interact(move |conn| run_migrations(conn, &backup_path))
        .await
        .context("Failed to run migrations")?;
    Ok(pool)
}

fn backup_database(conn: &mut SqliteConnection, backup_path: &Path) -> Result<()> {
    info!("Creating database backup at {}", backup_path.display());
    let backup_path_str = backup_path
        .to_str()
        .context("Backup path contains invalid UTF-8")?;
    diesel::sql_query("VACUUM INTO ?")
        .bind::<diesel::sql_types::Text, _>(backup_path_str)
        .execute(conn)?;
    Ok(())
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut SqliteConnection, backup_path: &Path) -> Result<()> {
    let applied_migrations = conn
        .applied_migrations()
        .map_err(anyhow::Error::from_boxed)?;
    if applied_migrations.len()
        > MigrationSource::<Sqlite>::migrations(&MIGRATIONS)
            .map_err(anyhow::Error::from_boxed)?
            .len()
    {
        bail!("Database has more applied migrations than expected");
    }

    let has_pending = conn
        .has_pending_migration(MIGRATIONS)
        .map_err(anyhow::Error::from_boxed)?;

    if has_pending {
        backup_database(conn, backup_path)?;
        info!("Running pending database migrations");
        let result = conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(anyhow::Error::from_boxed)?;
        for migration in result {
            info!("Database migration {} completed", migration);
        }
    } else {
        info!("Database is up to date, no migrations needed");
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
    async fn interact<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T> + Send + 'static,
        T: Send + 'static;
}

impl PoolExt for Pool {
    async fn interact<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut SqliteConnection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.get().await?;
        conn.interact(f)
            .await
            .map_err(|e| anyhow!("Failed to interact with database connection: {e}"))
            .flatten()
    }
}

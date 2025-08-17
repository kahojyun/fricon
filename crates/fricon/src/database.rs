mod models;
#[rustfmt::skip]
#[allow(clippy::module_name_repetitions)]
pub mod schema;
mod types;

pub use self::{
    models::{Dataset, DatasetTag, DatasetUpdate, NewDataset, NewTag, Tag},
    types::{JsonValue, SimpleUuid},
};

use std::path::Path;

use anyhow::{Context, Result};
use deadpool_diesel::{
    Runtime,
    sqlite::{Hook, HookError, Manager, Pool},
};
use diesel::{QueryResult, SqliteConnection, connection::SimpleConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures::FutureExt;
use tracing::info;

pub async fn connect(path: impl AsRef<Path>) -> Result<Pool> {
    let path = path.as_ref();
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
    pool.interact(run_migrations)
        .await
        .context("Failed to run migrations")?;
    Ok(pool)
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut SqliteConnection) -> Result<()> {
    let result = conn
        .run_pending_migrations(MIGRATIONS)
        .map_err(anyhow::Error::from_boxed)?;
    for migration in result {
        info!("Database migration {} completed", migration);
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
        conn.interact(f).await.expect("Interact error")
    }
}

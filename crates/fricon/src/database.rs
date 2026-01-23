mod models;
#[rustfmt::skip]
pub mod schema;
mod types;

use std::{
    error::Error as StdError,
    path::{Path, PathBuf},
};

use diesel::{
    RunQueryDsl, SqliteConnection,
    connection::SimpleConnection,
    migration::MigrationSource,
    prelude::*,
    r2d2,
    r2d2::{ConnectionManager, CustomizeConnection},
    result::Error as DieselError,
    sql_types::Text,
    sqlite::Sqlite,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;
use tracing::info;

pub use self::{
    models::{Dataset, DatasetTag, DatasetUpdate, NewDataset, Tag},
    types::{DatasetStatus, SimpleUuid},
};

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Invalid backup path encoding.")]
    InvalidBackupPath,
    #[error("Migration failed: {0}")]
    Migration(Box<dyn StdError + Send + Sync>),
    #[error(transparent)]
    Pool(#[from] r2d2::PoolError),
    #[error(transparent)]
    Query(#[from] DieselError),
    #[error(transparent)]
    General(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct Pool(r2d2::Pool<ConnectionManager<SqliteConnection>>);

impl Pool {
    pub fn get(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
        Ok(self.0.get()?)
    }

    fn build(database_url: String) -> Result<Self, DatabaseError> {
        #[derive(Debug)]
        struct Customizer;

        impl CustomizeConnection<SqliteConnection, r2d2::Error> for Customizer {
            fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
                // https://docs.rs/diesel/2.2.12/diesel/sqlite/struct.SqliteConnection.html#concurrency
                conn.batch_execute("PRAGMA busy_timeout = 5000;")?;
                conn.batch_execute("PRAGMA journal_mode = WAL;")?;
                conn.batch_execute("PRAGMA synchronous = NORMAL;")?;
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
                Ok(())
            }
        }

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .connection_customizer(Box::new(Customizer))
            .build(manager)?;
        Ok(Self(pool))
    }
}

pub fn connect(
    path: impl AsRef<Path>,
    backup_path: impl Into<PathBuf>,
) -> Result<Pool, DatabaseError> {
    let path = path.as_ref();
    let backup_path = backup_path.into();
    info!("Connect to database at {}", path.display());
    let pool = Pool::build(path.display().to_string())?;
    let mut conn = pool.get()?;
    run_migrations(&mut conn, &backup_path).map_err(DatabaseError::Migration)?;
    Ok(pool)
}

fn run_migrations(
    conn: &mut SqliteConnection,
    backup_path: &Path,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    let applied_migrations = conn.applied_migrations()?;
    let available_migrations = MigrationSource::<Sqlite>::migrations(&MIGRATIONS)?;

    if applied_migrations.len() > available_migrations.len() {
        return Err("Migration count mismatch".into());
    }

    if conn.has_pending_migration(MIGRATIONS)? {
        info!("Running pending database migrations");
        backup_database(conn, backup_path)?;
        let _applied = conn.run_pending_migrations(MIGRATIONS)?;
        info!("Database migrations completed");
    }

    Ok(())
}

fn backup_database(conn: &mut SqliteConnection, backup_path: &Path) -> Result<(), DatabaseError> {
    let backup_path_str = backup_path
        .to_str()
        .ok_or(DatabaseError::InvalidBackupPath)?;
    diesel::sql_query("VACUUM INTO ?")
        .bind::<Text, _>(backup_path_str)
        .execute(conn)?;
    Ok(())
}

/// Updates all datasets with 'writing' status to 'aborted' status
/// This should be called during service startup to handle interrupted writes
pub fn cleanup_writing_datasets(pool: &Pool) -> Result<usize, DatabaseError> {
    use self::schema::datasets::dsl::{datasets, status};

    let mut conn = pool.get()?;
    let updated_count = diesel::update(datasets.filter(status.eq(DatasetStatus::Writing)))
        .set(status.eq(DatasetStatus::Aborted))
        .execute(&mut conn)?;

    if updated_count > 0 {
        info!(
            "Updated {} datasets from 'writing' to 'aborted' status",
            updated_count
        );
    }

    Ok(updated_count)
}

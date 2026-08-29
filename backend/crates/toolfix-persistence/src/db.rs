//! Database connection and migrations.

use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

use crate::error::PersistenceError;

/// The embedded migration set (backend/migrations/postgres).
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations/postgres");

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, PersistenceError> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn ping(&self) -> Result<(), PersistenceError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

/// Applies pending migrations. Migration files are immutable once applied:
/// SQLx records checksums in `_sqlx_migrations`.
pub async fn run_migrations(pool: &PgPool) -> Result<(), PersistenceError> {
    match MIGRATOR.run(pool).await {
        Ok(()) => {
            tracing::info!("migrations up to date");
            Ok(())
        }
        Err(sqlx::migrate::MigrateError::VersionMissing(v)) => Err(PersistenceError::Migration(
            format!("database is missing applied migration {v}; migrations are immutable — restore it"),
        )),
        Err(sqlx::migrate::MigrateError::VersionMismatch(v)) => Err(PersistenceError::Migration(
            format!("migration {v} was previously applied but has been modified; add a new migration instead"),
        )),
        Err(e) => Err(PersistenceError::Migration(e.to_string())),
    }
}

/// Convenience: connect + migrate.
pub async fn connect(database_url: &str) -> Result<PgPool, PersistenceError> {
    let options: PgConnectOptions = database_url.parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

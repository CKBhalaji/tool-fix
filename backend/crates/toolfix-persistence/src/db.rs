//! Database connections and migrations.
//!
//! Two backends are supported and selected by configuration:
//! - **SQLite** for local development (`sqlite://dev.db?mode=rwc` — the
//!   database file is created on startup if missing).
//! - **PostgreSQL** for production (the database itself is created on
//!   startup when missing).
//!
//! Repositories write each query once and dispatch through the [`dual`]
//! / [`dual_tx`] macros, because SQL is kept dialect-neutral (no
//! `FOR UPDATE`, `now()`, `::casts`, or server arrays in repository SQL).

use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::Transaction;

use crate::error::PersistenceError;

/// Postgres migrator (backend/migrations/postgres).
static MIGRATOR_PG: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations/postgres");
/// SQLite migrator (backend/migrations/sqlite).
static MIGRATOR_SQLITE: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations/sqlite");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbDriver {
    Postgres,
    Sqlite,
}

impl DbDriver {
    /// Infers the driver from a `DATABASE_URL` scheme
    /// (`sqlite://…` / `postgres://…` / `postgresql://…`).
    pub fn infer(url: &str) -> Result<Self, PersistenceError> {
        let lower = url.trim().to_ascii_lowercase();
        if lower.starts_with("sqlite:") {
            Ok(DbDriver::Sqlite)
        } else if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
            Ok(DbDriver::Postgres)
        } else {
            Err(PersistenceError::InvalidData(format!(
                "cannot infer database driver from DATABASE_URL scheme: {url}"
            )))
        }
    }
}

/// A handle to whichever backend is configured.
#[derive(Clone)]
pub enum Db {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

/// Transaction handle ('static: pools are Arc-based internally).
pub enum DbTx {
    Postgres(Transaction<'static, sqlx::Postgres>),
    Sqlite(Transaction<'static, sqlx::Sqlite>),
}

impl Db {
    pub fn driver(&self) -> DbDriver {
        match self {
            Db::Postgres(_) => DbDriver::Postgres,
            Db::Sqlite(_) => DbDriver::Sqlite,
        }
    }

    pub fn is_sqlite(&self) -> bool {
        self.driver() == DbDriver::Sqlite
    }

    pub async fn begin(&self) -> Result<DbTx, PersistenceError> {
        Ok(match self {
            Db::Postgres(p) => DbTx::Postgres(p.begin().await?),
            Db::Sqlite(p) => DbTx::Sqlite(p.begin().await?),
        })
    }

    pub async fn ping(&self) -> Result<(), PersistenceError> {
        match self {
            Db::Postgres(p) => {
                sqlx::query("SELECT 1").execute(p).await.map(|_| ())?;
                Ok(())
            }
            Db::Sqlite(p) => {
                sqlx::query("SELECT 1").execute(p).await.map(|_| ())?;
                Ok(())
            }
        }
    }
}

impl DbTx {
    pub async fn commit(self) -> Result<(), PersistenceError> {
        match self {
            DbTx::Postgres(t) => {
                t.commit().await?;
                Ok(())
            }
            DbTx::Sqlite(t) => {
                t.commit().await?;
                Ok(())
            }
        }
    }

    pub async fn rollback(self) -> Result<(), PersistenceError> {
        match self {
            DbTx::Postgres(t) => {
                t.rollback().await?;
                Ok(())
            }
            DbTx::Sqlite(t) => {
                t.rollback().await?;
                Ok(())
            }
        }
    }
}

/// Normalizes a SQLite URL and guarantees the database file can be created:
/// the parent directory is created and `mode=rwc` is forced so a missing
/// file is created automatically at startup.
fn prepare_sqlite_url(url: &str) -> Result<String, PersistenceError> {
    if url.trim().to_ascii_lowercase().starts_with("sqlite::memory:") {
        return Ok(url.to_string());
    }
    let mut normalized = url.to_string();
    if !normalized.contains('?') {
        normalized.push_str("?mode=rwc");
    } else if !normalized.contains("mode=") {
        normalized.push_str("&mode=rwc");
    }
    // Extract the filesystem path (sqlite://path or sqlite:path) and make
    // sure its directory exists.
    let raw = normalized
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let path = raw.split('?').next().unwrap_or_default();
    if !path.is_empty()
        && let Some(parent) = std::path::Path::new(path).parent()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| PersistenceError::InvalidData(format!("cannot create database directory: {e}")))?;
    }
    Ok(normalized)
}

/// When the target PostgreSQL database does not exist, connect to the
/// `postgres` maintenance database with the same credentials and create it,
/// then return the original options for a retry.
async fn ensure_postgres_database(options: &PgConnectOptions) -> Result<(), PersistenceError> {
    let Some(target) = options.get_database() else {
        return Ok(());
    };
    let maintenance = options.clone().database("postgres");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(maintenance)
        .await
        .map_err(PersistenceError::from)?;
    let (exists,): (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(target)
            .fetch_one(&pool)
            .await
            .map_err(PersistenceError::from)?;
    if !exists {
        // Identifiers cannot be bound as parameters; the name is taken from
        // the configured URL operator, and sanitized via quoting.
        let quoted = format!("\"{}\"", target.replace('"', "\"\""));
        sqlx::query(&format!("CREATE DATABASE {quoted}"))
            .execute(&pool)
            .await
            .map_err(PersistenceError::from)?;
        tracing::info!(database = %target, "created PostgreSQL database");
    }
    pool.close().await;
    Ok(())
}

/// Connects (creating the SQLite file / PostgreSQL database when missing)
/// and applies the migrations for the selected driver.
pub async fn connect_with_driver(driver: DbDriver, url: &str) -> Result<Db, PersistenceError> {
    match driver {
        DbDriver::Postgres => {
            let options: PgConnectOptions = url
                .parse()
                .map_err(|e| PersistenceError::InvalidData(format!("invalid postgres url: {e}")))?;

            let pool = PgPoolOptions::new()
                .max_connections(20)
                .acquire_timeout(Duration::from_secs(10))
                .connect_with(options.clone())
                .await;
            let pool = match pool {
                Ok(pool) => pool,
                Err(sqlx::Error::Database(db_err))
                    if db_err.code().as_deref() == Some("3D000") =>
                {
                    // 3D000: database does not exist — create and retry once.
                    ensure_postgres_database(&options).await?;
                    PgPoolOptions::new()
                        .max_connections(20)
                        .acquire_timeout(Duration::from_secs(10))
                        .connect_with(options)
                        .await?
                }
                Err(e) => return Err(e.into()),
            };
            run_migrations(&Db::Postgres(pool.clone())).await?;
            Ok(Db::Postgres(pool))
        }
        DbDriver::Sqlite => {
            let normalized = prepare_sqlite_url(url)?;
            let options: SqliteConnectOptions = normalized
                .parse()
                .map_err(|e| PersistenceError::InvalidData(format!("invalid sqlite url: {e}")))?;
            let pool = SqlitePoolOptions::new()
                .max_connections(10)
                .acquire_timeout(Duration::from_secs(10))
                .connect_with(options)
                .await?;
            run_migrations(&Db::Sqlite(pool.clone())).await?;
            Ok(Db::Sqlite(pool))
        }
    }
}

/// Convenience: driver inferred from the URL.
pub async fn connect(url: &str) -> Result<Db, PersistenceError> {
    connect_with_driver(DbDriver::infer(url)?, url).await
}

/// Applies pending migrations for the selected backend.
pub async fn run_migrations(db: &Db) -> Result<(), PersistenceError> {
    let result = match db {
        Db::Postgres(pool) => MIGRATOR_PG.run(pool).await,
        Db::Sqlite(pool) => MIGRATOR_SQLITE.run(pool).await,
    };
    match result {
        Ok(()) => {
            tracing::info!(driver = ?db.driver(), "migrations up to date");
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

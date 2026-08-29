//! PostgreSQL **and** SQLite persistence: the durable source of truth.
//!
//! The backend is selected by configuration (`DATABASE_DRIVER` / URL scheme);
//! repositories write each query once and dispatch to the active pool via
//! the `dual!`/`dual_tx!` macros. SQL is dialect-neutral: timestamps are
//! bound from Rust, list columns are JSON text, and locking-specific syntax
//! is selected at runtime.

pub mod db;
pub mod error;
pub mod models;
pub mod repositories;

pub use db::{connect, connect_with_driver, run_migrations, Db, DbDriver, DbTx};
pub use error::PersistenceError;

/// Runs a query expression against whichever pool is configured.
///
/// Usage:
/// `dual!(&self.db, |e| sqlx::query_as::<_, Row>(SQL).bind(x).fetch_one(e).await)?`
macro_rules! dual {
    ($db:expr, |$exec:ident| $body:expr) => {
        match $db {
            $crate::db::Db::Postgres($exec) => $body,
            $crate::db::Db::Sqlite($exec) => $body,
        }
    };
}

/// Same as [`dual`] for an open transaction. Accepts an owned `DbTx` or a
/// `&mut DbTx` (repository parameters) and yields `&mut <DB Connection>`
/// (via the transaction's `DerefMut`) as the executor.
macro_rules! dual_tx {
    ($tx:expr, |$exec:ident| $body:expr) => {
        match $tx {
            $crate::db::DbTx::Postgres(tx_pg) => {
                let $exec = &mut **tx_pg;
                $body
            }
            $crate::db::DbTx::Sqlite(tx_sq) => {
                let $exec = &mut **tx_sq;
                $body
            }
        }
    };
}

pub(crate) use dual;
pub(crate) use dual_tx;

/// Serializes a list column (stored as JSON text on both backends).
pub(crate) fn list_to_json(items: &[String]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}

/// Deserializes a JSON list column; malformed values degrade to empty.
pub(crate) fn list_from_json(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

/// Bundles every repository behind one handle so the composition root can
/// construct services from a single object.
#[derive(Clone)]
pub struct Repositories {
    db: Db,
    pub users: repositories::Users,
    pub auth_sessions: repositories::AuthSessions,
    pub vehicles: repositories::Vehicles,
    pub mechanics: repositories::Mechanics,
    pub breakdowns: repositories::Breakdowns,
    pub jobs: repositories::Jobs,
    pub offers: repositories::Offers,
    pub notifications: repositories::Notifications,
    pub locations: repositories::Locations,
    pub agents: repositories::Agents,
    pub pricing: repositories::Pricing,
    pub payments: repositories::Payments,
    pub ratings: repositories::Ratings,
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        Self {
            users: repositories::Users::new(db.clone()),
            auth_sessions: repositories::AuthSessions::new(db.clone()),
            vehicles: repositories::Vehicles::new(db.clone()),
            mechanics: repositories::Mechanics::new(db.clone()),
            breakdowns: repositories::Breakdowns::new(db.clone()),
            jobs: repositories::Jobs::new(db.clone()),
            offers: repositories::Offers::new(db.clone()),
            notifications: repositories::Notifications::new(db.clone()),
            locations: repositories::Locations::new(db.clone()),
            agents: repositories::Agents::new(db.clone()),
            pricing: repositories::Pricing::new(db.clone()),
            payments: repositories::Payments::new(db.clone()),
            ratings: repositories::Ratings::new(db.clone()),
            db,
        }
    }

    /// Handle for services that orchestrate their own transactions.
    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn driver(&self) -> DbDriver {
        self.db.driver()
    }

    pub async fn begin(&self) -> Result<DbTx, PersistenceError> {
        self.db.begin().await
    }

    /// Dependency probe (readiness checks).
    pub async fn ping(&self) -> Result<(), PersistenceError> {
        self.db.ping().await
    }
}

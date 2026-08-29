//! PostgreSQL persistence: the durable source of truth.
//!
//! All SQL lives in this crate. Repositories are thin, explicit, and
//! transaction-aware; callers above (services, API) never write SQL.

pub mod db;
pub mod error;
pub mod models;
pub mod repositories;

pub use db::{connect, run_migrations, Database};
pub use error::PersistenceError;

/// Bundles every repository behind one handle so the composition root can
/// construct services from a single object.
#[derive(Clone)]
pub struct Repositories {
    pool: sqlx::PgPool,
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
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            users: repositories::Users::new(pool.clone()),
            auth_sessions: repositories::AuthSessions::new(pool.clone()),
            vehicles: repositories::Vehicles::new(pool.clone()),
            mechanics: repositories::Mechanics::new(pool.clone()),
            breakdowns: repositories::Breakdowns::new(pool.clone()),
            jobs: repositories::Jobs::new(pool.clone()),
            offers: repositories::Offers::new(pool.clone()),
            notifications: repositories::Notifications::new(pool.clone()),
            locations: repositories::Locations::new(pool.clone()),
            agents: repositories::Agents::new(pool.clone()),
            pricing: repositories::Pricing::new(pool.clone()),
            payments: repositories::Payments::new(pool.clone()),
            ratings: repositories::Ratings::new(pool.clone()),
            pool,
        }
    }

    /// Pool access for services that orchestrate their own transactions.
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

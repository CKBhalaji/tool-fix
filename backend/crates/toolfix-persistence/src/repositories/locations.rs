//! Location events repository.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use toolfix_location::events::LocationSubject;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::LatestLocationRow;

#[derive(Clone)]
pub struct Locations {
    pool: PgPool,
}

impl Locations {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_event(
        &self,
        subject: LocationSubject,
        subject_id: Uuid,
        job_id: Option<Uuid>,
        latitude: f64,
        longitude: f64,
        accuracy_m: Option<f64>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Uuid, PersistenceError> {
        let subject_str = match subject {
            LocationSubject::Customer => "customer",
            LocationSubject::Mechanic => "mechanic",
        };
        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO location_events (id, subject, subject_id, job_id, latitude, longitude, accuracy_m, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(subject_str)
        .bind(subject_id)
        .bind(job_id)
        .bind(latitude)
        .bind(longitude)
        .bind(accuracy_m)
        .bind(recorded_at)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Latest recorded position for a subject (for tracking views).
    pub async fn latest_for_subject(
        &self,
        subject: LocationSubject,
        subject_id: Uuid,
    ) -> Result<LatestLocationRow, PersistenceError> {
        let subject_str = match subject {
            LocationSubject::Customer => "customer",
            LocationSubject::Mechanic => "mechanic",
        };
        sqlx::query_as::<_, LatestLocationRow>(
            r#"
            SELECT latitude, longitude, recorded_at
              FROM location_events
             WHERE subject = $1 AND subject_id = $2
             ORDER BY recorded_at DESC
             LIMIT 1
            "#,
        )
        .bind(subject_str)
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::NotFound)
    }

    pub async fn latest_for_job(
        &self,
        job_id: Uuid,
        subject: LocationSubject,
    ) -> Result<LatestLocationRow, PersistenceError> {
        let subject_str = match subject {
            LocationSubject::Customer => "customer",
            LocationSubject::Mechanic => "mechanic",
        };
        sqlx::query_as::<_, LatestLocationRow>(
            r#"
            SELECT latitude, longitude, recorded_at
              FROM location_events
             WHERE job_id = $1 AND subject = $2
             ORDER BY recorded_at DESC
             LIMIT 1
            "#,
        )
        .bind(job_id)
        .bind(subject_str)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::NotFound)
    }

    /// Retention sweep: drops samples older than the cutoff.
    pub async fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, PersistenceError> {
        let result = sqlx::query("DELETE FROM location_events WHERE recorded_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

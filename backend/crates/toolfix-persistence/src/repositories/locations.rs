//! Location events repository.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::LatestLocationRow;
use crate::{dual, Db};
use toolfix_location::events::LocationSubject;

#[derive(Clone)]
pub struct Locations {
    db: Db,
}

impl Locations {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn subject_str(subject: LocationSubject) -> &'static str {
        match subject {
            LocationSubject::Customer => "customer",
            LocationSubject::Mechanic => "mechanic",
        }
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
        let id = Uuid::now_v7();
        dual!(
            &self.db,
            |e| sqlx::query(
                r#"
                INSERT INTO location_events (id, subject, subject_id, job_id, latitude, longitude, accuracy_m, recorded_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(id)
            .bind(Self::subject_str(subject))
            .bind(subject_id)
            .bind(job_id)
            .bind(latitude)
            .bind(longitude)
            .bind(accuracy_m)
            .bind(recorded_at)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(id)
    }

    /// Latest recorded position for a subject (for tracking views).
    pub async fn latest_for_subject(
        &self,
        subject: LocationSubject,
        subject_id: Uuid,
    ) -> Result<LatestLocationRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, LatestLocationRow>(
                r#"
                SELECT latitude, longitude, recorded_at
                  FROM location_events
                 WHERE subject = $1 AND subject_id = $2
                 ORDER BY recorded_at DESC
                 LIMIT 1
                "#,
            )
            .bind(Self::subject_str(subject))
            .bind(subject_id)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn latest_for_job(
        &self,
        job_id: Uuid,
        subject: LocationSubject,
    ) -> Result<LatestLocationRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, LatestLocationRow>(
                r#"
                SELECT latitude, longitude, recorded_at
                  FROM location_events
                 WHERE job_id = $1 AND subject = $2
                 ORDER BY recorded_at DESC
                 LIMIT 1
                "#,
            )
            .bind(job_id)
            .bind(Self::subject_str(subject))
            .fetch_one(e)
            .await
        )?)
    }

    /// Retention sweep: drops samples older than the cutoff.
    pub async fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, PersistenceError> {
        let result = dual!(
            &self.db,
            |e| sqlx::query("DELETE FROM location_events WHERE recorded_at < $1")
                .bind(cutoff)
                .execute(e)
                .await
            .map(|r| r.rows_affected())
        )?;
        Ok(result)
    }
}

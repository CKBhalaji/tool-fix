//! Assistance jobs repository. Status changes always run through the
//! state-machine-validating helper `transition_job` and write history.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Transaction};
use toolfix_contracts::JobStatus;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::{FeedJobRow, JobRow, JobStatusHistoryRow};

#[derive(Clone)]
pub struct Jobs {
    pool: PgPool,
}

impl Jobs {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        breakdown_id: Uuid,
        customer_user_id: Uuid,
        vehicle_id: Uuid,
    ) -> Result<JobRow, PersistenceError> {
        sqlx::query_as::<_, JobRow>(
            r#"
            INSERT INTO assistance_jobs (id, breakdown_id, customer_user_id, vehicle_id, status)
            VALUES ($1, $2, $3, $4, 'created')
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(breakdown_id)
        .bind(customer_user_id)
        .bind(vehicle_id)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn find_by_id(&self, job_id: Uuid) -> Result<JobRow, PersistenceError> {
        sqlx::query_as::<_, JobRow>("SELECT * FROM assistance_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn find_by_breakdown(
        &self,
        breakdown_id: Uuid,
    ) -> Result<JobRow, PersistenceError> {
        sqlx::query_as::<_, JobRow>("SELECT * FROM assistance_jobs WHERE breakdown_id = $1")
            .bind(breakdown_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn list_by_customer(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<JobRow>, PersistenceError> {
        sqlx::query_as::<_, JobRow>(
            "SELECT * FROM assistance_jobs WHERE customer_user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn list_by_mechanic(
        &self,
        mechanic_id: Uuid,
        limit: i64,
    ) -> Result<Vec<JobRow>, PersistenceError> {
        sqlx::query_as::<_, JobRow>(
            "SELECT * FROM assistance_jobs WHERE selected_mechanic_id = $1 ORDER BY updated_at DESC LIMIT $2",
        )
        .bind(mechanic_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    /// Locks the job row for the caller's transaction (used by the offer
    /// acceptance path).
    pub async fn find_by_id_for_update(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        job_id: Uuid,
    ) -> Result<JobRow, PersistenceError> {
        sqlx::query_as::<_, JobRow>("SELECT * FROM assistance_jobs WHERE id = $1 FOR UPDATE")
            .bind(job_id)
            .fetch_optional(&mut **tx)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    /// Applies a state-machine transition inside an existing transaction,
    /// appending a history row. Returns the updated job.
    #[allow(clippy::too_many_arguments)]
    pub async fn transition_job(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        job_id: Uuid,
        expected_from: JobStatus,
        to: JobStatus,
        changed_by_user_id: Option<Uuid>,
        reason: Option<&str>,
    ) -> Result<JobRow, PersistenceError> {
        let now = Utc::now();
        let job = sqlx::query_as::<_, JobRow>(
            r#"
            UPDATE assistance_jobs
               SET status = $2,
                   updated_at = $3
             WHERE id = $1 AND status = $4
            RETURNING *
            "#,
        )
        .bind(job_id)
        .bind(to.as_str())
        .bind(now)
        .bind(expected_from.as_str())
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(PersistenceError::NotFound)?;

        sqlx::query(
            r#"
            INSERT INTO job_status_history (id, job_id, from_status, to_status, changed_by_user_id, reason)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(expected_from.as_str())
        .bind(to.as_str())
        .bind(changed_by_user_id)
        .bind(reason)
        .execute(&mut **tx)
        .await?;
        Ok(job)
    }

    /// Sets the offer window deadline (used when mechanics are notified).
    pub async fn set_offer_window(
        &self,
        job_id: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "UPDATE assistance_jobs SET offer_window_expires_at = $2, updated_at = now() WHERE id = $1",
        )
        .bind(job_id)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_completion(
        &self,
        job_id: Uuid,
        mechanic_id: Uuid,
        final_amount_minor: i64,
    ) -> Result<(), PersistenceError> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE assistance_jobs SET selected_mechanic_id = $2, final_amount_minor = $3, updated_at = now() WHERE id = $1",
        )
        .bind(job_id)
        .bind(mechanic_id)
        .bind(final_amount_minor)
        .execute(&mut *tx)
        .await?;
        if row.rows_affected() == 0 {
            return Err(PersistenceError::NotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn status_history(
        &self,
        job_id: Uuid,
        limit: i64,
    ) -> Result<Vec<JobStatusHistoryRow>, PersistenceError> {
        sqlx::query_as::<_, JobStatusHistoryRow>(
            "SELECT * FROM job_status_history WHERE job_id = $1 ORDER BY created_at ASC LIMIT $2",
        )
        .bind(job_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    /// Open jobs (accepting offers) with their breakdowns inside a bounding
    /// box — the mechanic requests feed.
    pub async fn list_open_with_breakdowns_in_box(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lng: f64,
        max_lng: f64,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<FeedJobRow>, PersistenceError> {
        sqlx::query_as::<_, FeedJobRow>(
            r#"
            SELECT j.id AS job_id,
                   j.status AS job_status,
                   j.offer_window_expires_at,
                   b.id AS breakdown_id,
                   b.vehicle_id,
                   b.problem_description,
                   b.vehicle_symptoms,
                   b.latitude,
                   b.longitude,
                   b.address,
                   b.created_at AS breakdown_created_at
              FROM assistance_jobs j
              JOIN breakdowns b ON b.id = j.breakdown_id
             WHERE j.status IN ('mechanics_notified', 'offers_received')
               AND b.created_at >= $5
               AND b.latitude BETWEEN $1 AND $2
               AND b.longitude BETWEEN $3 AND $4
             ORDER BY b.created_at DESC
             LIMIT $6
            "#,
        )
        .bind(min_lat)
        .bind(max_lat)
        .bind(min_lng)
        .bind(max_lng)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }
}

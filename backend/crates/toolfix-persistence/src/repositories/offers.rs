//! Mechanic offers repository, including the transactional offer-acceptance
//! (accept one, expire competitors, advance the job state, create the
//! notifications — all atomically).

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::{JobRow, OfferRow};

#[derive(Clone)]
pub struct Offers {
    pool: PgPool,
}

impl Offers {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        job_id: Uuid,
        mechanic_id: Uuid,
        quoted_price_minor: i64,
        estimated_arrival_minutes: i32,
        message: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<OfferRow, PersistenceError> {
        sqlx::query_as::<_, OfferRow>(
            r#"
            INSERT INTO mechanic_offers (id, job_id, mechanic_id, quoted_price_minor,
                                         estimated_arrival_minutes, message, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(mechanic_id)
        .bind(quoted_price_minor)
        .bind(estimated_arrival_minutes)
        .bind(message)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    /// Offers joined with mechanic display data for comparison UIs.
    pub async fn list_by_job(&self, job_id: Uuid) -> Result<Vec<OfferRow>, PersistenceError> {
        sqlx::query_as::<_, OfferRow>(
            r#"
            SELECT o.id, o.job_id, o.mechanic_id, o.quoted_price_minor, o.currency,
                   o.estimated_arrival_minutes, o.message, o.status, o.expires_at,
                   o.created_at, o.updated_at,
                   COALESCE(m.display_name, u.display_name) AS mechanic_name,
                   m.rating_average AS mechanic_rating,
                   m.completed_jobs AS mechanic_completed_jobs
              FROM mechanic_offers o
              JOIN mechanics m ON m.id = o.mechanic_id
              JOIN users u ON u.id = m.user_id
             WHERE o.job_id = $1
             ORDER BY o.created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn find_by_id(&self, offer_id: Uuid) -> Result<OfferRow, PersistenceError> {
        sqlx::query_as::<_, OfferRow>("SELECT * FROM mechanic_offers WHERE id = $1")
            .bind(offer_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn list_by_mechanic(
        &self,
        mechanic_id: Uuid,
        limit: i64,
    ) -> Result<Vec<OfferRow>, PersistenceError> {
        sqlx::query_as::<_, OfferRow>(
            "SELECT * FROM mechanic_offers WHERE mechanic_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(mechanic_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    /// The full acceptance transaction. Returns the updated job and the
    /// count of competing offers expired alongside the accepted one.
    pub async fn accept_offer_transaction(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        offer_id: Uuid,
        job_id: Uuid,
        customer_user_id: Uuid,
        from_status: toolfix_contracts::JobStatus,
        to_status: toolfix_contracts::JobStatus,
    ) -> Result<(JobRow, u64), PersistenceError> {
        // 1. Lock the job and verify ownership + status.
        let job = sqlx::query_as::<_, JobRow>(
            "SELECT * FROM assistance_jobs WHERE id = $1 FOR UPDATE",
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(PersistenceError::NotFound)?;
        if job.customer_user_id != customer_user_id {
            return Err(PersistenceError::NotFound); // do not leak existence
        }
        if job.status != from_status.as_str() {
            return Err(PersistenceError::InvalidData(format!(
                "job is in status {} and cannot accept an offer",
                job.status
            )));
        }

        // 2. Lock and accept the chosen offer (must belong to this job and
        //    still be pending).
        let offer = sqlx::query_as::<_, OfferRow>(
            "SELECT * FROM mechanic_offers WHERE id = $1 AND job_id = $2 FOR UPDATE",
        )
        .bind(offer_id)
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(PersistenceError::NotFound)?;
        if offer.status != "pending" {
            return Err(PersistenceError::InvalidData(format!(
                "offer is {} and can no longer be accepted",
                offer.status
            )));
        }
        sqlx::query("UPDATE mechanic_offers SET status = 'accepted', updated_at = now() WHERE id = $1")
            .bind(offer_id)
            .execute(&mut **tx)
            .await?;

        // 3. Expire competing offers.
        let expired = sqlx::query(
            "UPDATE mechanic_offers SET status = 'expired', updated_at = now() WHERE job_id = $1 AND id <> $2 AND status = 'pending'",
        )
        .bind(job_id)
        .bind(offer_id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

        // 4. Advance the job state (history row included).
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE assistance_jobs
               SET status = $2, selected_mechanic_id = $3, updated_at = $4
             WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(to_status.as_str())
        .bind(offer.mechanic_id)
        .bind(now)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO job_status_history (id, job_id, from_status, to_status, changed_by_user_id, reason)
            VALUES ($1, $2, $3, $4, $5, 'offer accepted')
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(from_status.as_str())
        .bind(to_status.as_str())
        .bind(customer_user_id)
        .execute(&mut **tx)
        .await?;

        // 5. Offer event trail.
        sqlx::query(
            "INSERT INTO offer_events (id, offer_id, event_kind, actor_user_id, detail) VALUES ($1, $2, 'accepted', $3, NULL)",
        )
        .bind(Uuid::now_v7())
        .bind(offer_id)
        .bind(customer_user_id)
        .execute(&mut **tx)
        .await?;

        let updated_job = sqlx::query_as::<_, JobRow>(
            "SELECT * FROM assistance_jobs WHERE id = $1",
        )
        .bind(job_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok((updated_job, expired))
    }

    pub async fn withdraw(
        &self,
        offer_id: Uuid,
        mechanic_id: Uuid,
    ) -> Result<bool, PersistenceError> {
        let result = sqlx::query(
            "UPDATE mechanic_offers SET status = 'withdrawn', updated_at = now() WHERE id = $1 AND mechanic_id = $2 AND status = 'pending'",
        )
        .bind(offer_id)
        .bind(mechanic_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Expires stale pending offers; returns the job ids affected so the
    /// runtime can re-evaluate job state afterwards.
    pub async fn expire_stale(&self, now: DateTime<Utc>) -> Result<Vec<Uuid>, PersistenceError> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE mechanic_offers SET status = 'expired', updated_at = now()
             WHERE status = 'pending' AND expires_at < $1
            RETURNING job_id
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    pub async fn insert_offer_event(
        &self,
        offer_id: Uuid,
        event_kind: &str,
        actor_user_id: Option<Uuid>,
        detail: Option<&str>,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "INSERT INTO offer_events (id, offer_id, event_kind, actor_user_id, detail) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::now_v7())
        .bind(offer_id)
        .bind(event_kind)
        .bind(actor_user_id)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

//! Payments repository.

use sqlx::PgPool;
use toolfix_contracts::{Currency, PaymentMethod, PaymentStatus};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::PaymentRow;

#[derive(Clone)]
pub struct Payments {
    pool: PgPool,
}

impl Payments {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        job_id: Uuid,
        amount_minor: i64,
        currency: Currency,
        method: PaymentMethod,
        provider: &str,
        provider_payment_id: Option<&str>,
    ) -> Result<PaymentRow, PersistenceError> {
        sqlx::query_as::<_, PaymentRow>(
            r#"
            INSERT INTO payments (id, job_id, amount_minor, currency, method, status, provider, provider_payment_id)
            VALUES ($1, $2, $3, $4, $5, 'initiated', $6, $7)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(job_id)
        .bind(amount_minor)
        .bind(currency.as_str())
        .bind(method.as_str())
        .bind(provider)
        .bind(provider_payment_id)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn mark_confirmed(
        &self,
        payment_id: Uuid,
        receipt_number: &str,
    ) -> Result<PaymentRow, PersistenceError> {
        sqlx::query_as::<_, PaymentRow>(
            "UPDATE payments SET status = 'confirmed', receipt_number = $2, confirmed_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(payment_id)
        .bind(receipt_number)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn mark_failed(&self, payment_id: Uuid) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE payments SET status = 'failed' WHERE id = $1")
            .bind(payment_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_transaction(
        &self,
        payment_id: Uuid,
        event_kind: &str,
        detail: Option<&str>,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "INSERT INTO payment_transactions (id, payment_id, event_kind, detail) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(payment_id)
        .bind(event_kind)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, payment_id: Uuid) -> Result<PaymentRow, PersistenceError> {
        sqlx::query_as::<_, PaymentRow>("SELECT * FROM payments WHERE id = $1")
            .bind(payment_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn status_count(&self, status: PaymentStatus) -> Result<i64, PersistenceError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM payments WHERE status = $1")
            .bind(status.as_str())
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}

//! Payments repository.

use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::PaymentRow;
use crate::{dual, Db};
use toolfix_contracts::{Currency, PaymentMethod};

#[derive(Clone)]
pub struct Payments {
    db: Db,
}

impl Payments {
    pub fn new(db: Db) -> Self {
        Self { db }
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
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PaymentRow>(
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
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn mark_confirmed(
        &self,
        payment_id: Uuid,
        receipt_number: &str,
    ) -> Result<PaymentRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PaymentRow>(
                "UPDATE payments SET status = 'confirmed', receipt_number = $2, confirmed_at = CURRENT_TIMESTAMP WHERE id = $1 RETURNING *"
            )
            .bind(payment_id)
            .bind(receipt_number)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn mark_failed(&self, payment_id: Uuid) -> Result<(), PersistenceError> {
        dual!(
            &self.db,
            |e| sqlx::query("UPDATE payments SET status = 'failed' WHERE id = $1")
                .bind(payment_id)
                .execute(e)
                .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    pub async fn add_transaction(
        &self,
        payment_id: Uuid,
        event_kind: &str,
        detail: Option<&str>,
    ) -> Result<(), PersistenceError> {
        dual!(
            &self.db,
            |e| sqlx::query(
                "INSERT INTO payment_transactions (id, payment_id, event_kind, detail) VALUES ($1, $2, $3, $4)"
            )
            .bind(Uuid::now_v7())
            .bind(payment_id)
            .bind(event_kind)
            .bind(detail)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    pub async fn find_by_id(&self, payment_id: Uuid) -> Result<PaymentRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PaymentRow>("SELECT * FROM payments WHERE id = $1")
                .bind(payment_id)
                .fetch_one(e)
                .await
        )?)
    }
}

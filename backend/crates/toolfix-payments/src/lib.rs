//! Payment provider abstraction.
//!
//! The domain never couples to a specific PSP. The stub/cash provider ships
//! first; Razorpay (or another PSP) implements the same trait later.

use async_trait::async_trait;
use chrono::Utc;
use toolfix_contracts::{Currency, PaymentMethod, PaymentStatus};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("payment provider error: {0}")]
    Provider(String),
}

#[derive(Debug, Clone)]
pub struct PaymentIntent {
    pub provider_payment_id: String,
    pub amount_minor: i64,
    pub currency: Currency,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone)]
pub struct PaymentReceipt {
    pub receipt_number: String,
    pub confirmed_at: chrono::DateTime<Utc>,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn initiate(
        &self,
        job_id: Uuid,
        amount_minor: i64,
        currency: Currency,
        method: PaymentMethod,
    ) -> Result<PaymentIntent, PaymentError>;

    async fn confirm(
        &self,
        provider_payment_id: &str,
        amount_minor: i64,
    ) -> Result<PaymentReceipt, PaymentError>;
}

/// Development provider: "cash on completion". Initiation immediately
/// produces a pending intent; confirmation succeeds synchronously.
pub struct CashStubProvider;

#[async_trait]
impl PaymentProvider for CashStubProvider {
    fn name(&self) -> &'static str {
        "cash_stub"
    }

    async fn initiate(
        &self,
        job_id: Uuid,
        amount_minor: i64,
        currency: Currency,
        method: PaymentMethod,
    ) -> Result<PaymentIntent, PaymentError> {
        Ok(PaymentIntent {
            provider_payment_id: format!("cash_{job_id}_{}", Uuid::now_v7()),
            amount_minor,
            currency,
            method,
            status: PaymentStatus::Initiated,
        })
    }

    async fn confirm(
        &self,
        provider_payment_id: &str,
        _amount_minor: i64,
    ) -> Result<PaymentReceipt, PaymentError> {
        Ok(PaymentReceipt {
            receipt_number: format!("RCPT-{provider_payment_id}"),
            confirmed_at: Utc::now(),
        })
    }
}

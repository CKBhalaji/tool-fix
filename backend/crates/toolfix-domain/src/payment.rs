//! Payment lifecycle rules.

use chrono::{DateTime, Utc};
use toolfix_contracts::{Currency, PaymentMethod, PaymentStatus};
use uuid::Uuid;

/// A payment against a job. The lifecycle is
/// initiated -> confirmed (or failed), with receipt generation on confirm.
#[derive(Debug, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub job_id: Uuid,
    pub amount_minor: i64,
    pub currency: Currency,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub provider: String,
    pub receipt_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

impl Payment {
    pub fn confirmable(&self) -> bool {
        self.status == PaymentStatus::Initiated
    }

    pub fn receipt(&self) -> Option<String> {
        match self.status {
            PaymentStatus::Confirmed => self.receipt_number.clone(),
            _ => None,
        }
    }
}

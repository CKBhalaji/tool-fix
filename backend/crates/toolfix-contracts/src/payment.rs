//! Payment DTOs.

use crate::enums::{Currency, PaymentMethod, PaymentStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentInitiateRequest {
    pub method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentResponse {
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

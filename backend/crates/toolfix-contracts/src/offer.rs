//! Mechanic offer DTOs.

use crate::enums::{Currency, OfferStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct OfferCreateRequest {
    pub quoted_price_minor: i64,
    pub estimated_arrival_minutes: i32,
    pub message: Option<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct OfferResponse {
    pub id: Uuid,
    pub job_id: Uuid,
    pub mechanic_id: Uuid,
    pub mechanic_name: Option<String>,
    pub mechanic_rating: Option<f64>,
    pub mechanic_completed_jobs: i64,
    pub quoted_price_minor: i64,
    pub currency: Currency,
    pub estimated_arrival_minutes: i32,
    pub message: Option<String>,
    pub status: OfferStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct OfferSelectionResponse {
    pub job: crate::job::JobResponse,
    pub accepted_offer: OfferResponse,
    pub expired_competing_offers: u64,
}

//! Rating/review DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct RatingCreateRequest {
    pub job_id: Uuid,
    pub score: i16,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatingResponse {
    pub id: Uuid,
    pub job_id: Uuid,
    pub mechanic_id: Uuid,
    pub customer_user_id: Uuid,
    pub score: i16,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MechanicRatingSummary {
    pub mechanic_id: Uuid,
    pub average_score: Option<f64>,
    pub total_ratings: i64,
}

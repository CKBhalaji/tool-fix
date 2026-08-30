//! Response DTOs for auth and mechanic feed surfaces.

use crate::user::UserResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Returned by `POST /api/v1/auth/google` for clients that want the consent
/// URL as JSON; `GET /api/v1/auth/google/login` redirects directly.
#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct GoogleLoginUrlResponse {
    pub login_url: String,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct LoggedInResponse {
    pub user: UserResponse,
    pub redirect: String,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct MechanicProfileResponse {
    pub mechanic_id: Uuid,
    pub user: UserResponse,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub service_area_km: f64,
    pub supported_vehicle_kinds: Vec<crate::enums::VehicleKind>,
    pub repair_categories: Vec<crate::enums::RepairCategory>,
    pub experience_years: Option<i32>,
    pub availability_status: crate::enums::AvailabilityStatus,
    pub rating_average: Option<f64>,
    pub completed_jobs: i64,
    pub is_verified: bool,
    pub current_latitude: Option<f64>,
    pub current_longitude: Option<f64>,
    pub location_updated_at: Option<DateTime<Utc>>,
}

/// A job visible to an eligible mechanic in the nearby-requests feed.
#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct MechanicFeedItem {
    pub job_id: Uuid,
    pub breakdown_id: Uuid,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub distance_km: f64,
    pub vehicle_kind: Option<crate::enums::VehicleKind>,
    pub diagnosis: Option<crate::breakdown::AgentDiagnosisDto>,
    pub price_estimate: Option<crate::breakdown::PriceEstimateDto>,
    pub offer_window_expires_at: Option<DateTime<Utc>>,
    pub notified: bool,
}

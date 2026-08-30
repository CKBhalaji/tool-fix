//! Vehicle DTOs.

use crate::enums::VehicleKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct VehicleCreateRequest {
    pub vehicle_kind: VehicleKind,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub registration_number: Option<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct VehicleUpdateRequest {
    pub vehicle_kind: Option<VehicleKind>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub registration_number: Option<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct VehicleResponse {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub vehicle_kind: VehicleKind,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub registration_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

//! Request DTOs that don't belong to a single entity module.

use crate::enums::{AvailabilityStatus, RepairCategory, VehicleKind};
use serde::{Deserialize};

/// Mechanic onboarding: authentication establishes identity; this collects
/// the mechanic-specific business profile. The backend decides the role.
#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct MechanicOnboardingRequest {
    pub phone: Option<String>,
    pub display_name: Option<String>,
    pub city: Option<String>,
    pub service_area_km: Option<f64>,
    pub supported_vehicle_kinds: Vec<VehicleKind>,
    pub repair_categories: Vec<RepairCategory>,
    pub experience_years: Option<i32>,
    pub hourly_signal: Option<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct MechanicProfileUpdateRequest {
    pub city: Option<String>,
    pub service_area_km: Option<f64>,
    pub supported_vehicle_kinds: Option<Vec<VehicleKind>>,
    pub repair_categories: Option<Vec<RepairCategory>>,
    pub experience_years: Option<i32>,
    pub display_name: Option<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct AvailabilityRequest {
    pub status: AvailabilityStatus,
}

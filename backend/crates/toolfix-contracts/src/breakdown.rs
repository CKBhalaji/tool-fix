//! Breakdown report DTOs.

use crate::enums::MediaKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(utoipa::ToSchema, Debug, Clone, Deserialize)]
pub struct BreakdownCreateRequest {
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct BreakdownMediaResponse {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub media_kind: MediaKind,
    pub storage_key: String,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiagnosisDto {
    pub possible_issue: String,
    pub confidence: f64,
    pub severity: crate::enums::Severity,
    pub recommended_service: String,
    pub repair_category: crate::enums::RepairCategory,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    pub requires_towing: bool,
    pub reasoning_summary: String,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct PriceEstimateDto {
    pub repair_category: crate::enums::RepairCategory,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    pub currency: crate::enums::Currency,
    pub source: crate::enums::EstimateSource,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// AI output is advisory only; this response is informational.
#[derive(utoipa::ToSchema, Debug, Clone, Serialize)]
pub struct DiagnosisResponse {
    pub breakdown_id: Uuid,
    pub diagnosis: AgentDiagnosisDto,
    pub price_estimate: PriceEstimateDto,
}

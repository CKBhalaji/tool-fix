//! Assistance job DTOs.

use crate::enums::JobStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub customer_user_id: Uuid,
    pub vehicle_id: Uuid,
    pub status: JobStatus,
    pub selected_mechanic_id: Option<Uuid>,
    pub final_amount_minor: Option<i64>,
    pub currency: crate::enums::Currency,
    pub offer_window_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobStatusHistoryEntry {
    pub id: Uuid,
    pub job_id: Uuid,
    pub from_status: Option<JobStatus>,
    pub to_status: JobStatus,
    pub changed_by_user_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Event broadcast over the job WebSocket channel.
#[derive(Debug, Clone, Serialize)]
pub struct JobEvent {
    pub job_id: Uuid,
    pub kind: String,
    pub payload: serde_json::Value,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MechanicLocationPing {
    pub job_id: Option<Uuid>,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MechanicLocationSample {
    pub latitude: f64,
    pub longitude: f64,
    pub recorded_at: DateTime<Utc>,
}

//! Location event types: what gets persisted when a party reports position.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use toolfix_contracts::LatLng;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationSubject {
    Customer,
    Mechanic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEvent {
    pub id: Uuid,
    pub subject: LocationSubject,
    pub subject_id: Uuid,
    pub job_id: Option<Uuid>,
    pub point: LatLng,
    pub accuracy_m: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

//! Vehicle entity helpers.

use chrono::{DateTime, Utc};
use toolfix_contracts::VehicleKind;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub vehicle_kind: VehicleKind,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub registration_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Vehicle {
    pub fn display_name(&self) -> String {
        match (&self.make, &self.model) {
            (Some(make), Some(model)) => format!("{make} {model}"),
            (Some(make), None) => make.clone(),
            (None, Some(model)) => model.clone(),
            (None, None) => self.vehicle_kind.to_string(),
        }
    }
}

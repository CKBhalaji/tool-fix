//! Breakdown report entity and input validation.

use chrono::{DateTime, Utc};
use toolfix_contracts::LatLng;
use uuid::Uuid;

use crate::error::DomainError;

#[derive(Debug, Clone)]
pub struct Breakdown {
    pub id: Uuid,
    pub user_id: Uuid,
    pub vehicle_id: Uuid,
    pub location: LatLng,
    pub address: Option<String>,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Validates the customer-supplied portion of a breakdown report.
pub fn validate_new_breakdown(
    problem_description: &str,
    symptoms: &[String],
    location: LatLng,
) -> Result<(), DomainError> {
    let description = problem_description.trim();
    if description.len() < 10 {
        return Err(DomainError::Validation(
            "problem_description must be at least 10 characters".into(),
        ));
    }
    if description.len() > 4000 {
        return Err(DomainError::Validation(
            "problem_description must be at most 4000 characters".into(),
        ));
    }
    if symptoms.len() > 20 {
        return Err(DomainError::Validation(
            "at most 20 vehicle_symptoms are allowed".into(),
        ));
    }
    if location.latitude == 0.0 && location.longitude == 0.0 {
        return Err(DomainError::Validation(
            "location (0,0) looks invalid; please share your real position".into(),
        ));
    }
    Ok(())
}

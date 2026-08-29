//! Mechanic entity and business rules.

use chrono::{DateTime, Utc};
use toolfix_contracts::{AvailabilityStatus, LatLng, RepairCategory, VehicleKind};
use uuid::Uuid;

use crate::error::DomainError;

#[derive(Debug, Clone)]
pub struct Mechanic {
    pub id: Uuid,
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub service_area_km: f64,
    pub supported_vehicle_kinds: Vec<VehicleKind>,
    pub repair_categories: Vec<RepairCategory>,
    pub experience_years: Option<i32>,
    pub availability_status: AvailabilityStatus,
    pub rating_average: Option<f64>,
    pub completed_jobs: i64,
    pub is_verified: bool,
    pub current_latitude: Option<f64>,
    pub current_longitude: Option<f64>,
    pub location_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Mechanic {
    /// Whether the mechanic can, in principle, service this vehicle and
    /// repair category. Availability/distance are handled by matching.
    pub fn supports(&self, vehicle: Option<VehicleKind>, category: RepairCategory) -> bool {
        if !self.is_verified || self.availability_status == AvailabilityStatus::Offline {
            return false;
        }
        let vehicle_ok = match vehicle {
            Some(kind) => self.supported_vehicle_kinds.contains(&kind),
            None => true,
        };
        vehicle_ok && self.repair_categories.contains(&category)
    }

    pub fn can_receive_more_jobs(&self, active_jobs: i64, max_active: i64) -> bool {
        active_jobs < max_active
    }

    pub fn set_location(&mut self, latitude: f64, longitude: f64, now: DateTime<Utc>) -> Result<(), DomainError> {
        let point = LatLng::new(latitude, longitude)
            .map_err(DomainError::Validation)?;
        self.current_latitude = Some(point.latitude);
        self.current_longitude = Some(point.longitude);
        self.location_updated_at = Some(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mechanic() -> Mechanic {
        Mechanic {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            display_name: None,
            phone: None,
            city: None,
            service_area_km: 10.0,
            supported_vehicle_kinds: vec![VehicleKind::Motorcycle, VehicleKind::Scooter],
            repair_categories: vec![RepairCategory::Battery, RepairCategory::Tyre],
            experience_years: Some(3),
            availability_status: AvailabilityStatus::Online,
            rating_average: Some(4.5),
            completed_jobs: 42,
            is_verified: true,
            current_latitude: None,
            current_longitude: None,
            location_updated_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn supports_checks_vehicle_and_category() {
        let m = mechanic();
        assert!(m.supports(Some(VehicleKind::Scooter), RepairCategory::Battery));
        assert!(!m.supports(Some(VehicleKind::Car), RepairCategory::Battery));
        assert!(!m.supports(Some(VehicleKind::Scooter), RepairCategory::Engine));
    }

    #[test]
    fn offline_mechanics_do_not_support_anything() {
        let mut m = mechanic();
        m.availability_status = AvailabilityStatus::Offline;
        assert!(!m.supports(Some(VehicleKind::Scooter), RepairCategory::Battery));
    }
}

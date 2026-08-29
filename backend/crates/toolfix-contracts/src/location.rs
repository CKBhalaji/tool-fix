//! Geographic primitives shared across crates.

use serde::{Deserialize, Serialize};

/// A WGS84 coordinate. Latitude must be within [-90, 90] and longitude
/// within [-180, 180]; construction is fallible for that reason.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

impl LatLng {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(format!("latitude out of range: {latitude}"));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(format!("longitude out of range: {longitude}"));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }
}

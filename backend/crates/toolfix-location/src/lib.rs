//! Geospatial primitives and helpers shared by matching, tracking, and the
//! persistence layer. Pure math: no I/O, no provider coupling.

use toolfix_contracts::LatLng;

pub mod events;

/// Earth radius in meters (mean).
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Great-circle distance in meters between two points (haversine).
pub fn haversine_distance_m(a: LatLng, b: LatLng) -> f64 {
    let lat1 = a.latitude.to_radians();
    let lat2 = b.latitude.to_radians();
    let dlat = (b.latitude - a.latitude).to_radians();
    let dlon = (b.longitude - a.longitude).to_radians();

    let sin_dlat = (dlat / 2.0).sin();
    let sin_dlon = (dlon / 2.0).sin();
    let h = sin_dlat * sin_dlat + lat1.cos() * lat2.cos() * sin_dlon * sin_dlon;
    2.0 * EARTH_RADIUS_M * h.sqrt().asin().clamp(0.0, std::f64::consts::PI)
}

/// Approximate degrees of latitude/longitude spanning `radius_m` — used to
/// build bounding-box WHERE clauses that an index can serve before exact
/// haversine ranking.
pub fn bounding_box(center: LatLng, radius_m: f64) -> (f64, f64, f64, f64) {
    let lat_delta = radius_m / 111_320.0;
    let lon_delta = radius_m / (111_320.0 * center.latitude.max(1.0).to_radians().cos().max(0.01));
    (
        center.latitude - lat_delta,
        center.latitude + lat_delta,
        center.longitude - lon_delta,
        center.longitude + lon_delta,
    )
}

/// Rough ETA in minutes given distance and an average city speed (km/h).
pub fn eta_minutes(distance_m: f64, avg_speed_kmh: f64) -> i64 {
    if avg_speed_kmh <= 0.0 {
        return i64::MAX;
    }
    ((distance_m / 1000.0) / avg_speed_kmh * 60.0).ceil() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_is_sane() {
        // Bengaluru MG Road -> Indiranagar ~ 3-4 km.
        let a = LatLng::new(12.9757, 77.6068).unwrap();
        let b = LatLng::new(12.9784, 77.6408).unwrap();
        let d = haversine_distance_m(a, b);
        assert!(d > 2_500.0 && d < 4_500.0, "distance was {d}");
        assert_eq!(haversine_distance_m(a, a), 0.0);
    }

    #[test]
    fn bounding_box_contains_center() {
        let c = LatLng::new(12.97, 77.60).unwrap();
        let (min_lat, max_lat, min_lng, max_lng) = bounding_box(c, 3_000.0);
        assert!(min_lat < c.latitude && c.latitude < max_lat);
        assert!(min_lng < c.longitude && c.longitude < max_lng);
    }

    #[test]
    fn eta_scales_with_speed() {
        assert_eq!(eta_minutes(10_000.0, 30.0), 20);
    }
}

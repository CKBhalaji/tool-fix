//! Location domain helpers (accuracy/sample hygiene).

use chrono::{DateTime, Duration, Utc};
use toolfix_contracts::LatLng;

/// A mechanic or customer position sample.
#[derive(Debug, Clone)]
pub struct LocationSample {
    pub point: LatLng,
    pub accuracy_m: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

impl LocationSample {
    /// Samples with absurd accuracy (GPS still converging) are stored but
    /// flagged so tracking UIs can smooth over them.
    pub fn is_low_confidence(&self) -> bool {
        matches!(self.accuracy_m, Some(a) if a > 200.0)
    }

    /// Live tracking UIs should ignore samples older than this.
    pub fn is_stale(&self, now: DateTime<Utc>, max_age: Duration) -> bool {
        now - self.recorded_at > max_age
    }
}

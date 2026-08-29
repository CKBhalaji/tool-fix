//! Deterministic mechanic matching.
//!
//! Pure business logic — no LLM anywhere in this crate. Pipeline:
//! bounding-box search -> exact distance -> availability/capability
//! filters -> deterministic ranking -> notify top-N.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use toolfix_contracts::{RepairCategory, VehicleKind};
use toolfix_persistence::models::MechanicCandidateRow;
use toolfix_persistence::repositories::{Mechanics, Notifications};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingConfig {
    /// Radii tried in order when too few candidates are found (meters).
    pub radius_stages_m: Vec<f64>,
    /// Maximum mechanics notified per job.
    pub max_notified: usize,
    /// Feed lookback window (seconds) for the mechanic requests feed.
    pub feed_lookback_secs: i64,
    /// Average mechanic travel speed for ETA estimates (km/h).
    pub avg_speed_kmh: f64,
    /// Active-job cap per mechanic used during matching.
    pub max_active_jobs_per_mechanic: i64,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            // 0-3 km, 3-5 km, 5-10 km expansion per the product spec.
            radius_stages_m: vec![3_000.0, 5_000.0, 10_000.0],
            max_notified: 10,
            feed_lookback_secs: 6 * 3600,
            avg_speed_kmh: 25.0,
            max_active_jobs_per_mechanic: 3,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MatchingError {
    #[error("database error: {0}")]
    Database(#[from] toolfix_persistence::PersistenceError),
    #[error("notification error: {0}")]
    Notification(#[from] toolfix_notifications::NotificationError),
}

/// One ranked candidate.
#[derive(Debug, Clone, Serialize)]
pub struct RankedMechanic {
    pub mechanic_id: uuid::Uuid,
    pub distance_m: f64,
    pub eta_minutes: i64,
    pub rating: Option<f64>,
    pub completed_jobs: i64,
    pub active_jobs: i64,
    pub score: f64,
}

/// Deterministic ranking: closer is better, higher rating is better, more
/// completed jobs (experience) is better, fewer active jobs is better.
/// Plain constant weights keep the behavior predictable and testable.
pub fn rank_score(
    distance_m: f64,
    rating: Option<f64>,
    completed_jobs: i64,
    active_jobs: i64,
) -> f64 {
    let distance_score = 1.0 / (1.0 + distance_m / 1_000.0);
    let rating_score = rating.unwrap_or(3.0).clamp(0.0, 5.0) / 5.0;
    let experience_score = (completed_jobs as f64).min(500.0) / 500.0;
    let load_score = 1.0 / (1.0 + active_jobs as f64);
    // Distance dominates; rating/experience break ties.
    0.55 * distance_score + 0.25 * rating_score + 0.10 * experience_score + 0.10 * load_score
}

/// Filters and ranks raw candidate rows (pure function, unit-tested here).
/// A `None` category means the fault is unknown — mechanics are matched by
/// proximity and vehicle alone.
pub fn rank_candidates(
    candidates: Vec<MechanicCandidateRow>,
    vehicle_kind: Option<VehicleKind>,
    category: Option<RepairCategory>,
    avg_speed_kmh: f64,
    max_active_jobs_per_mechanic: i64,
    active_jobs_by_mechanic: &std::collections::HashMap<uuid::Uuid, i64>,
) -> Vec<RankedMechanic> {
    let mut ranked: Vec<RankedMechanic> = candidates
        .into_iter()
        .filter_map(|row| {
            let mechanic = toolfix_domain::mechanic::Mechanic {
                id: row.mechanic.id,
                user_id: row.mechanic.user_id,
                display_name: row.mechanic.display_name.clone(),
                phone: row.mechanic.phone.clone(),
                city: row.mechanic.city.clone(),
                service_area_km: row.mechanic.service_area_km as f64,
                supported_vehicle_kinds: row.mechanic.vehicle_kinds().ok()?,
                repair_categories: row.mechanic.categories().ok()?,
                experience_years: row.mechanic.experience_years.map(i32::from),
                availability_status: row.mechanic.availability().ok()?,
                rating_average: row.mechanic.rating_average.map(f64::from),
                completed_jobs: row.mechanic.completed_jobs,
                is_verified: row.mechanic.is_verified,
                current_latitude: row.mechanic.current_latitude,
                current_longitude: row.mechanic.current_longitude,
                location_updated_at: row.mechanic.location_updated_at,
                created_at: row.mechanic.created_at,
                updated_at: row.mechanic.updated_at,
            };

            if mechanic.availability_status == toolfix_contracts::AvailabilityStatus::Offline {
                return None;
            }
            // The mechanic must be able to reach the job within their own
            // declared service area.
            if mechanic.service_area_km > 0.0
                && row.distance_m > mechanic.service_area_km * 1_000.0
            {
                return None;
            }
            if let Some(category) = category
                && !mechanic.supports(vehicle_kind, category)
            {
                return None;
            }
            let active = active_jobs_by_mechanic
                .get(&mechanic.id)
                .copied()
                .unwrap_or(0);
            if !mechanic.can_receive_more_jobs(active, max_active_jobs_per_mechanic) {
                return None;
            }

            let eta = toolfix_location::eta_minutes(row.distance_m, avg_speed_kmh);
            let score = rank_score(
                row.distance_m,
                mechanic.rating_average,
                mechanic.completed_jobs,
                active,
            );
            Some(RankedMechanic {
                mechanic_id: mechanic.id,
                distance_m: row.distance_m,
                eta_minutes: eta,
                rating: mechanic.rating_average,
                completed_jobs: mechanic.completed_jobs,
                active_jobs: active,
                score,
            })
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.mechanic_id.cmp(&b.mechanic_id))
    });
    ranked
}

#[derive(Clone)]
pub struct MatchingService {
    mechanics: Mechanics,
    notifications: Notifications,
    provider: Arc<dyn toolfix_notifications::NotificationProvider>,
    config: MatchingConfig,
}

impl MatchingService {
    pub fn new(
        mechanics: Mechanics,
        notifications: Notifications,
        provider: Arc<dyn toolfix_notifications::NotificationProvider>,
        config: MatchingConfig,
    ) -> Self {
        Self {
            mechanics,
            notifications,
            provider,
            config,
        }
    }

    pub fn config(&self) -> &MatchingConfig {
        &self.config
    }

    /// Finds and ranks eligible mechanics for a breakdown. The radius
    /// expands stage by stage until at least one candidate is found.
    /// A `None` category matches by proximity/vehicle only.
    pub async fn find_mechanics(
        &self,
        breakdown: &toolfix_persistence::models::BreakdownRow,
        vehicle_kind: Option<VehicleKind>,
        category: Option<RepairCategory>,
    ) -> Result<Vec<RankedMechanic>, MatchingError> {
        let max_radius = self
            .config
            .radius_stages_m
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        let center = toolfix_contracts::LatLng::new(breakdown.latitude, breakdown.longitude)
            .map_err(|e| {
                MatchingError::Database(toolfix_persistence::PersistenceError::InvalidData(e))
            })?;
        let (min_lat, max_lat, min_lng, max_lng) =
            toolfix_location::bounding_box(center, max_radius);

        // Fetch once for the largest radius; stage expansion is applied
        // against the ranked result to keep behavior deterministic.
        let candidates = self
            .mechanics
            .find_candidates_in_box(
                breakdown.latitude,
                breakdown.longitude,
                min_lat,
                max_lat,
                min_lng,
                max_lng,
                max_radius,
                200,
            )
            .await?;

        let mut active_jobs = std::collections::HashMap::new();
        for candidate in &candidates {
            let count = self
                .mechanics
                .count_active_jobs(candidate.mechanic.id)
                .await?;
            active_jobs.insert(candidate.mechanic.id, count);
        }

        let ranked = rank_candidates(
            candidates,
            vehicle_kind,
            category,
            self.config.avg_speed_kmh,
            self.config.max_active_jobs_per_mechanic,
            &active_jobs,
        );

        // Expand through the radius stages until the first stage with
        // candidates; cap at max_notified.
        let mut notified: Vec<RankedMechanic> = Vec::new();
        for stage in &self.config.radius_stages_m {
            notified = ranked
                .iter()
                .filter(|m| m.distance_m <= *stage)
                .cloned()
                .collect();
            if !notified.is_empty() {
                break;
            }
        }
        notified.truncate(self.config.max_notified);
        Ok(notified)
    }

    /// Notifies the selected mechanics: persists the notification intent,
    /// dispatches through the provider, records the delivery result.
    pub async fn notify_mechanics(
        &self,
        job_id: uuid::Uuid,
        breakdown: &toolfix_persistence::models::BreakdownRow,
        mechanics: &[RankedMechanic],
    ) -> Result<(), MatchingError> {
        for mechanic in mechanics {
            let Ok(user_id) = self.mechanics.user_id_of(mechanic.mechanic_id).await else {
                continue;
            };
            let distance_km = (mechanic.distance_m / 1000.0 * 10.0).round() / 10.0;
            let payload = serde_json::json!({
                "job_id": job_id.to_string(),
                "breakdown_id": breakdown.id.to_string(),
                "distance_km": distance_km,
                "eta_minutes": mechanic.eta_minutes,
                "description": breakdown.problem_description,
            });
            let notification = self
                .notifications
                .create(
                    user_id,
                    toolfix_contracts::NotificationKind::NewBreakdownNearby.as_str(),
                    toolfix_contracts::NotificationChannel::Push.as_str(),
                    Some(job_id),
                    payload.clone(),
                )
                .await?;

            let result = self
                .provider
                .send(toolfix_notifications::DeliveryRequest {
                    notification_id: notification.id,
                    recipient_user_id: user_id,
                    kind: toolfix_contracts::NotificationKind::NewBreakdownNearby,
                    channel: toolfix_contracts::NotificationChannel::Push,
                    title: "New breakdown nearby".to_string(),
                    body: format!("Breakdown {distance_km} km away — open the request to bid."),
                    payload,
                })
                .await;
            match result {
                Ok(delivery) => {
                    self.notifications
                        .mark_sent(
                            notification.id,
                            self.provider.name(),
                            &delivery.detail,
                            chrono::Utc::now(),
                        )
                        .await?;
                }
                Err(err) => {
                    tracing::warn!(error = %err, "notification delivery failed");
                    self.notifications
                        .mark_failed(notification.id, self.provider.name(), &err.to_string())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

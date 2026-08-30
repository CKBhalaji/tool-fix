//! Mechanic handlers: profile, availability, location, requests feed.

use axum::extract::State;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::{authorization, AuthUser};
use toolfix_contracts::request::{
    AvailabilityRequest, MechanicOnboardingRequest, MechanicProfileUpdateRequest,
};
use toolfix_contracts::response::MechanicProfileResponse;
use toolfix_persistence::models::MechanicRow;

pub(crate) fn profile_response(
    row: MechanicRow,
    user: toolfix_persistence::models::UserRow,
) -> Result<MechanicProfileResponse, ApiError> {
    let kinds = row.vehicle_kinds()?;
    let categories = row.categories()?;
    let availability = row.availability()?;
    Ok(MechanicProfileResponse {
        mechanic_id: row.id,
        user: crate::handlers::auth::to_user_response(&user)?,
        display_name: row.display_name,
        phone: row.phone,
        city: row.city,
        service_area_km: row.service_area_km as f64,
        supported_vehicle_kinds: kinds,
        repair_categories: categories,
        experience_years: row.experience_years.map(i32::from),
        availability_status: availability,
        rating_average: row.rating_average.map(f64::from),
        completed_jobs: row.completed_jobs,
        is_verified: row.is_verified,
        current_latitude: row.current_latitude,
        current_longitude: row.current_longitude,
        location_updated_at: row.location_updated_at,
    })
}

/// GET /api/v1/mechanics/me
pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MechanicProfileResponse>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    let row = state.repos.mechanics.find_by_user_id(auth.user_id).await?;
    let user = state.repos.users.find_by_id(auth.user_id).await?;
    Ok(Json(profile_response(row, user)?))
}

/// POST /api/v1/mechanics/onboarding — collects the mechanic business
/// profile (authentication and onboarding are separate concerns).
pub async fn onboarding(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<MechanicOnboardingRequest>,
) -> Result<Json<MechanicProfileResponse>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    if !(0.0..=500.0).contains(&request.service_area_km.unwrap_or(10.0)) {
        return Err(ApiError::validation("service_area_km must be 0-500"));
    }
    let row = state
        .repos
        .mechanics
        .upsert_for_user(
            auth.user_id,
            request.display_name.as_deref(),
            request.phone.as_deref(),
            request.city.as_deref(),
            request.service_area_km.unwrap_or(10.0),
            &request.supported_vehicle_kinds,
            &request.repair_categories,
            request.experience_years.map(|y| y as i16),
        )
        .await?;
    let user = state.repos.users.find_by_id(auth.user_id).await?;
    Ok(Json(profile_response(row, user)?))
}

/// PATCH /api/v1/mechanics/me
pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<MechanicProfileUpdateRequest>,
) -> Result<Json<MechanicProfileResponse>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    let mechanic = state.repos.mechanics.find_by_user_id(auth.user_id).await?;
    if let Some(area) = request.service_area_km
        && !(0.0..=500.0).contains(&area)
    {
        return Err(ApiError::validation("service_area_km must be 0-500"));
    }
    let row = state
        .repos
        .mechanics
        .update_profile(
            mechanic.id,
            request.display_name.as_deref(),
            request.city.as_deref(),
            request.service_area_km,
            request.supported_vehicle_kinds.as_deref(),
            request.repair_categories.as_deref(),
            request.experience_years.map(|y| y as i16),
        )
        .await?;
    let user = state.repos.users.find_by_id(auth.user_id).await?;
    Ok(Json(profile_response(row, user)?))
}

/// POST /api/v1/mechanics/availability — go online/offline.
pub async fn set_availability(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AvailabilityRequest>,
) -> Result<Json<MechanicProfileResponse>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    let mechanic = state.repos.mechanics.find_by_user_id(auth.user_id).await?;
    let row = state
        .repos
        .mechanics
        .set_availability(mechanic.id, request.status)
        .await?;
    let user = state.repos.users.find_by_id(auth.user_id).await?;
    Ok(Json(profile_response(row, user)?))
}

/// POST /api/v1/mechanics/location — availability pings (no active job).
pub async fn push_location(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(ping): Json<toolfix_contracts::job::MechanicLocationPing>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    state
        .jobs
        .submit_mechanic_location(
            auth.user_id,
            ping.job_id,
            ping.latitude,
            ping.longitude,
            ping.accuracy_m,
        )
        .await?;
    Ok(Json(serde_json::json!({ "accepted": true })))
}

/// GET /api/v1/mechanics/requests?latitude=..&longitude=.. — nearby feed.
pub async fn requests(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<toolfix_contracts::response::MechanicFeedItem>>, ApiError> {
    authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    let latitude: f64 = params
        .get("latitude")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ApiError::validation("latitude query parameter required"))?;
    let longitude: f64 = params
        .get("longitude")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ApiError::validation("longitude query parameter required"))?;
    let items = state
        .jobs
        .mechanic_feed(auth.user_id, latitude, longitude)
        .await?;
    Ok(Json(items))
}

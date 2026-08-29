//! Vehicle CRUD handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::vehicle::{VehicleCreateRequest, VehicleResponse, VehicleUpdateRequest};
use toolfix_persistence::models::VehicleRow;

pub(crate) fn to_response(row: VehicleRow) -> Result<VehicleResponse, ApiError> {
    Ok(VehicleResponse {
        id: row.id,
        owner_user_id: row.owner_user_id,
        vehicle_kind: row.kind()?,
        make: row.make,
        model: row.model,
        year: row.year.map(i32::from),
        registration_number: row.registration_number,
        created_at: row.created_at,
    })
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<VehicleCreateRequest>,
) -> Result<Json<VehicleResponse>, ApiError> {
    if request.year.is_some_and(|y| !(1950..=2100).contains(&y)) {
        return Err(ApiError::validation("year must be between 1950 and 2100"));
    }
    let row = state
        .repos
        .vehicles
        .create(
            auth.user_id,
            request.vehicle_kind,
            request.make.as_deref(),
            request.model.as_deref(),
            request.year.map(|y| y as i16),
            request.registration_number.as_deref(),
        )
        .await?;
    Ok(Json(to_response(row)?))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<VehicleResponse>>, ApiError> {
    let rows = state.repos.vehicles.list_by_owner(auth.user_id).await?;
    rows.into_iter()
        .map(to_response)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vehicle_id): Path<uuid::Uuid>,
) -> Result<Json<VehicleResponse>, ApiError> {
    let row = state.repos.vehicles.find_by_id(vehicle_id).await?;
    if row.owner_user_id != auth.user_id {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    Ok(Json(to_response(row)?))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vehicle_id): Path<uuid::Uuid>,
    Json(request): Json<VehicleUpdateRequest>,
) -> Result<Json<VehicleResponse>, ApiError> {
    if request.year.is_some_and(|y| !(1950..=2100).contains(&y)) {
        return Err(ApiError::validation("year must be between 1950 and 2100"));
    }
    let row = state
        .repos
        .vehicles
        .update(
            vehicle_id,
            auth.user_id,
            request.vehicle_kind,
            request.make.as_deref(),
            request.model.as_deref(),
            request.year.map(|y| y as i16),
            request.registration_number.as_deref(),
        )
        .await?;
    Ok(Json(to_response(row)?))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(vehicle_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state
        .repos
        .vehicles
        .delete_owned(vehicle_id, auth.user_id)
        .await?;
    if !deleted {
        return Err(ApiError::from(toolfix_persistence::PersistenceError::NotFound));
    }
    Ok(StatusCode::NO_CONTENT)
}

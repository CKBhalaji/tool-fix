//! Job handlers: read + mechanic execution transitions.

use axum::extract::{Path, State};
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::job::{JobResponse, JobStatusHistoryEntry, MechanicLocationPing};
use toolfix_contracts::{Currency, JobStatus};
use toolfix_persistence::models::JobRow;

pub(crate) fn job_response(row: &JobRow) -> Result<JobResponse, ApiError> {
    Ok(JobResponse {
        id: row.id,
        breakdown_id: row.breakdown_id,
        customer_user_id: row.customer_user_id,
        vehicle_id: row.vehicle_id,
        status: row.status()?,
        selected_mechanic_id: row.selected_mechanic_id,
        final_amount_minor: row.final_amount_minor,
        currency: Currency::Inr,
        offer_window_expires_at: row.offer_window_expires_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

/// GET /api/v1/jobs/{id}
pub async fn get_job(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = state.repos.jobs.find_by_id(job_id).await?;
    // Only the customer or the selected mechanic may view the job.
    let is_customer = job.customer_user_id == auth.user_id;
    let is_selected = match job.selected_mechanic_id {
        Some(mid) => state
            .repos
            .mechanics
            .user_id_of(mid)
            .await
            .map(|uid| uid == auth.user_id)
            .unwrap_or(false),
        None => false,
    };
    let is_admin = auth.role == toolfix_contracts::UserRole::Admin;
    if !(is_customer || is_selected || is_admin) {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    Ok(Json(job_response(&job)?))
}

/// GET /api/v1/jobs/{id}/status-history
pub async fn status_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<JobStatusHistoryEntry>>, ApiError> {
    // Access control mirrors get_job.
    let job = state.repos.jobs.find_by_id(job_id).await?;
    let is_customer = job.customer_user_id == auth.user_id;
    let is_selected = match job.selected_mechanic_id {
        Some(mid) => state
            .repos
            .mechanics
            .user_id_of(mid)
            .await
            .map(|uid| uid == auth.user_id)
            .unwrap_or(false),
        None => false,
    };
    if !(is_customer || is_selected) {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }

    let rows = state.jobs.status_history(job_id).await?;
    rows.into_iter()
        .map(|row| {
            Ok(JobStatusHistoryEntry {
                id: row.id,
                job_id: row.job_id,
                from_status: row.from_status()?,
                to_status: row.to_status()?,
                changed_by_user_id: row.changed_by_user_id,
                reason: row.reason,
                created_at: row.created_at,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

/// POST /api/v1/jobs/{id}/arrived (mechanic)
pub async fn mark_arrived(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let updated = state.jobs.mark_arrived(auth.user_id, job_id).await?;
    Ok(Json(job_response(&updated)?))
}

/// POST /api/v1/jobs/{id}/start-repair (mechanic)
pub async fn start_repair(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let updated = state.jobs.start_repair(auth.user_id, job_id).await?;
    Ok(Json(job_response(&updated)?))
}

/// POST /api/v1/jobs/{id}/complete (mechanic)
pub async fn complete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let updated = state.jobs.complete_repair(auth.user_id, job_id).await?;
    Ok(Json(job_response(&updated)?))
}

/// POST /api/v1/jobs/{id}/start-travel (mechanic; also exposed via mechanic
/// handlers as the post-acceptance action).
pub async fn start_travel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let updated = state.jobs.start_travel(auth.user_id, job_id).await?;
    Ok(Json(job_response(&updated)?))
}

/// GET /api/v1/jobs — the caller's jobs (customer or mechanic view).
pub async fn list_mine(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<JobResponse>>, ApiError> {
    let rows = match auth.role {
        toolfix_contracts::UserRole::Mechanic => state.jobs.mechanic_jobs(auth.user_id).await?,
        _ => state.jobs.my_jobs(auth.user_id, 50).await?,
    };
    rows.iter()
        .map(job_response)
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

/// PUT /api/v1/jobs/{id}/location — mechanic location ping during a job
/// (also works without a job for general availability pings).
pub async fn push_location(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(_job_id): Path<uuid::Uuid>,
    Json(ping): Json<MechanicLocationPing>,
) -> Result<Json<serde_json::Value>, ApiError> {
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

/// Compile-time guard: the status enum stays in sync with the state
/// machine's happy path.
#[allow(dead_code)]
fn _assert_happy_path() -> Result<(), toolfix_domain::TransitionError> {
    toolfix_domain::validate_transition(JobStatus::Created, JobStatus::Analyzing)
}

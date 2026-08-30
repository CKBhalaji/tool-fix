//! Offer (bidding) handlers.

use axum::extract::{Path, State};
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::offer::{OfferCreateRequest, OfferResponse, OfferSelectionResponse};
use toolfix_contracts::Currency;
use toolfix_persistence::models::OfferRow;

fn offer_response(row: OfferRow) -> Result<OfferResponse, ApiError> {
    let status = row.status()?;
    Ok(OfferResponse {
        id: row.id,
        job_id: row.job_id,
        mechanic_id: row.mechanic_id,
        mechanic_name: row.mechanic_name,
        mechanic_rating: row.mechanic_rating.map(f64::from),
        mechanic_completed_jobs: row.mechanic_completed_jobs,
        quoted_price_minor: row.quoted_price_minor,
        currency: Currency::Inr,
        estimated_arrival_minutes: row.estimated_arrival_minutes,
        message: row.message,
        status,
        expires_at: row.expires_at,
        created_at: row.created_at,
    })
}

/// POST /api/v1/jobs/{job_id}/offers (mechanic)
#[utoipa::path(post, path = "/api/v1/jobs/{job_id}/offers", tag = "offers", operation_id = "offers_create", params(("job_id" = Uuid, Path)), request_body = OfferCreateRequest, responses((status = 200, body = OfferResponse)))]
pub async fn create_offer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
    Json(request): Json<OfferCreateRequest>,
) -> Result<Json<OfferResponse>, ApiError> {
    let row = state
        .jobs
        .create_offer(auth.user_id, job_id, request)
        .await?;
    Ok(Json(offer_response(row)?))
}

/// GET /api/v1/jobs/{job_id}/offers (customer compares; mechanic sees own)
#[utoipa::path(get, path = "/api/v1/jobs/{job_id}/offers", tag = "offers", operation_id = "offers_list", params(("job_id" = Uuid, Path)), responses((status = 200, body = [OfferResponse])))]
pub async fn list_offers(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<OfferResponse>>, ApiError> {
    let job = state.repos.jobs.find_by_id(job_id).await?;
    if job.customer_user_id != auth.user_id {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    state
        .jobs
        .list_offers(job_id)
        .await?
        .into_iter()
        .map(offer_response)
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

/// POST /api/v1/offers/{offer_id}/select (customer) — transactional.
#[utoipa::path(post, path = "/api/v1/offers/{offer_id}/select", tag = "offers", operation_id = "offers_select", params(("offer_id" = Uuid, Path)), responses((status = 200, body = OfferSelectionResponse)))]
pub async fn select_offer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(offer_id): Path<uuid::Uuid>,
) -> Result<Json<OfferSelectionResponse>, ApiError> {
    let outcome = state.jobs.select_offer(auth.user_id, offer_id).await?;
    Ok(Json(OfferSelectionResponse {
        job: crate::handlers::jobs::job_response(&outcome.job)?,
        accepted_offer: offer_response(outcome.accepted)?,
        expired_competing_offers: outcome.expired_competing,
    }))
}

/// POST /api/v1/offers/{offer_id}/withdraw (mechanic)
#[utoipa::path(post, path = "/api/v1/offers/{offer_id}/withdraw", tag = "offers", operation_id = "offers_withdraw", params(("offer_id" = Uuid, Path)), responses((status = 200)))]
pub async fn withdraw_offer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(offer_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.jobs.withdraw_offer(auth.user_id, offer_id).await?;
    Ok(Json(serde_json::json!({ "withdrawn": true })))
}

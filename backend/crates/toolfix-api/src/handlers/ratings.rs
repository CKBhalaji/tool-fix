//! Rating/review handlers.

use axum::extract::{Path, State};
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::rating::{RatingCreateRequest, RatingResponse};

fn rating_response(row: toolfix_persistence::models::RatingRow) -> RatingResponse {
    RatingResponse {
        id: row.id,
        job_id: row.job_id,
        mechanic_id: row.mechanic_id,
        customer_user_id: row.customer_user_id,
        score: row.score,
        comment: row.comment,
        created_at: row.created_at,
    }
}

/// POST /api/v1/ratings (customer rates a completed job)
#[utoipa::path(post, path = "/api/v1/ratings", tag = "ratings", operation_id = "ratings_create", request_body = RatingCreateRequest, responses((status = 200, body = RatingResponse)))]
pub async fn create_rating(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RatingCreateRequest>,
) -> Result<Json<RatingResponse>, ApiError> {
    let rating = state.jobs.rate_job(auth.user_id, request).await?;
    Ok(Json(rating_response(rating)))
}

/// GET /api/v1/mechanics/{mechanic_id}/reviews
#[utoipa::path(get, path = "/api/v1/ratings/mechanics/{mechanic_id}/reviews", tag = "ratings", operation_id = "ratings_mechanic_reviews", params(("mechanic_id" = Uuid, Path)), responses((status = 200, body = [RatingResponse])))]
pub async fn mechanic_reviews(
    State(state): State<AppState>,
    Path(mechanic_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<RatingResponse>>, ApiError> {
    let rows = state.jobs.mechanic_reviews(mechanic_id).await?;
    Ok(Json(rows.into_iter().map(rating_response).collect()))
}

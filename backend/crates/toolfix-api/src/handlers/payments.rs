//! Payment handlers.

use axum::extract::{Path, State};
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::payment::{PaymentInitiateRequest, PaymentResponse};

/// POST /api/v1/jobs/{job_id}/pay (customer)
#[utoipa::path(post, path = "/api/v1/jobs/{job_id}/pay", tag = "payments", operation_id = "payments_pay", params(("job_id" = Uuid, Path)), request_body = PaymentInitiateRequest, responses((status = 200, body = PaymentResponse)))]
pub async fn pay(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
    Json(request): Json<PaymentInitiateRequest>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let payment = state
        .jobs
        .pay_job(auth.user_id, job_id, request.method)
        .await?;
    Ok(Json(payment))
}

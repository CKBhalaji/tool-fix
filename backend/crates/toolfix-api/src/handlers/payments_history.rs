//! Customer payment-history handlers.

use axum::extract::{Query, State};
use axum::Json;
use serde::Serialize;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;

#[derive(Serialize)]
pub struct PaymentView {
    pub payment_id: String,
    pub job_id: String,
    pub amount_minor: i64,
    pub method: String,
    pub status: String,
    pub receipt_number: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub job_status: String,
}

/// GET /api/v1/payments — the caller's own payment history.
pub async fn my_payments(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<PaymentView>>, ApiError> {
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let rows = state
        .repos
        .payments
        .list_for_customer(auth.user_id, limit)
        .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| PaymentView {
                payment_id: row.id.to_string(),
                job_id: row.job_id.to_string(),
                amount_minor: row.amount_minor,
                method: row.method,
                status: row.status,
                receipt_number: row.receipt_number,
                created_at: row.created_at,
                confirmed_at: row.confirmed_at,
                job_status: row.job_status,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct MechanicPaymentView {
    pub payment_id: String,
    pub job_id: String,
    pub amount_minor: i64,
    pub method: String,
    pub status: String,
    pub receipt_number: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub job_status: String,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
}

/// GET /api/v1/mechanics/payments — the signed-in mechanic's incoming
/// payments (jobs they were selected for), with payment status + receipts.
pub async fn my_earnings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<MechanicPaymentView>>, ApiError> {
    toolfix_auth::authorization::ensure_role(&auth, toolfix_contracts::UserRole::Mechanic)
        .map_err(ApiError::from)?;
    let rows = state
        .repos
        .payments
        .list_for_mechanic(auth.user_id, 200)
        .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| MechanicPaymentView {
                payment_id: row.id.to_string(),
                job_id: row.job_id.to_string(),
                amount_minor: row.amount_minor,
                method: row.method,
                status: row.status,
                receipt_number: row.receipt_number,
                created_at: row.created_at,
                confirmed_at: row.confirmed_at,
                job_status: row.job_status,
                customer_email: row.customer_email,
                customer_name: row.customer_name,
            })
            .collect(),
    ))
}

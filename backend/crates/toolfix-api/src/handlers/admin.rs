//! Admin console handlers. Every route except the login requires the
//! ADMIN role (derived from the ToolFix session, never the browser).

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use crate::{ApiError, AppState};
use toolfix_auth::cookies::{append_cookies, auth_cookies};
use toolfix_auth::{authorization, AuthUser};
use toolfix_contracts::response::{LoggedInResponse, MechanicProfileResponse};
use toolfix_persistence::models::{AdminJobRow, UserRow};

/// POST /api/v1/admin/login — static credentials → ADMIN session cookies.
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminLoginRequest>,
) -> Result<axum::response::Response, ApiError> {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| raw.split(',').next())
        .map(str::trim)
        .map(str::to_string);

    let session = state
        .auth
        .admin_login(
            &request.email,
            &request.password,
            user_agent.as_deref(),
            ip_address.as_deref(),
        )
        .await?;

    let response = (
        StatusCode::OK,
        Json(LoggedInResponse {
            user: crate::handlers::auth::to_user_response(&session.user)?,
            redirect: "/admin".to_string(),
        }),
    )
        .into_response();
    let mut response = response;
    append_cookies(
        response.headers_mut(),
        &auth_cookies(state.auth.config(), &session.access_token, &session.refresh_token),
    );
    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

fn require_admin(auth: &AuthUser) -> Result<(), ApiError> {
    authorization::ensure_role(auth, toolfix_contracts::UserRole::Admin)
        .map_err(ApiError::from)
}

/// GET /api/v1/admin/overview
pub async fn overview(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<toolfix_persistence::models::OverviewRow>, ApiError> {
    require_admin(&auth)?;
    Ok(Json(state.repos.admin_overview().await?))
}

#[derive(serde::Serialize)]
pub struct AdminUserView {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub status: String,
    pub onboarding_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn user_view(row: UserRow) -> Result<AdminUserView, ApiError> {
    Ok(AdminUserView {
        id: row.id.to_string(),
        email: row.email,
        display_name: row.display_name,
        phone: row.phone,
        role: row.role,
        status: row.status,
        onboarding_status: row.onboarding_status,
        created_at: row.created_at,
    })
}

/// GET /api/v1/admin/users?role=customer|mechanic|admin&limit=200
pub async fn users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminUserView>>, ApiError> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let role = params.get("role").map(String::as_str);
    let rows = state.repos.users.list_all(role, limit).await?;
    rows.into_iter()
        .map(user_view)
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct SetStatusRequest {
    pub status: String,
}

/// POST /api/v1/admin/users/{id}/status { status: active|suspended|deleted }
pub async fn set_user_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<uuid::Uuid>,
    Json(request): Json<SetStatusRequest>,
) -> Result<StatusCode, ApiError> {
    require_admin(&auth)?;
    let status = toolfix_contracts::UserStatus::parse(&request.status)
        .ok_or_else(|| ApiError::validation("status must be active|suspended|deleted"))?;
    state.repos.users.set_status(user_id, status).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/admin/mechanics
pub async fn mechanics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<MechanicProfileResponse>>, ApiError> {
    require_admin(&auth)?;
    let rows = state.repos.mechanics.list_all(500).await?;
    let mut profiles = Vec::with_capacity(rows.len());
    for row in rows {
        let user = state.repos.users.find_by_id(row.user_id).await?;
        profiles.push(crate::handlers::mechanics::profile_response(row, user)?);
    }
    Ok(Json(profiles))
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub verified: bool,
}

/// POST /api/v1/admin/mechanics/{id}/verify { verified: bool }
pub async fn verify_mechanic(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(mechanic_id): Path<uuid::Uuid>,
    Json(request): Json<VerifyRequest>,
) -> Result<StatusCode, ApiError> {
    require_admin(&auth)?;
    state
        .repos
        .mechanics
        .set_verified(mechanic_id, request.verified)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Serialize)]
pub struct AdminJobView {
    pub job_id: String,
    pub status: toolfix_contracts::JobStatus,
    pub final_amount_minor: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub breakdown_id: String,
    pub problem_description: String,
    pub latitude: f64,
    pub longitude: f64,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub mechanic_id: Option<String>,
    pub mechanic_name: Option<String>,
}

fn job_view(row: AdminJobRow) -> Result<AdminJobView, ApiError> {
    Ok(AdminJobView {
        job_id: row.job_id.to_string(),
        status: row.job_status()?,
        final_amount_minor: row.final_amount_minor,
        created_at: row.job_created_at,
        breakdown_id: row.breakdown_id.to_string(),
        problem_description: row.problem_description,
        latitude: row.latitude,
        longitude: row.longitude,
        customer_email: row.customer_email,
        customer_name: row.customer_name,
        mechanic_id: row.mechanic_id.map(|m| m.to_string()),
        mechanic_name: row.mechanic_name,
    })
}

/// GET /api/v1/admin/jobs?limit=200
pub async fn jobs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminJobView>>, ApiError> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    state
        .repos
        .jobs
        .list_all_admin(limit)
        .await?
        .into_iter()
        .map(job_view)
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

/// GET /api/v1/admin/payments?limit=200
pub async fn payments(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminPaymentView>>, ApiError> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    state
        .repos
        .payments
        .list_all_admin(limit)
        .await?
        .into_iter()
        .map(payment_view)
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

#[derive(serde::Serialize)]
pub struct AdminPaymentView {
    pub payment_id: String,
    pub job_id: String,
    pub amount_minor: i64,
    pub method: String,
    pub status: String,
    pub provider: String,
    pub receipt_number: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub job_status: String,
}

fn payment_view(row: toolfix_persistence::models::AdminPaymentRow) -> Result<AdminPaymentView, ApiError> {
    Ok(AdminPaymentView {
        payment_id: row.payment_id.to_string(),
        job_id: row.job_id.to_string(),
        amount_minor: row.amount_minor,
        method: row.method,
        status: row.status,
        provider: row.provider,
        receipt_number: row.receipt_number,
        created_at: row.created_at,
        confirmed_at: row.confirmed_at,
        customer_email: row.customer_email,
        customer_name: row.customer_name,
        job_status: row.job_status,
    })
}

/// GET /api/v1/admin/agent-runs?limit=200 — the AI usage table.
pub async fn agent_runs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AgentRunView>>, ApiError> {
    require_admin(&auth)?;
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    state
        .repos
        .agents
        .list_all_admin(limit)
        .await?
        .into_iter()
        .map(|row| {
            Ok(AgentRunView {
                run_id: row.run_id.to_string(),
                breakdown_id: row.breakdown_id.to_string(),
                job_id: row.job_id.map(|j| j.to_string()),
                kind: row.kind,
                provider: row.provider,
                model: row.model,
                status: row.status,
                error_message: row.error_message,
                latency_ms: row.latency_ms,
                created_at: row.created_at,
                problem_description: row.problem_description,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()
        .map(Json)
}

#[derive(serde::Serialize)]
pub struct AgentRunView {
    pub run_id: String,
    pub breakdown_id: String,
    pub job_id: Option<String>,
    pub kind: String,
    pub provider: String,
    pub model: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub latency_ms: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub problem_description: String,
}

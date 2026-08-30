//! Auth handlers: the only authentication surface the frontend sees.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use crate::{ApiError, AppState};
use toolfix_auth::cookies::{
    append_cookies, auth_cookies, clearing_cookies, oauth_handshake_clearing_cookies,
    oauth_handshake_cookies, read_cookie, REFRESH_COOKIE,
};
use toolfix_auth::{AuthError, AuthUser};
use toolfix_contracts::response::GoogleLoginUrlResponse;
use toolfix_contracts::user::{AuthMeResponse, UserResponse};
use toolfix_persistence::models::MechanicRow;

fn user_response(row: &toolfix_persistence::models::UserRow) -> Result<UserResponse, ApiError> {
    Ok(UserResponse {
        id: row.id,
        firebase_uid: row.firebase_uid.clone(),
        email: row.email.clone(),
        display_name: row.display_name.clone(),
        photo_url: row.photo_url.clone(),
        phone: row.phone.clone(),
        role: row.role()?,
        status: row.status()?,
        onboarding_status: row.onboarding_status()?,
        created_at: row.created_at,
    })
}

/// POST /api/v1/auth/google — returns the consent URL as JSON for clients
/// that want to drive the redirect from JavaScript.
#[utoipa::path(post, path = "/api/v1/auth/google", tag = "auth", operation_id = "auth_google_login_url", responses((status = 200, body = GoogleLoginUrlResponse)))]
pub async fn google_login_url(
    State(state): State<AppState>,
) -> Result<Json<GoogleLoginUrlResponse>, ApiError> {
    let login = state.auth.begin_google_login()?;
    Ok(Json(GoogleLoginUrlResponse { login_url: login.url }))
}

/// GET /api/v1/auth/google/login — 302 to Google's consent page with
/// OAuth state + PKCE verifier stored in short-lived HttpOnly cookies.
#[utoipa::path(get, path = "/api/v1/auth/google/login", tag = "auth", operation_id = "auth_google_login", responses((status = 302, description = "Redirect to Google consent")))]
pub async fn google_login(State(state): State<AppState>) -> Result<Response, ApiError> {
    let login = state.auth.begin_google_login()?;
    let mut headers = HeaderMap::new();
    append_cookies(&mut headers, &oauth_handshake_cookies(state.auth.config(), &login.state, &login.verifier));
    Ok((
        StatusCode::SEE_OTHER,
        headers,
        Redirect::to(login.url.as_str()),
    )
        .into_response())
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    #[allow(dead_code)]
    pub scope: Option<String>,
    pub error: Option<String>,
}

/// GET /api/v1/auth/google/callback — completes the code exchange
/// server-to-server, provisions Firebase server-side, creates the
/// ToolFix session, sets HttpOnly cookies, redirects to the frontend.
#[utoipa::path(get, path = "/api/v1/auth/google/callback", tag = "auth", operation_id = "auth_google_callback", responses((status = 302, description = "Redirect to frontend with session cookies")))]
pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(callback): Query<OAuthCallback>,
) -> Result<Response, ApiError> {
    if callback.error.is_some() || callback.code.is_none() || callback.state.is_none() {
        return Err(ApiError::from(AuthError::OAuth(
            "google returned an error or missing code/state".into(),
        )));
    }
    let expected_state = read_cookie(&headers, toolfix_auth::OAUTH_STATE_COOKIE)
        .ok_or(AuthError::StateMismatch)?;
    if expected_state != callback.state.as_deref().unwrap_or_default() {
        return Err(ApiError::from(AuthError::StateMismatch));
    }
    let verifier = read_cookie(&headers, toolfix_auth::OAUTH_VERIFIER_COOKIE)
        .ok_or(AuthError::StateMismatch)?;

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
        .complete_google_login(
            callback.code.as_deref().unwrap_or_default(),
            &verifier,
            user_agent.as_deref(),
            ip_address.as_deref(),
        )
        .await?;

    let mut headers = HeaderMap::new();
    append_cookies(&mut headers, &auth_cookies(state.auth.config(), &session.access_token, &session.refresh_token));
    append_cookies(&mut headers, &oauth_handshake_clearing_cookies(state.auth.config()));

    let redirect = format!("{}/auth/callback", state.auth.frontend_origin().trim_end_matches('/'));
    Ok((StatusCode::SEE_OTHER, headers, Redirect::to(redirect.as_str())).into_response())
}

/// POST /api/v1/auth/refresh — rotates the refresh token and issues a new
/// access token (both as new cookies).
#[utoipa::path(post, path = "/api/v1/auth/refresh", tag = "auth", operation_id = "auth_refresh", responses((status = 204)))]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<axum::response::Response, ApiError> {
    let token = read_cookie(&headers, REFRESH_COOKIE)
        .ok_or(AuthError::Unauthenticated)?;
    let session = state.auth.refresh(&token, None, None).await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    append_cookies(response.headers_mut(), &auth_cookies(state.auth.config(), &session.access_token, &session.refresh_token));
    Ok(response)
}

/// POST /api/v1/auth/logout — revokes the session and clears cookies.
#[utoipa::path(post, path = "/api/v1/auth/logout", tag = "auth", operation_id = "auth_logout", responses((status = 204)))]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<axum::response::Response, ApiError> {
    if let Some(token) = read_cookie(&headers, REFRESH_COOKIE) {
        state.auth.logout(&token).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    append_cookies(response.headers_mut(), &clearing_cookies(state.auth.config()));
    Ok(response)
}

/// GET /api/v1/auth/me
#[utoipa::path(get, path = "/api/v1/auth/me", tag = "auth", operation_id = "auth_me", responses((status = 200, body = AuthMeResponse)))]
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<AuthMeResponse>, ApiError> {
    let row = state.auth.me(auth.user_id).await?;
    let mechanic_id = state.repos.mechanics.find_by_user_id(auth.user_id).await.ok().map(|m: MechanicRow| m.id);
    Ok(Json(AuthMeResponse {
        user: user_response(&row)?,
        mechanic_id,
    }))
}

/// Shared mapper used by other handlers.
pub(crate) fn to_user_response(
    row: &toolfix_persistence::models::UserRow,
) -> Result<UserResponse, ApiError> {
    user_response(row)
}

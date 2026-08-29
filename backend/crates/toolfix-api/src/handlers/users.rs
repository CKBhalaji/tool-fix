//! User profile + onboarding handlers.

use axum::extract::State;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::user::{OnboardingRequest, UpdateProfileRequest, UserResponse};

use crate::handlers::auth::to_user_response;

/// POST /api/v1/users/onboarding — choose CUSTOMER or MECHANIC. The role is
/// decided server-side; ADMIN is never granted through the API.
pub async fn onboarding(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<OnboardingRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state
        .auth
        .complete_onboarding(
            auth.user_id,
            request.requested_role,
            request.phone.as_deref(),
            request.display_name.as_deref(),
        )
        .await?;

    // Creating the mechanic profile together with role onboarding keeps the
    // two consistent; it is created empty and filled via PATCH mechanics/me.
    if request.requested_role == toolfix_contracts::UserRole::Mechanic {
        state
            .repos
            .mechanics
            .upsert_for_user(
                auth.user_id,
                request.display_name.as_deref(),
                request.phone.as_deref(),
                None,
                10.0,
                &[],
                &[],
                None,
            )
            .await?;
    }

    Ok(Json(to_user_response(&user)?))
}

/// GET /api/v1/users/me
pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, ApiError> {
    let row = state.repos.users.find_by_id(auth.user_id).await?;
    Ok(Json(to_user_response(&row)?))
}

/// PATCH /api/v1/users/me
pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let row = state
        .repos
        .users
        .update_profile(
            auth.user_id,
            request.display_name.as_deref(),
            request.phone.as_deref(),
        )
        .await?;
    Ok(Json(to_user_response(&row)?))
}

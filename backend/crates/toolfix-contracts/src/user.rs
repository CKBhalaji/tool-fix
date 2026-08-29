//! User-facing auth/account DTOs.

use crate::enums::{OnboardingStatus, UserRole, UserStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub onboarding_status: OnboardingStatus,
    pub created_at: DateTime<Utc>,
}

/// Sent by the frontend after Google login to finish onboarding. The role
/// request is advisory: the backend decides the final role.
#[derive(Debug, Clone, Deserialize)]
pub struct OnboardingRequest {
    pub requested_role: UserRole,
    pub phone: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMeResponse {
    pub user: UserResponse,
    pub mechanic_id: Option<Uuid>,
}

//! User entity helpers.

use chrono::{DateTime, Utc};
use toolfix_contracts::{OnboardingStatus, UserRole, UserStatus};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
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
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn has_role(&self, role: UserRole) -> bool {
        self.role == role
    }
}

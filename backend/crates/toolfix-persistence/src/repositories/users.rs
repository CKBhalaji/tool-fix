//! Users repository.

use chrono::Utc;
use sqlx::PgPool;
use toolfix_contracts::{OnboardingStatus, UserRole, UserStatus};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::UserRow;

#[derive(Clone)]
pub struct Users {
    pool: PgPool,
}

impl Users {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Finds a user by external identity or creates one. The role is decided
    /// here (server-side), never by the client.
    pub async fn upsert_by_external_identity(
        &self,
        firebase_uid: &str,
        email: Option<&str>,
        display_name: Option<&str>,
        photo_url: Option<&str>,
    ) -> Result<UserRow, PersistenceError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, firebase_uid, email, display_name, photo_url, role, status, onboarding_status)
            VALUES ($1, $2, $3, $4, $5, 'customer', 'active', 'pending')
            ON CONFLICT (firebase_uid) DO UPDATE
                SET email = COALESCE(EXCLUDED.email, users.email),
                    display_name = COALESCE(EXCLUDED.display_name, users.display_name),
                    photo_url = COALESCE(EXCLUDED.photo_url, users.photo_url),
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(firebase_uid)
        .bind(email)
        .bind(display_name)
        .bind(photo_url)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn find_by_id(&self, user_id: Uuid) -> Result<UserRow, PersistenceError> {
        sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn find_by_firebase_uid(&self, uid: &str) -> Result<UserRow, PersistenceError> {
        sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE firebase_uid = $1")
            .bind(uid)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn set_role(&self, user_id: Uuid, role: UserRole) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE users SET role = $2, updated_at = now() WHERE id = $1")
            .bind(user_id)
            .bind(role.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_status(&self, user_id: Uuid, status: UserStatus) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE users SET status = $2, updated_at = now() WHERE id = $1")
            .bind(user_id)
            .bind(status.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_onboarding_status(
        &self,
        user_id: Uuid,
        status: OnboardingStatus,
    ) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE users SET onboarding_status = $2, updated_at = now() WHERE id = $1")
            .bind(user_id)
            .bind(status.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        display_name: Option<&str>,
        phone: Option<&str>,
    ) -> Result<UserRow, PersistenceError> {
        sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
               SET display_name = COALESCE($2, display_name),
                   phone = COALESCE($3, phone),
                   updated_at = $4
             WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(display_name)
        .bind(phone)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }
}

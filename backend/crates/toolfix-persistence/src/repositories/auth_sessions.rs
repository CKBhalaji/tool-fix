//! Auth session repository. Only refresh-token HASHES are stored.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::AuthSessionRow;
use crate::{dual, Db};

#[derive(Clone)]
pub struct AuthSessions {
    db: Db,
}

impl AuthSessions {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        user_id: Uuid,
        refresh_token_hash: &str,
        expires_at: DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<AuthSessionRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, AuthSessionRow>(
                r#"
                INSERT INTO auth_sessions (id, user_id, refresh_token_hash, expires_at, user_agent, ip_address)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(user_id)
            .bind(refresh_token_hash)
            .bind(expires_at)
            .bind(user_agent)
            .bind(ip_address)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn find_by_hash(
        &self,
        refresh_token_hash: &str,
    ) -> Result<AuthSessionRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, AuthSessionRow>(
                "SELECT * FROM auth_sessions WHERE refresh_token_hash = $1"
            )
            .bind(refresh_token_hash)
            .fetch_one(e)
            .await
        )?)
    }

    /// Rotation: re-hash the session to a new refresh token and stamp usage.
    pub async fn rotate(
        &self,
        session_id: Uuid,
        new_refresh_token_hash: &str,
    ) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query(
                "UPDATE auth_sessions SET refresh_token_hash = $2, last_used_at = $3 WHERE id = $1"
            )
            .bind(session_id)
            .bind(new_refresh_token_hash)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    pub async fn revoke(&self, session_id: Uuid) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query(
                "UPDATE auth_sessions SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL"
            )
            .bind(session_id)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    /// Reuse detection: a replayed refresh token revokes every session of
    /// that user.
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query(
                "UPDATE auth_sessions SET revoked_at = $2 WHERE user_id = $1 AND revoked_at IS NULL"
            )
            .bind(user_id)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    /// Best-effort sweep of expired sessions.
    pub async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64, PersistenceError> {
        let result = dual!(
            &self.db,
            |e| sqlx::query("DELETE FROM auth_sessions WHERE expires_at < $1")
                .bind(now)
                .execute(e)
                .await
            .map(|r| r.rows_affected())
        )?;
        Ok(result)
    }
}

//! Notifications repository.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::NotificationRow;

#[derive(Clone)]
pub struct Notifications {
    pool: PgPool,
}

impl Notifications {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        recipient_user_id: Uuid,
        kind: &str,
        channel: &str,
        job_id: Option<Uuid>,
        payload: serde_json::Value,
    ) -> Result<NotificationRow, PersistenceError> {
        sqlx::query_as::<_, NotificationRow>(
            r#"
            INSERT INTO notifications (id, recipient_user_id, kind, channel, job_id, payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(recipient_user_id)
        .bind(kind)
        .bind(channel)
        .bind(job_id)
        .bind(payload)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<NotificationRow>, PersistenceError> {
        sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE recipient_user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn claim_pending(&self, limit: i64) -> Result<Vec<NotificationRow>, PersistenceError> {
        // Delivery is at-least-once: rows are read then marked sent/failed.
        // A single dispatcher runs per process in the initial architecture;
        // multi-replica dedup can lease rows later.
        sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE status = 'pending' ORDER BY created_at LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn mark_sent(
        &self,
        notification_id: Uuid,
        provider: &str,
        detail: &str,
        sent_at: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "UPDATE notifications SET status = 'sent', provider = $2, delivery_detail = $3, sent_at = $4 WHERE id = $1",
        )
        .bind(notification_id)
        .bind(provider)
        .bind(detail)
        .bind(sent_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(
        &self,
        notification_id: Uuid,
        provider: &str,
        detail: &str,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "UPDATE notifications SET status = 'failed', provider = $2, delivery_detail = $3 WHERE id = $1",
        )
        .bind(notification_id)
        .bind(provider)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

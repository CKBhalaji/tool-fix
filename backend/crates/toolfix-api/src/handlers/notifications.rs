//! In-app notification inbox handlers.

use axum::extract::State;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct NotificationView {
    pub id: uuid::Uuid,
    pub kind: String,
    pub channel: String,
    pub job_id: Option<uuid::Uuid>,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/v1/notifications
#[utoipa::path(get, path = "/api/v1/notifications", tag = "notifications", operation_id = "notifications_list", responses((status = 200, body = [NotificationView])))]
pub async fn list_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<NotificationView>>, ApiError> {
    let rows = state
        .repos
        .notifications
        .list_for_user(auth.user_id, 50)
        .await?;
    let views = rows
        .into_iter()
        .map(|row| {
            let kind = row.kind()?.as_str().to_string();
            let channel = row.channel()?.as_str().to_string();
            let status = row.status()?.as_str().to_string();
            Ok(NotificationView {
                id: row.id,
                kind,
                channel,
                job_id: row.job_id,
                payload: row.payload_json(),
                status,
                created_at: row.created_at,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    Ok(Json(views))
}

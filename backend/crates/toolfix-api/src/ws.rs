//! Job WebSocket channel.
//!
//! Transport only: events mirror state that is already persisted in
//! PostgreSQL. A reconnecting client re-reads the job and history via the
//! REST API — the socket is never authoritative.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures::stream::StreamExt;
use futures::SinkExt;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::job::JobEvent;

/// GET /api/v1/ws/jobs/{job_id}?as=mechanic — authenticated upgrade.
/// The cookie rides along on the upgrade request.
pub async fn job_events(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<uuid::Uuid>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    // Authorization: the customer, the selected mechanic, or an admin.
    let job = state.repos.jobs.find_by_id(job_id).await?;
    let is_customer = job.customer_user_id == auth.user_id;
    let is_selected = match job.selected_mechanic_id {
        Some(mid) => state
            .repos
            .mechanics
            .user_id_of(mid)
            .await
            .map(|uid| uid == auth.user_id)
            .unwrap_or(false),
        None => false,
    };
    if !(is_customer || is_selected || auth.role == toolfix_contracts::UserRole::Admin) {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }

    Ok(upgrade.on_upgrade(move |socket| handle_socket(socket, state, job_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, job_id: uuid::Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = state.jobs.events().subscribe();

    let mut push_task = tokio::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if event.job_id != job_id {
                        continue;
                    }
                    let text = serde_json::to_string(&event).unwrap_or_else(|_| {
                        serde_json::to_string(&JobEvent {
                            job_id,
                            kind: "parse_error".into(),
                            payload: serde_json::json!({}),
                            at: chrono::Utc::now(),
                        })
                        .unwrap()
                    });
                    if sender.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::debug!(skipped, "ws subscriber lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Drain client messages; any client-initiated close ends the session.
    let mut drain_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {
            // Client messages are not interpreted.
        }
    });

    tokio::select! {
        _ = &mut push_task => drain_task.abort(),
        _ = &mut drain_task => push_task.abort(),
    }
}

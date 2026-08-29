//! Route registration.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use axum::extract::DefaultBodyLimit;

use crate::AppState;

/// Simple text metrics (process-level counters are added later).
async fn metrics() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "toolfix-api",
        "status": "ok"
    }))
}

async fn health_live() -> StatusCode {
    StatusCode::OK
}

/// Readiness checks the database dependency.
async fn health_ready(State(state): State<crate::AppState>) -> StatusCode {
    match state.repos.ping().await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub fn router(state: AppState, cors: tower_http::cors::CorsLayer) -> Router {
    let auth_routes = Router::new()
        .route("/google", post(crate::handlers::auth::google_login_url))
        .route("/google/login", get(crate::handlers::auth::google_login))
        .route(
            "/google/callback",
            get(crate::handlers::auth::google_callback),
        )
        .route("/refresh", post(crate::handlers::auth::refresh))
        .route("/logout", post(crate::handlers::auth::logout))
        .route("/me", get(crate::handlers::auth::me));

    let user_routes = Router::new()
        .route("/me", get(crate::handlers::users::get_me))
        .route("/me", patch(crate::handlers::users::update_me))
        .route(
            "/onboarding",
            post(crate::handlers::users::onboarding),
        );

    let vehicle_routes = Router::new()
        .route("/", post(crate::handlers::vehicles::create))
        .route("/", get(crate::handlers::vehicles::list))
        .route(
            "/{vehicle_id}",
            get(crate::handlers::vehicles::get_one)
                .patch(crate::handlers::vehicles::update)
                .delete(crate::handlers::vehicles::delete),
        );

    let breakdown_routes = Router::new()
        .route("/", post(crate::handlers::breakdowns::create))
        .route(
            "/{id}",
            get(crate::handlers::breakdowns::get_one),
        )
        .route(
            "/{id}/cancel",
            post(crate::handlers::breakdowns::cancel),
        )
        .route(
            "/{id}/media",
            post(crate::handlers::breakdowns::upload_media),
        )
        .route(
            "/{id}/media/{media_id}/content",
            get(crate::handlers::breakdowns::download_media),
        )
        .route(
            "/{id}/diagnosis",
            get(crate::handlers::breakdowns::diagnosis),
        );

    let mechanic_routes = Router::new()
        .route(
            "/me",
            get(crate::handlers::mechanics::get_me)
                .patch(crate::handlers::mechanics::update_me),
        )
        .route(
            "/onboarding",
            post(crate::handlers::mechanics::onboarding),
        )
        .route(
            "/availability",
            post(crate::handlers::mechanics::set_availability),
        )
        .route(
            "/location",
            post(crate::handlers::mechanics::push_location),
        )
        .route(
            "/requests",
            get(crate::handlers::mechanics::requests),
        );

    let job_routes = Router::new()
        .route("/", get(crate::handlers::jobs::list_mine))
        .route(
            "/{id}",
            get(crate::handlers::jobs::get_job),
        )
        .route(
            "/{id}/start-travel",
            post(crate::handlers::jobs::start_travel),
        )
        .route(
            "/{id}/arrived",
            post(crate::handlers::jobs::mark_arrived),
        )
        .route(
            "/{id}/start-repair",
            post(crate::handlers::jobs::start_repair),
        )
        .route(
            "/{id}/complete",
            post(crate::handlers::jobs::complete),
        )
        .route(
            "/{id}/status-history",
            get(crate::handlers::jobs::status_history),
        )
        .route(
            "/{id}/pay",
            post(crate::handlers::payments::pay),
        )
        .route(
            "/{id}/location",
            post(crate::handlers::jobs::push_location),
        )
        .route(
            "/{job_id}/offers",
            get(crate::handlers::offers::list_offers)
                .post(crate::handlers::offers::create_offer),
        );

    let offer_routes = Router::new()
        .route(
            "/{offer_id}/select",
            post(crate::handlers::offers::select_offer),
        )
        .route(
            "/{offer_id}/withdraw",
            post(crate::handlers::offers::withdraw_offer),
        );

    let rating_routes = Router::new()
        .route("/", post(crate::handlers::ratings::create_rating))
        .route(
            "/mechanics/{mechanic_id}/reviews",
            get(crate::handlers::ratings::mechanic_reviews),
        );

    let notification_routes =
        Router::new().route("/", get(crate::handlers::notifications::list_notifications));

    let agent_routes = Router::new().route(
        "/diagnose",
        post(crate::handlers::agent::diagnose),
    );

    let ws_routes =
        Router::new().route("/jobs/{job_id}", get(crate::ws::job_events));

    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/metrics", get(metrics))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/users", user_routes)
        .nest("/api/v1/vehicles", vehicle_routes)
        .nest("/api/v1/breakdowns", breakdown_routes)
        .nest("/api/v1/mechanics", mechanic_routes)
        .nest("/api/v1/jobs", job_routes)
        .nest("/api/v1/offers", offer_routes)
        .nest("/api/v1/ratings", rating_routes)
        .nest("/api/v1/notifications", notification_routes)
        .nest("/api/v1/agent", agent_routes)
        .nest("/api/v1/ws", ws_routes)
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .layer(cors)
        .with_state(state)
}

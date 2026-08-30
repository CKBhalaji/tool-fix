//! Route registration.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use axum::extract::DefaultBodyLimit;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

#[derive(OpenApi)]
#[openapi(
    info(title = "ToolFix API", description = "On-demand roadside assistance marketplace", version = "0.1.0"),
    paths(
        crate::handlers::auth::google_login_url,
        crate::handlers::auth::google_login,
        crate::handlers::auth::google_callback,
        crate::handlers::auth::refresh,
        crate::handlers::auth::logout,
        crate::handlers::auth::me,
        crate::handlers::users::onboarding,
        crate::handlers::users::get_me,
        crate::handlers::users::update_me,
        crate::handlers::vehicles::create,
        crate::handlers::vehicles::list,
        crate::handlers::vehicles::get_one,
        crate::handlers::vehicles::update,
        crate::handlers::vehicles::delete,
        crate::handlers::breakdowns::create,
        crate::handlers::breakdowns::get_one,
        crate::handlers::breakdowns::cancel,
        crate::handlers::breakdowns::upload_media,
        crate::handlers::breakdowns::download_media,
        crate::handlers::breakdowns::diagnosis,
        crate::handlers::mechanics::get_me,
        crate::handlers::mechanics::onboarding,
        crate::handlers::mechanics::update_me,
        crate::handlers::mechanics::set_availability,
        crate::handlers::mechanics::push_location,
        crate::handlers::mechanics::requests,
        crate::handlers::jobs::list_mine,
        crate::handlers::jobs::get_job,
        crate::handlers::jobs::status_history,
        crate::handlers::jobs::mark_arrived,
        crate::handlers::jobs::start_repair,
        crate::handlers::jobs::complete,
        crate::handlers::jobs::start_travel,
        crate::handlers::jobs::push_location,
        crate::handlers::offers::create_offer,
        crate::handlers::offers::list_offers,
        crate::handlers::offers::select_offer,
        crate::handlers::offers::withdraw_offer,
        crate::handlers::payments::pay,
        crate::handlers::ratings::create_rating,
        crate::handlers::ratings::mechanic_reviews,
        crate::handlers::notifications::list_notifications,
        crate::handlers::payments_history::my_payments,
        crate::handlers::payments_history::my_earnings,
        crate::handlers::agent::diagnose,
        crate::handlers::admin::login,
        crate::handlers::admin::overview,
        crate::handlers::admin::users,
        crate::handlers::admin::set_user_status,
        crate::handlers::admin::mechanics,
        crate::handlers::admin::verify_mechanic,
        crate::handlers::admin::jobs,
        crate::handlers::admin::payments,
        crate::handlers::admin::agent_runs,
    ),
    tags(
        (name = "auth", description = "Authentication and sessions"),
        (name = "users", description = "Profiles and onboarding"),
        (name = "vehicles", description = "Customer vehicles"),
        (name = "breakdowns", description = "Breakdown reports and AI analysis"),
        (name = "mechanics", description = "Mechanic profile, availability, feed"),
        (name = "jobs", description = "Assistance job lifecycle"),
        (name = "offers", description = "Mechanic bidding"),
        (name = "payments", description = "Payment history and receipts"),
        (name = "ratings", description = "Ratings and reviews"),
        (name = "notifications", description = "In-app notifications"),
        (name = "agent", description = "Advisory AI endpoints"),
        (name = "admin", description = "Admin console"),
    )
)]
struct ApiDoc;

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
        )
        .route(
            "/payments",
            get(crate::handlers::payments_history::my_earnings),
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

    let payment_history_routes = Router::new().route("/", get(crate::handlers::payments_history::my_payments));

    let ws_routes =
        Router::new().route("/jobs/{job_id}", get(crate::ws::job_events));

    let admin_routes = Router::new()
        .route("/login", post(crate::handlers::admin::login))
        .route("/overview", get(crate::handlers::admin::overview))
        .route("/users", get(crate::handlers::admin::users))
        .route(
            "/users/{user_id}/status",
            post(crate::handlers::admin::set_user_status),
        )
        .route("/mechanics", get(crate::handlers::admin::mechanics))
        .route(
            "/mechanics/{mechanic_id}/verify",
            post(crate::handlers::admin::verify_mechanic),
        )
        .route("/jobs", get(crate::handlers::admin::jobs))
        .route(
            "/payments",
            get(crate::handlers::admin::payments),
        )
        .route(
            "/agent-runs",
            get(crate::handlers::admin::agent_runs),
        );

    let app: Router = Router::new()
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
        .nest("/api/v1/admin", admin_routes)
        .nest("/api/v1/payments", payment_history_routes)
        .nest("/api/v1/ws", ws_routes)
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .layer(cors)
        .with_state(state);

    app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
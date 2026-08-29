//! Composition root: assemble every service and run the server.

mod config;
mod state;

use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use toolfix_api::AppState;
use toolfix_auth::firebase::FirebaseIdentityKit;
use toolfix_auth::google::GoogleOAuthClient;
use toolfix_auth::AuthService;
use toolfix_jobs::{EventHub, JobService};
use toolfix_matching::MatchingService;
use toolfix_notifications::LoggingProvider;
use toolfix_payments::CashStubProvider;
use toolfix_persistence::Repositories;
use toolfix_pricing::PricingService;
use toolfix_runtime::{Runtime, RuntimeConfig};
use toolfix_storage::LocalFsBackend;

use crate::config::Config;
use crate::state::build_state;

#[tokio::main]
async fn main() {
    // Load .env from the working directory (process env wins).
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "server failed to start");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let config = Config::from_env()?;

    // 1. Database (connect + migrations).
    let db = toolfix_persistence::connect_with_driver(config.database_driver, &config.database_url)
        .await
        .map_err(|e| format!("database connect/migrate failed: {e}"))?;
    tracing::info!(driver = ?db.driver(), "database connected and migrations applied");

    let repos = Repositories::new(db);

    // 2. Storage (local FS first; S3-compatible backends plug in here).
    std::fs::create_dir_all(&config.storage_dir)
        .map_err(|e| format!("cannot create storage dir: {e}"))?;
    let storage: Arc<dyn toolfix_storage::StorageBackend> =
        Arc::new(LocalFsBackend::new(config.storage_dir.clone()));

    // 3. Notifications (logging provider first; FCM later).
    let notifications: Arc<dyn toolfix_notifications::NotificationProvider> =
        Arc::new(LoggingProvider);

    // 4. Matching + pricing.
    let matching = MatchingService::new(
        repos.mechanics.clone(),
        repos.notifications.clone(),
        notifications.clone(),
        config.matching.clone(),
    );
    let pricing = PricingService::new(repos.pricing.clone());

    // 5. Agentic layer (optional at boot; advisory only).
    let agent = match toolfix_agent::workflow::build_provider(
        &config.ai_provider,
        config.gemini_api_key.as_deref(),
        config.gemini_model.as_deref(),
        config.vertex_project_id.as_deref(),
        config.vertex_location.as_deref(),
        config.vertex_model.as_deref(),
        config.service_account_json.as_deref(),
    ) {
        Ok(provider) => {
            let workflow = toolfix_agent::AgentWorkflow::new(provider);
            tracing::info!(
                provider = workflow.provider_name(),
                model = workflow.model(),
                "agentic layer configured"
            );
            Some(workflow)
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "AI provider not configured; running without agent analysis"
            );
            None
        }
    };

    // 6. Authentication (backend-only Firebase + Google OAuth).
    let google_client = config.google_oauth_client.as_ref().map(|cfg| {
        GoogleOAuthClient::from_parts(
            cfg.client_id.clone(),
            cfg.client_secret.clone(),
            cfg.redirect_uri.clone(),
        )
    });
    let firebase_kit = config
        .firebase_identity_kit
        .as_ref()
        .map(|(project, key)| FirebaseIdentityKit::new(project, key));
    let request_origin = config
        .google_oauth_client
        .as_ref()
        .map(|cfg| cfg.redirect_uri.clone())
        .unwrap_or_else(|| {
            format!("http://localhost:{}/api/v1/auth/google/callback", config.port)
        });
    let auth_service = AuthService::new(
        config.auth.clone(),
        repos.users.clone(),
        repos.auth_sessions.clone(),
        google_client,
        firebase_kit,
        request_origin,
    );

    // 7. Payments (stub/cash first; Razorpay later).
    let payments: Arc<dyn toolfix_payments::PaymentProvider> = Arc::new(CashStubProvider);

    // 8. Job service.
    let events = EventHub::new();
    let jobs = JobService::new(
        repos.clone(),
        matching,
        pricing,
        agent.clone(),
        storage.clone(),
        payments,
        events.clone(),
        config.offer_expiry_secs,
    );

    // 9. Background runtime.
    let shutdown = CancellationToken::new();
    let runtime = Runtime::new(RuntimeConfig::default());
    let runtime_tasks =
        runtime.spawn_all(repos.clone(), jobs.clone(), notifications, shutdown.clone());
    tracing::info!(tasks = runtime_tasks.len(), "background runtime started");

    // 10. API state + CORS + server.
    let state: AppState = build_state(auth_service, jobs, repos, agent, storage);
    let cors = build_cors(&config.frontend_origin);
    let router = toolfix_api::router(state, cors);

    let address = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|e| format!("cannot bind {address}: {e}"))?;
    tracing::info!(%address, "ToolFix API listening");

    let shutdown_for_server = shutdown.clone();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("shutdown signal received; draining");
            shutdown_for_server.cancel();
        })
        .await
        .map_err(|e| format!("server error: {e}"))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn build_cors(frontend_origin: &str) -> CorsLayer {
    use tower_http::cors::AllowOrigin;
    // Explicit origins only: wildcard origins are incompatible with
    // credentialed (cookie) requests.
    let mut origins: Vec<String> = ["http://localhost:3000", "http://127.0.0.1:3000"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    origins.push(frontend_origin.to_string());
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect::<Vec<_>>(),
        ))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
}

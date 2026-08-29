//! AppState assembly for the API layer.

use std::sync::Arc;

use toolfix_api::AppState;
use toolfix_auth::AuthService;
use toolfix_jobs::JobService;
use toolfix_persistence::Repositories;

pub fn build_state(
    auth: AuthService,
    jobs: JobService,
    repos: Repositories,
    agent: Option<toolfix_agent::AgentWorkflow>,
    storage: Arc<dyn toolfix_storage::StorageBackend>,
) -> AppState {
    AppState {
        auth: Arc::new(auth),
        jobs: Arc::new(jobs),
        repos,
        agent,
        storage,
    }
}

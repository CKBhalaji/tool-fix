//! Shared API state. The composition root constructs one of these.

use std::sync::Arc;

use toolfix_auth::AuthService;
use toolfix_jobs::JobService;
use toolfix_persistence::Repositories;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub jobs: Arc<JobService>,
    pub repos: Repositories,
    pub agent: Option<toolfix_agent::AgentWorkflow>,
    pub storage: Arc<dyn toolfix_storage::StorageBackend>,
}

impl toolfix_auth::AuthProvider for AppState {
    fn token_verifier(&self) -> &toolfix_auth::TokenVerifier {
        self.auth.token_verifier()
    }
}

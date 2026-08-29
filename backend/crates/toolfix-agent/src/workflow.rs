//! High-level agent facade used by the application services.

use std::sync::Arc;

use crate::diagnosis;
use crate::error::AgentError;
use crate::pricing;
use crate::provider::{AgentProvider, DiagnosisInput, PriceEstimateInput};
use crate::schemas::{DiagnosisOutput, PriceEstimateOutput};

#[derive(Clone)]
pub struct AgentWorkflow {
    provider: Arc<dyn AgentProvider>,
}

impl AgentWorkflow {
    pub fn new(provider: Arc<dyn AgentProvider>) -> Self {
        Self { provider }
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    pub fn model(&self) -> &str {
        self.provider.model()
    }

    pub async fn diagnose_breakdown(
        &self,
        input: &DiagnosisInput,
    ) -> Result<DiagnosisOutput, AgentError> {
        diagnosis::diagnose_breakdown(self.provider.as_ref(), input).await
    }

    pub async fn estimate_repair_cost(
        &self,
        input: &PriceEstimateInput,
    ) -> Result<PriceEstimateOutput, AgentError> {
        pricing::estimate_repair_cost(self.provider.as_ref(), input).await
    }
}

/// Selects the configured provider. `AI_PROVIDER` is `gemini_api` (default)
/// or `vertex_ai`; both implement the same `AgentProvider` trait.
pub fn build_provider(
    ai_provider: &str,
    gemini_api_key: Option<&str>,
    gemini_model: Option<&str>,
    vertex_project_id: Option<&str>,
    vertex_location: Option<&str>,
    vertex_model: Option<&str>,
    service_account_json: Option<&str>,
) -> Result<Arc<dyn AgentProvider>, AgentError> {
    match ai_provider {
        "vertex_ai" => {
            let account_json = service_account_json.ok_or_else(|| {
                AgentError::Config(
                    "AI_PROVIDER=vertex_ai requires GOOGLE_APPLICATION_CREDENTIALS".into(),
                )
            })?;
            let vertex = crate::vertex::VertexProvider::from_service_account_json(
                account_json,
                vertex_project_id,
                vertex_location.unwrap_or("us-central1"),
                vertex_model.unwrap_or("gemini-2.0-flash"),
            )?;
            Ok(Arc::new(vertex))
        }
        _ => {
            let key = gemini_api_key.ok_or_else(|| {
                AgentError::Config("AI_PROVIDER=gemini_api requires GEMINI_API_KEY".into())
            })?;
            Ok(Arc::new(crate::gemini_api::GeminiApiKeyProvider::new(
                key,
                gemini_model.unwrap_or("gemini-2.0-flash"),
            )))
        }
    }
}

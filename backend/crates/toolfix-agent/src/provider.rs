//! Provider abstraction: swappable AI backends behind one interface.

use async_trait::async_trait;
use toolfix_contracts::{Severity, VehicleKind};

use crate::error::AgentError;

/// What the diagnosis agent sees. Images arrive as base64 (already
/// uploaded by the customer through the storage layer).
#[derive(Debug, Clone, Default)]
pub struct DiagnosisInput {
    pub vehicle_kind: Option<VehicleKind>,
    /// Human-readable vehicle description, e.g. "Honda Activa 2019".
    pub vehicle_display: Option<String>,
    pub description: String,
    pub symptoms: Vec<String>,
    pub image_base64: Option<String>,
    pub image_mime: Option<String>,
}

/// Historical price stats the estimate conditions on.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct HistoricalSummary {
    pub sample_count: i64,
    pub avg_amount_minor: Option<i64>,
    pub min_amount_minor: Option<i64>,
    pub max_amount_minor: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct PriceEstimateInput {
    pub vehicle_kind: Option<VehicleKind>,
    pub repair_category: Option<toolfix_contracts::RepairCategory>,
    pub severity: Option<Severity>,
    pub description: String,
    pub historical: Option<HistoricalSummary>,
    pub travel_distance_km: Option<f64>,
}

/// Every AI backend (Gemini API key, Vertex AI, future providers, test
/// fakes) implements this. The rest of the application never knows which
/// provider is configured.
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &'static str;

    fn model(&self) -> &str;

    /// Runs a prompt (with an optional image attachment) and returns the
    /// model's JSON answer as a raw value. Schema validation happens above
    /// this trait — providers only transport.
    async fn generate_json(
        &self,
        prompt: &str,
        image: Option<(&str, &str)>, // (mime, base64)
    ) -> Result<serde_json::Value, AgentError>;
}

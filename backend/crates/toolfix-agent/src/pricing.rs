//! Pricing sub-workflow: build prompt -> call provider -> validate.

use crate::error::AgentError;
use crate::provider::{AgentProvider, PriceEstimateInput};
use crate::prompts;
use crate::schemas::PriceEstimateOutput;

pub async fn estimate_repair_cost(
    provider: &dyn AgentProvider,
    input: &PriceEstimateInput,
) -> Result<PriceEstimateOutput, AgentError> {
    let prompt = prompts::price_prompt(input);
    let started = std::time::Instant::now();
    let raw = provider.generate_json(&prompt, None).await?;
    let output = PriceEstimateOutput::from_value(&raw)?;
    tracing::info!(
        provider = provider.name(),
        model = provider.model(),
        latency_ms = started.elapsed().as_millis() as i64,
        category = %output.repair_category(),
        "price estimate complete"
    );
    Ok(output)
}

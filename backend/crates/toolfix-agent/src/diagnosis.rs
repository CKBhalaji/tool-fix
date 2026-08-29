//! Diagnosis sub-workflow: build prompt -> call provider -> validate.

use crate::error::AgentError;
use crate::provider::{AgentProvider, DiagnosisInput};
use crate::prompts;
use crate::schemas::DiagnosisOutput;

pub async fn diagnose_breakdown(
    provider: &dyn AgentProvider,
    input: &DiagnosisInput,
) -> Result<DiagnosisOutput, AgentError> {
    let prompt = prompts::diagnosis_prompt(input);
    let image = input
        .image_base64
        .as_deref()
        .zip(input.image_mime.as_deref());

    let started = std::time::Instant::now();
    let raw = provider.generate_json(&prompt, image).await?;
    let output = DiagnosisOutput::from_value(&raw)?;
    tracing::info!(
        provider = provider.name(),
        model = provider.model(),
        latency_ms = started.elapsed().as_millis() as i64,
        category = %output.repair_category(),
        "diagnosis complete"
    );
    Ok(output)
}

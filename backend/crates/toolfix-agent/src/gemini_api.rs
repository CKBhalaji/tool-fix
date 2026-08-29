//! Gemini API-key provider (Google AI Studio). REST, no SDK.

use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;

use crate::error::AgentError;
use crate::provider::AgentProvider;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GeminiApiKeyProvider {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiApiKeyProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    async fn generate_content(
        &self,
        prompt: &str,
        image: Option<(&str, &str)>,
    ) -> Result<serde_json::Value, AgentError> {
        let mut parts = vec![json!({ "text": prompt })];
        if let Some((mime, data)) = image {
            parts.push(json!({
                "inline_data": { "mime_type": mime, "data": data }
            }));
        }

        let body = json!({
            "contents": [{ "role": "user", "parts": parts }],
            "generationConfig": {
                "temperature": 0.2,
                "response_mime_type": "application/json"
            }
        });

        let url = format!("{}/{}:generateContent?key={}", BASE_URL, self.model, self.api_key);
        let response: serde_json::Value = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let text = response
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::Provider(format!(
                    "unexpected gemini response shape: {}",
                    truncate_debug(&response)
                ))
            })?;
        parse_model_json(text)
    }
}

/// Strips optional markdown fences models sometimes add despite the
/// response_mime_type contract.
pub fn parse_model_json(text: &str) -> Result<serde_json::Value, AgentError> {
    let trimmed = text.trim();
    let without_fences = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_start();
    let without_fences = without_fences
        .strip_suffix("```")
        .unwrap_or(without_fences)
        .trim();
    serde_json::from_str(without_fences)
        .map_err(|e| AgentError::Parse(format!("model text is not JSON: {e}")))
}

fn truncate_debug(value: &serde_json::Value) -> String {
    let rendered = value.to_string();
    if rendered.len() > 300 {
        format!("{}…", &rendered[..300])
    } else {
        rendered
    }
}

#[async_trait]
impl AgentProvider for GeminiApiKeyProvider {
    fn name(&self) -> &'static str {
        "gemini_api_key"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate_json(
        &self,
        prompt: &str,
        image: Option<(&str, &str)>,
    ) -> Result<serde_json::Value, AgentError> {
        self.generate_content(prompt, image).await
    }
}

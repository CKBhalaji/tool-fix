//! NVIDIA NIM provider (free-tier API key from build.nvidia.com).
//!
//! OpenAI-compatible chat-completions endpoint; vision models accept
//! base64 data-URL images, so the same diagnosis flow works here as on
//! Gemini/Vertex.

use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;

use crate::error::AgentError;
use crate::provider::AgentProvider;

const BASE_URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";

pub struct NvidiaNimProvider {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl NvidiaNimProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(90))
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
        // Content is a plain string for text-only prompts and a multimodal
        // parts array when an image is attached.
        let content = match image {
            None => json!(prompt),
            Some((mime, data)) => json!([
                { "type": "text", "text": prompt },
                { "type": "image_url", "image_url": { "url": format!("data:{mime};base64,{data}") } }
            ]),
        };

        let body = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": content }],
            "temperature": 0.2,
            "response_format": { "type": "json_object" },
            "max_tokens": 1024
        });

        let response: serde_json::Value = self
            .http
            .post(BASE_URL)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let text = response
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::Provider(format!(
                    "unexpected nvidia response shape: {}",
                    response.to_string().chars().take(300).collect::<String>()
                ))
            })?;
        crate::gemini_api::parse_model_json(text)
    }
}

#[async_trait]
impl AgentProvider for NvidiaNimProvider {
    fn name(&self) -> &'static str {
        "nvidia_nim"
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

//! Vertex AI provider: service-account authentication and the Vertex
//! generateContent endpoint (mirrors the Gemini body shape).

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use crate::error::AgentError;
use crate::provider::AgentProvider;

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccount {
    pub client_email: String,
    pub private_key: String,
    #[serde(default)]
    pub private_key_id: Option<String>,
    #[serde(default = "default_token_uri")]
    pub token_uri: String,
    #[serde(default)]
    pub project_id: Option<String>,
}

fn default_token_uri() -> String {
    "https://oauth2.googleapis.com/token".to_string()
}

pub struct VertexProvider {
    http: reqwest::Client,
    service_account: ServiceAccount,
    project_id: String,
    location: String,
    model: String,
    cached_token: Mutex<Option<(String, i64)>>, // (token, expires_at epoch)
}

const OAUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

impl VertexProvider {
    pub fn from_service_account_json(
        json: &str,
        project_id: Option<&str>,
        location: &str,
        model: &str,
    ) -> Result<Self, AgentError> {
        let account: ServiceAccount = serde_json::from_str(json)
            .map_err(|e| AgentError::Config(format!("service account parse failed: {e}")))?;
        let project_id = project_id
            .map(str::to_string)
            .or(account.project_id.clone())
            .ok_or_else(|| AgentError::Config("vertex project id missing".into()))?;
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
            service_account: account,
            project_id,
            location: location.to_string(),
            model: model.to_string(),
            cached_token: Mutex::new(None),
        })
    }

    fn endpoint(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, self.model
        )
    }

    /// JWT-bearer token exchange against the Google OAuth endpoint, with an
    /// in-process cache honoring the returned expiry.
    async fn access_token(&self) -> Result<String, AgentError> {
        let now = chrono::Utc::now().timestamp();
        if let Some((token, expires_at)) = self
            .cached_token
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            && now < expires_at - 60
        {
            return Ok(token);
        }

        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        if let Some(kid) = &self.service_account.private_key_id {
            header.kid = Some(kid.clone());
        }
        let claims = json!({
            "iss": self.service_account.client_email,
            "scope": OAUTH_SCOPE,
            "aud": self.service_account.token_uri,
            "iat": now,
            "exp": now + 3600,
        });
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(
            self.service_account.private_key.as_bytes(),
        )
        .map_err(|e| AgentError::Config(format!("bad service account private key: {e}")))?;
        let assertion = jsonwebtoken::encode(&header, &claims, &key)?;

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            #[serde(default)]
            expires_in: Option<i64>,
        }

        let response: TokenResponse = self
            .http
            .post(&self.service_account.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let expires_at = now + response.expires_in.unwrap_or(3600);
        if let Ok(mut guard) = self.cached_token.lock() {
            *guard = Some((response.access_token.clone(), expires_at));
        }
        Ok(response.access_token)
    }
}

#[async_trait]
impl AgentProvider for VertexProvider {
    fn name(&self) -> &'static str {
        "vertex_ai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate_json(
        &self,
        prompt: &str,
        image: Option<(&str, &str)>,
    ) -> Result<serde_json::Value, AgentError> {
        let token = self.access_token().await?;

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

        let response: serde_json::Value = self
            .http
            .post(self.endpoint())
            .bearer_auth(token)
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
                    "unexpected vertex response shape: {}",
                    response.to_string().chars().take(300).collect::<String>()
                ))
            })?;
        crate::gemini_api::parse_model_json(text)
    }
}

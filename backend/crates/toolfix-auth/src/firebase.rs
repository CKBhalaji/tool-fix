//! Firebase Identity Toolkit access — strictly server-side.
//!
//! The backend provisions/links the Firebase user with the verified Google
//! identity (`accounts:signInWithIdp`). The Firebase Web API key used here
//! is a backend environment value and never reaches the frontend.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::AuthError;

const SIGN_IN_WITH_IDP_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts:signInWithIdp";
const FIREBASE_JWKS_URI: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

/// Identity of the provisioned Firebase user.
#[derive(Debug, Clone, Serialize)]
pub struct FirebaseIdentity {
    pub local_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    /// Firebase ID token minted by the provisioning exchange. Kept
    /// server-side only.
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct SignInWithIdpResponse {
    #[serde(rename = "localId")]
    local_id: String,
    #[serde(default)]
    idToken: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    displayName: Option<String>,
    #[serde(default)]
    photoUrl: Option<String>,
}

#[derive(Clone)]
pub struct FirebaseIdentityKit {
    http: reqwest::Client,
    project_id: String,
    web_api_key: String,
    jwks: std::sync::Arc<std::sync::RwLock<Option<jsonwebtoken::jwk::JwkSet>>>,
}

impl FirebaseIdentityKit {
    pub fn new(project_id: &str, web_api_key: &str) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            project_id: project_id.to_string(),
            web_api_key: web_api_key.to_string(),
            jwks: std::sync::Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Exchanges a verified Google ID token for a Firebase session. This is
    /// a direct server-to-server call to Google's Identity Toolkit.
    pub async fn sign_in_with_google(
        &self,
        google_id_token: &str,
        request_uri: &str,
    ) -> Result<FirebaseIdentity, AuthError> {
        let post_body = format!("id_token={google_id_token}&providerId=google.com");
        let body = serde_json::json!({
            "postBody": post_body,
            "requestUri": request_uri,
            "returnIdpCredential": true,
            "returnSecureToken": true
        });

        let response: SignInWithIdpResponse = self
            .http
            .post(format!("{SIGN_IN_WITH_IDP_URL}?key={}", self.web_api_key))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(|e| AuthError::Firebase(e.to_string()))?;

        Ok(FirebaseIdentity {
            local_id: response.local_id,
            email: response.email,
            display_name: response.displayName,
            photo_url: response.photoUrl,
            id_token: response.idToken,
        })
    }

    /// Verifies a Firebase ID token against Google's public JWKS with the
    /// project-id audience. (Provisioned tokens come straight from Google
    /// over TLS, but verification is available for server-accepted tokens.)
    pub async fn verify_id_token(
        &self,
        token: &str,
    ) -> Result<serde_json::Value, AuthError> {
        #[derive(Deserialize)]
        struct FirebaseClaims {
            #[allow(dead_code)]
            iss: String,
            #[allow(dead_code)]
            aud: String,
            #[serde(flatten)]
            rest: serde_json::Value,
        }

        let jwks = self.current_jwks().await?;
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("firebase token missing kid".into()))?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| AuthError::InvalidToken("no firebase JWKS key matches".into()))?;
        let key = jsonwebtoken::DecodingKey::from_jwk(jwk)?;
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[format!("https://securetoken.google.com/{}", self.project_id)]);
        validation.set_audience(&[&self.project_id]);
        let data = jsonwebtoken::decode::<FirebaseClaims>(token, &key, &validation)?;
        Ok(data.claims.rest)
    }

    async fn current_jwks(&self) -> Result<jsonwebtoken::jwk::JwkSet, AuthError> {
        if let Some(cached) = self.jwks.read().ok().and_then(|g| g.clone()) {
            return Ok(cached);
        }
        let fresh: jsonwebtoken::jwk::JwkSet = self
            .http
            .get(FIREBASE_JWKS_URI)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if let Ok(mut guard) = self.jwks.write() {
            *guard = Some(fresh.clone());
        }
        Ok(fresh)
    }
}

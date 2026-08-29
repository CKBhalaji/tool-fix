//! Server-side Google OAuth 2.0 authorization-code flow.
//!
//! The browser is only ever redirected to Google's consent page; the
//! authorization code is exchanged server-to-server, so no token of any
//! kind is handled by frontend JavaScript.

use std::time::Duration;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::config::GoogleOAuthConfig;
use crate::error::AuthError;

const AUTH_URI: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const JWKS_URI: &str = "https://www.googleapis.com/oauth2/v3/certs";

#[derive(Debug, Clone, Deserialize)]
struct GoogleClientSecretFile {
    #[serde(default)]
    web: Option<GoogleClientSecretInner>,
    #[serde(default)]
    installed: Option<GoogleClientSecretInner>,
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleClientSecretInner {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
    #[allow(dead_code)]
    access_token: Option<String>,
}

/// Verified Google identity of the signed-in human. The raw Google ID
/// token is retained (crate-internally) for the server-side Firebase
/// provisioning exchange; it is never logged, serialized, or sent to the
/// frontend.
#[derive(Clone, serde::Serialize)]
pub struct GoogleIdentity {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
    #[serde(skip)]
    pub(crate) google_id_token: String,
}

impl std::fmt::Debug for GoogleIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleIdentity")
            .field("sub", &self.sub)
            .field("email", &self.email)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Deserialize)]
struct GoogleIdClaims {
    #[allow(dead_code)]
    iss: String,
    sub: String,
    #[serde(default)]
    aud: serde_json::Value,
    email: Option<String>,
    #[serde(default, rename = "email_verified")]
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Clone)]
pub struct GoogleOAuthClient {
    http: reqwest::Client,
    config: GoogleOAuthConfig,
    jwks: std::sync::Arc<std::sync::RwLock<Option<jsonwebtoken::jwk::JwkSet>>>,
}

impl GoogleOAuthClient {
    /// Builds the client from parsed id/secret/redirect parts.
    pub fn from_parts(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            config: GoogleOAuthConfig {
                client_id: client_id.into(),
                client_secret: client_secret.into(),
                redirect_uri: redirect_uri.into(),
            },
            jwks: std::sync::Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Builds the client from the downloaded Google Cloud "client secret"
    /// JSON file (web or installed/desktop variant).
    pub fn from_secret_json(json: &str, redirect_uri: &str) -> Result<Self, AuthError> {
        let file: GoogleClientSecretFile =
            serde_json::from_str(json).map_err(|e| AuthError::Storage(e.to_string()))?;
        let inner = file
            .web
            .or(file.installed)
            .ok_or_else(|| AuthError::Storage("client secret file has no web/installed entry".into()))?;
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()?,
            config: GoogleOAuthConfig {
                client_id: inner.client_id,
                client_secret: inner.client_secret,
                redirect_uri: redirect_uri.to_string(),
            },
            jwks: std::sync::Arc::new(std::sync::RwLock::new(None)),
        })
    }

    pub fn client_id(&self) -> &str {
        &self.config.client_id
    }

    pub fn redirect_uri(&self) -> &str {
        &self.config.redirect_uri
    }

    /// Consent URL with S256 PKCE.
    pub fn consent_url(&self, state: &str, code_challenge: &str) -> String {
        let mut url = reqwest::Url::parse(AUTH_URI).expect("static auth uri");
        let mut query = url.query_pairs_mut();
        query.append_pair("client_id", &self.config.client_id);
        query.append_pair("redirect_uri", &self.config.redirect_uri);
        query.append_pair("response_type", "code");
        query.append_pair("scope", "openid email profile");
        query.append_pair("state", state);
        query.append_pair("code_challenge", code_challenge);
        query.append_pair("code_challenge_method", "S256");
        query.append_pair("access_type", "online");
        query.append_pair("prompt", "select_account");
        drop(query);
        url.to_string()
    }

    /// Exchanges the authorization code for tokens and verifies the ID
    /// token against Google's JWKS.
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<GoogleIdentity, AuthError> {
        let params = [
            ("code", code),
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ];
        let response: TokenResponse = self
            .http
            .post(TOKEN_URI)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let id_token = response
            .id_token
            .ok_or_else(|| AuthError::OAuth("token response missing id_token".into()))?;
        self.verify_id_token(&id_token).await
    }

    pub async fn verify_id_token(&self, id_token: &str) -> Result<GoogleIdentity, AuthError> {
        let jwks = self.current_jwks().await?;
        let header = jsonwebtoken::decode_header(id_token)?;
        let kid = header.kid.ok_or_else(|| {
            AuthError::InvalidToken("google id_token header missing kid".into())
        })?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| AuthError::InvalidToken("no JWKS key matches token kid".into()))?;
        let decoding_key = DecodingKey::from_jwk(jwk)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
        validation.validate_aud = false; // aud is checked explicitly below
        let data = decode::<GoogleIdClaims>(id_token, &decoding_key, &validation)?;

        let claims = data.claims;
        if !claims.aud_as_iter().any(|aud| aud == self.config.client_id) {
            return Err(AuthError::InvalidToken("id_token audience mismatch".into()));
        }
        Ok(GoogleIdentity {
            sub: claims.sub,
            email: claims.email,
            email_verified: claims.email_verified.unwrap_or(false),
            name: claims.name,
            picture: claims.picture,
            google_id_token: id_token.to_string(),
        })
    }

    async fn current_jwks(&self) -> Result<jsonwebtoken::jwk::JwkSet, AuthError> {
        if let Some(cached) = self.jwks.read().ok().and_then(|g| g.clone()) {
            return Ok(cached);
        }
        let fresh: jsonwebtoken::jwk::JwkSet = self
            .http
            .get(JWKS_URI)
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

impl GoogleIdClaims {
    fn aud_as_iter(&self) -> impl Iterator<Item = &str> {
        match &self.aud {
            serde_json::Value::String(s) => vec![s.as_str()].into_iter(),
            serde_json::Value::Array(list) => list
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .into_iter(),
            _ => Vec::new().into_iter(),
        }
    }
}

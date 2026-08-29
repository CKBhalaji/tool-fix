//! Typed token claims and the access-token verifier.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AuthError;
use toolfix_contracts::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    /// ToolFix user id.
    pub sub: String,
    pub role: UserRole,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub aud: String,
}

/// Issues and verifies ToolFix access tokens (HS256, short-lived).
/// Deliberately distinct from Firebase/Google identity tokens.
#[derive(Clone)]
pub struct TokenVerifier {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    ttl_secs: i64,
}

impl TokenVerifier {
    pub fn new(secret: &str, issuer: &str, audience: &str, ttl_secs: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            ttl_secs,
        }
    }

    pub fn ttl_secs(&self) -> i64 {
        self.ttl_secs
    }

    pub fn issue(&self, user_id: uuid::Uuid, role: UserRole) -> Result<(AccessClaims, String), AuthError> {
        let now = chrono::Utc::now().timestamp();
        let claims = AccessClaims {
            sub: user_id.to_string(),
            role,
            iat: now,
            exp: now + self.ttl_secs,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)?;
        Ok((claims, token))
    }

    pub fn verify(&self, token: &str) -> Result<AccessClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.leeway = 30;
        let data = decode::<AccessClaims>(token, &self.decoding_key, &validation)?;
        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn verifier() -> TokenVerifier {
        TokenVerifier::new("test-secret", "toolfix", "toolfix-api", 900)
    }

    #[tokio::test]
    async fn issue_and_verify_roundtrip() {
        let v = verifier();
        let user_id = Uuid::now_v7();
        let (_, token) = v.issue(user_id, UserRole::Customer).unwrap();
        let claims = v.verify(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, UserRole::Customer);
    }

    #[tokio::test]
    async fn rejects_tampered_tokens() {
        let v = verifier();
        let (_, token) = v.issue(Uuid::now_v7(), UserRole::Admin).unwrap();
        let mut tampered = token.to_string();
        tampered.push('x');
        assert!(v.verify(&tampered).is_err());
    }
}

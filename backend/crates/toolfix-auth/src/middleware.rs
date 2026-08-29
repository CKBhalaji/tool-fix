//! Axum authentication plumbing: an extractor-backed identity context.
//!
//! Handlers take `AuthUser` as an argument; identity is always derived from
//! the access-token cookie, never from request bodies or query parameters.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use toolfix_contracts::UserRole;

use crate::cookies::{read_cookie, ACCESS_COOKIE};
use crate::error::AuthError;
use crate::claims::TokenVerifier;

/// Implemented by the composition root's AppState so the extractor can
/// reach the verifier without depending on concrete app types.
pub trait AuthProvider {
    fn token_verifier(&self) -> &TokenVerifier;
}

/// Authenticated identity resolved from the HttpOnly access-token cookie.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub role: UserRole,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + AuthProvider,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = read_cookie(&parts.headers, ACCESS_COOKIE)
            .ok_or(AuthError::Unauthenticated)?;
        let claims = state.token_verifier().verify(&token)?;
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken("malformed subject claim".into()))?;
        Ok(AuthUser {
            user_id,
            role: claims.role,
        })
    }
}

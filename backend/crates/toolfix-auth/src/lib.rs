//! ToolFix authentication.
//!
//! Firebase is a BACKEND-ONLY dependency: the browser never sees the
//! Firebase SDK, configuration, or tokens. The server runs the Google
//! OAuth authorization-code flow, provisions the Firebase identity
//! server-side, and issues its own access/refresh tokens which live
//! exclusively in HttpOnly cookies.

pub mod authorization;
pub mod claims;
pub mod config;
pub mod cookies;
pub mod error;
pub mod firebase;
pub mod google;
pub mod middleware;
pub mod session;
pub mod tokens;

pub use claims::{AccessClaims, TokenVerifier};
pub use config::{AuthConfig, GoogleOAuthConfig};
pub use cookies::{
    ACCESS_COOKIE, OAUTH_STATE_COOKIE, OAUTH_VERIFIER_COOKIE, REFRESH_COOKIE,
};
pub use error::AuthError;
pub use middleware::{AuthUser, AuthProvider};
pub use session::AuthService;
pub use tokens::hash_refresh_token;

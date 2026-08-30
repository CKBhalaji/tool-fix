//! Auth configuration (values come from the environment via the server).

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    /// Callback URL registered in Google Cloud Console, e.g.
    /// `http://localhost:8080/api/v1/auth/google/callback`.
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    // Static admin console credentials (backend-verified; server-derived role).
    pub admin_email: String,
    pub admin_password: String,
    pub access_secret: String,
    pub access_ttl_secs: i64,
    pub refresh_ttl_secs: i64,
    pub cookie_secure: bool,
    pub cookie_same_site_lax: bool,
    pub frontend_origin: String,
    pub issuer: String,
    pub audience: String,
    pub google_oauth: Option<GoogleOAuthConfig>,
    pub firebase_project_id: Option<String>,
    /// Firebase Web API key (backend-only) used for the server-side
    /// `accounts:signInWithIdp` provisioning call. Optional: when absent,
    /// Firebase provisioning is skipped (Google identity still verified).
    pub firebase_web_api_key: Option<String>,
}

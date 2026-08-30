//! Environment configuration. Process environment values are authoritative;
//! `dotenvy` supplies the rest from `.env`.

use toolfix_auth::{AuthConfig, GoogleOAuthConfig};
use toolfix_matching::MatchingConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_driver: toolfix_persistence::DbDriver,
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub frontend_origin: String,
    pub storage_dir: String,
    pub offer_expiry_secs: i64,
    pub matching: MatchingConfig,
    pub auth: AuthConfig,
    pub google_oauth_client: Option<GoogleOAuthConfig>,
    pub firebase_identity_kit: Option<(String, String)>, // (project_id, web_api_key)
    pub ai_provider: String,
    pub gemini_api_key: Option<String>,
    pub gemini_model: Option<String>,
    pub nvidia_api_key: Option<String>,
    pub nvidia_model: Option<String>,
    pub vertex_project_id: Option<String>,
    pub vertex_location: Option<String>,
    pub vertex_model: Option<String>,
    pub service_account_json: Option<String>,
}

fn env_str(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn env_required(key: &str) -> Result<String, String> {
    env_str(key).ok_or_else(|| format!("missing required environment variable: {key}"))
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    env_str(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env_required("DATABASE_URL")?;
        // Driver comes from DATABASE_DRIVER when set; otherwise it is
        // inferred from the URL scheme. The two must agree when both given.
        let database_driver = match env_str("DATABASE_DRIVER") {
            Some(value) => {
                let driver = match value.to_ascii_lowercase().as_str() {
                    "sqlite" => toolfix_persistence::DbDriver::Sqlite,
                    "postgres" | "postgresql" => toolfix_persistence::DbDriver::Postgres,
                    other => {
                        return Err(format!(
                            "unknown DATABASE_DRIVER '{other}' (use sqlite|postgres)"
                        ))
                    }
                };
                let inferred = toolfix_persistence::DbDriver::infer(&database_url)
                    .map_err(|e| e.to_string())?;
                if driver != inferred {
                    return Err(format!(
                        "DATABASE_DRIVER={value} does not match DATABASE_URL scheme ({database_url})"
                    ));
                }
                driver
            }
            None => toolfix_persistence::DbDriver::infer(&database_url)
                .map_err(|e| e.to_string())?,
        };
        let host = env_str("TOOLFIX_HOST").unwrap_or_else(|| "0.0.0.0".into());
        let port: u16 = env_parse("TOOLFIX_PORT", 8080);
        let frontend_origin = env_str("FRONTEND_ORIGIN").unwrap_or_else(|| "http://localhost:3000".into());

        let access_secret = env_required("JWT_ACCESS_SECRET")?;
        if access_secret.len() < 32 {
            return Err("JWT_ACCESS_SECRET must be at least 32 characters".into());
        }
        let cookie_secure = env_parse("COOKIE_SECURE", false);
        let cookie_same_site_lax = env_parse("COOKIE_SAME_SITE", true);
        let access_ttl_secs = env_parse("JWT_ACCESS_TTL_SECS", 900_i64);
        let refresh_ttl_secs = env_parse("JWT_REFRESH_TTL_SECS", 30 * 24 * 3600_i64);
        let offer_expiry_secs = env_parse("OFFER_EXPIRY_SECONDS", 900_i64);

        // Google OAuth client from the downloaded client-secret file.
        let google_oauth_client = match env_str("GOOGLE_OAUTH_CLIENT_PATH") {
            Some(path) => {
                let json = std::fs::read_to_string(&path)
                    .map_err(|e| format!("cannot read GOOGLE_OAUTH_CLIENT_PATH ({path}): {e}"))?;
                let redirect_uri = env_str("GOOGLE_OAUTH_REDIRECT_URI").unwrap_or_else(|| {
                    format!("http://localhost:{port}/api/v1/auth/google/callback")
                });
                // Parse just enough to build the config (validation of
                // shape happens in the auth client).
                let parsed = parse_google_client(&json)?;
                Some(GoogleOAuthConfig {
                    client_id: parsed.0,
                    client_secret: parsed.1,
                    redirect_uri,
                })
            }
            None => {
                tracing::warn!("GOOGLE_OAUTH_CLIENT_PATH not set; Google login disabled");
                None
            }
        };

        let firebase_project_id = env_str("FIREBASE_PROJECT_ID");
        let firebase_web_api_key = env_str("FIREBASE_WEB_API_KEY");
        let firebase_identity_kit = match (&firebase_project_id, &firebase_web_api_key) {
            (Some(project), Some(key)) => Some((project.clone(), key.clone())),
            _ => {
                tracing::warn!(
                    "FIREBASE_PROJECT_ID/FIREBASE_WEB_API_KEY not fully set; firebase provisioning disabled"
                );
                None
            }
        };

        // AI provider setup (gemini_api | nvidia | vertex_ai).
        let ai_provider = env_str("AI_PROVIDER").unwrap_or_else(|| "gemini_api".into());
        let gemini_api_key = env_str("GEMINI_API_KEY");
        let gemini_model = env_str("GEMINI_MODEL");
        let nvidia_api_key = env_str("NVIDIA_API_KEY");
        let nvidia_model = env_str("NVIDIA_MODEL");
        let vertex_project_id = env_str("VERTEX_PROJECT_ID");
        let vertex_location = env_str("VERTEX_LOCATION");
        let vertex_model = env_str("VERTEX_MODEL");
        // The Vertex service account may have its own file; when unset, the
        // shared GOOGLE_APPLICATION_CREDENTIALS is used.
        let service_account_json = match env_str("VERTEX_SERVICE_ACCOUNT_PATH")
            .or_else(|| env_str("GOOGLE_APPLICATION_CREDENTIALS"))
        {
            Some(path) => match std::fs::read_to_string(&path) {
                Ok(json) => Some(json),
                Err(e) => {
                    return Err(format!(
                        "cannot read vertex service account ({path}): {e}"
                    ))
                }
            },
            None => None,
        };

        // Static admin console credentials (change them for production!).
        let admin_email = env_str("ADMIN_EMAIL").unwrap_or_else(|| "admin@toolfix.com".into());
        let admin_password =
            env_str("ADMIN_PASSWORD").unwrap_or_else(|| "toolfix@2026".into());

        let matching = MatchingConfig {
            radius_stages_m: vec![
                env_parse("MATCHING_RADIUS_STAGE1_M", 3_000.0),
                env_parse("MATCHING_RADIUS_STAGE2_M", 5_000.0),
                env_parse("MATCHING_RADIUS_STAGE3_M", 10_000.0),
            ],
            max_notified: env_parse("MAX_MECHANICS_NOTIFIED", 10),
            feed_lookback_secs: env_parse("FEED_LOOKBACK_SECONDS", 6 * 3600),
            avg_speed_kmh: env_parse("MATCHING_AVG_SPEED_KMH", 25.0),
            max_active_jobs_per_mechanic: env_parse("MAX_ACTIVE_JOBS_PER_MECHANIC", 3),
        };

        let auth = AuthConfig {
            admin_email,
            admin_password,
            access_secret,
            access_ttl_secs,
            refresh_ttl_secs,
            cookie_secure,
            cookie_same_site_lax,
            frontend_origin: frontend_origin.clone(),
            issuer: env_str("JWT_ISSUER").unwrap_or_else(|| "toolfix".into()),
            audience: env_str("JWT_AUDIENCE").unwrap_or_else(|| "toolfix-api".into()),
            google_oauth: google_oauth_client.clone(),
            firebase_project_id,
            firebase_web_api_key,
        };

        Ok(Self {
            database_driver,
            database_url,
            host,
            port,
            frontend_origin,
            storage_dir: env_str("STORAGE_DIR").unwrap_or_else(|| "./storage".into()),
            offer_expiry_secs,
            matching,
            auth,
            google_oauth_client,
            firebase_identity_kit,
            ai_provider,
            gemini_api_key,
            gemini_model,
            nvidia_api_key,
            nvidia_model,
            vertex_project_id,
            vertex_location,
            vertex_model,
            service_account_json,
        })
    }
}

/// Extracts (client_id, client_secret) from the Google client-secret JSON.
fn parse_google_client(json: &str) -> Result<(String, String), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("invalid google client secret file: {e}"))?;
    for section in ["web", "installed"] {
        let Some(inner) = value.get(section) else {
            continue;
        };
        if let (Some(id), Some(secret)) = (inner.get("client_id"), inner.get("client_secret")) {
            return Ok((
                id.as_str().unwrap_or_default().to_string(),
                secret.as_str().unwrap_or_default().to_string(),
            ));
        }
    }
    Err("google client secret file has no web/installed client".into())
}

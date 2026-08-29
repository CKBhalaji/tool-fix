//! Cookie building/parsing. Tokens live ONLY here — never in
//! localStorage, sessionStorage, URLs, or logs.

use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use cookie::{Cookie, SameSite};
use cookie::time::Duration;

use crate::config::AuthConfig;

pub const ACCESS_COOKIE: &str = "access_token";
pub const REFRESH_COOKIE: &str = "refresh_token";
pub const OAUTH_STATE_COOKIE: &str = "oauth_state";
pub const OAUTH_VERIFIER_COOKIE: &str = "oauth_verifier";

fn same_site(config: &AuthConfig) -> SameSite {
    if config.cookie_same_site_lax {
        SameSite::Lax
    } else {
        SameSite::None
    }
}

fn base_cookie(name: &str, value: &str, config: &AuthConfig, path: &str) -> Cookie<'static> {
    Cookie::build((name.to_string(), value.to_string()))
        .path(path.to_string())
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(same_site(config))
        .build()
}

/// access_token (path /) + refresh_token (path /api/v1/auth).
pub fn auth_cookies(config: &AuthConfig, access_token: &str, refresh_token: &str) -> [Cookie<'static>; 2] {
    let mut access = base_cookie(ACCESS_COOKIE, access_token, config, "/");
    access.set_max_age(Duration::seconds(config.access_ttl_secs));
    let mut refresh = base_cookie(REFRESH_COOKIE, refresh_token, config, "/api/v1/auth");
    refresh.set_max_age(Duration::seconds(config.refresh_ttl_secs));
    [access, refresh]
}

/// Short-lived OAuth handshake cookies (state + PKCE verifier).
pub fn oauth_handshake_cookies(config: &AuthConfig, state: &str, verifier: &str) -> [Cookie<'static>; 2] {
    let mut state_cookie = base_cookie(OAUTH_STATE_COOKIE, state, config, "/api/v1/auth");
    state_cookie.set_max_age(Duration::seconds(600));
    let mut verifier_cookie = base_cookie(OAUTH_VERIFIER_COOKIE, verifier, config, "/api/v1/auth");
    verifier_cookie.set_max_age(Duration::seconds(600));
    [state_cookie, verifier_cookie]
}

/// Clears the auth cookies using the same attributes they were set with.
pub fn clearing_cookies(config: &AuthConfig) -> [Cookie<'static>; 2] {
    let mut access = base_cookie(ACCESS_COOKIE, "", config, "/");
    access.make_removal();
    let mut refresh = base_cookie(REFRESH_COOKIE, "", config, "/api/v1/auth");
    refresh.make_removal();
    [access, refresh]
}

pub fn oauth_handshake_clearing_cookies(config: &AuthConfig) -> [Cookie<'static>; 2] {
    let mut state_cookie = base_cookie(OAUTH_STATE_COOKIE, "", config, "/api/v1/auth");
    state_cookie.make_removal();
    let mut verifier_cookie = base_cookie(OAUTH_VERIFIER_COOKIE, "", config, "/api/v1/auth");
    verifier_cookie.make_removal();
    [state_cookie, verifier_cookie]
}

/// Reads a named cookie out of the Cookie header (first match wins).
pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in raw.split(';') {
        let pair = pair.trim();
        let (k, v) = pair.split_once('=')?;
        if k.trim() == name {
            return Some(v.trim().to_string());
        }
    }
    None
}

/// Appends cookies to a HeaderMap as Set-Cookie headers.
pub fn append_cookies(headers: &mut HeaderMap, cookies: &[Cookie<'static>]) {
    for cookie in cookies {
        if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
            headers.append(SET_COOKIE, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::{COOKIE, SET_COOKIE};

    fn config() -> AuthConfig {
        AuthConfig {
            access_secret: "s".into(),
            access_ttl_secs: 900,
            refresh_ttl_secs: 2_592_000,
            cookie_secure: false,
            cookie_same_site_lax: true,
            frontend_origin: "http://localhost:3000".into(),
            issuer: "toolfix".into(),
            audience: "toolfix-api".into(),
            google_oauth: None,
            firebase_project_id: None,
            firebase_web_api_key: None,
        }
    }

    #[test]
    fn auth_cookies_have_required_attributes() {
        let cfg = config();
        let cookies = auth_cookies(&cfg, "acc", "ref");
        for cookie in &cookies {
            assert!(cookie.http_only().unwrap_or(false));
            assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        }
        assert_eq!(cookies[0].path().unwrap_or_default(), "/");
        assert_eq!(cookies[1].path().unwrap_or_default(), "/api/v1/auth");
    }

    #[test]
    fn cookies_roundtrip_through_headers() {
        let cfg = config();
        let mut headers = HeaderMap::new();
        append_cookies(&mut headers, &auth_cookies(&cfg, "acc", "ref"));
        let set = headers.get(SET_COOKIE).unwrap().to_str().unwrap();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(set.split(';').next().unwrap()).unwrap(),
        );
        assert_eq!(read_cookie(&headers, ACCESS_COOKIE).as_deref(), Some("acc"));
    }

    #[test]
    fn clearing_cookies_remove_values() {
        let cfg = config();
        let cookies = clearing_cookies(&cfg);
        for cookie in &cookies {
            assert!(cookie.max_age().map(|d| d.is_zero()).unwrap_or(false));
            assert!(cookie.value().is_empty());
        }
    }
}

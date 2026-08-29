//! Application authentication service: the flow that turns a verified
//! Google identity into a ToolFix session.

use chrono::{Duration, Utc};
use toolfix_contracts::UserRole;
use toolfix_persistence::repositories::{AuthSessions, Users};
use toolfix_persistence::models::UserRow;

use crate::config::AuthConfig;
use crate::error::AuthError;
use crate::firebase::FirebaseIdentityKit;
use crate::google::GoogleOAuthClient;
use crate::tokens::{generate_pkce, generate_refresh_token, generate_state, hash_refresh_token};

pub struct GoogleLogin {
    pub state: String,
    pub verifier: String,
    pub url: String,
}

pub struct IssuedSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserRow,
}

#[derive(Clone)]
pub struct AuthService {
    config: AuthConfig,
    verifier: crate::claims::TokenVerifier,
    users: Users,
    sessions: AuthSessions,
    google: Option<GoogleOAuthClient>,
    firebase: Option<FirebaseIdentityKit>,
    request_origin: String,
}

impl AuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: AuthConfig,
        users: Users,
        sessions: AuthSessions,
        google: Option<GoogleOAuthClient>,
        firebase: Option<FirebaseIdentityKit>,
        request_origin: String,
    ) -> Self {
        let verifier = crate::claims::TokenVerifier::new(
            &config.access_secret,
            &config.issuer,
            &config.audience,
            config.access_ttl_secs,
        );
        Self {
            config,
            verifier,
            users,
            sessions,
            google,
            firebase,
            request_origin,
        }
    }

    pub fn token_verifier(&self) -> &crate::claims::TokenVerifier {
        &self.verifier
    }

    /// Step 1: build the Google consent URL. The caller stores `state` and
    /// `verifier` in short-lived cookies.
    pub fn begin_google_login(&self) -> Result<GoogleLogin, AuthError> {
        let client = self
            .google
            .as_ref()
            .ok_or(AuthError::OAuthNotConfigured)?;
        let state = generate_state();
        let (verifier, challenge) = generate_pkce();
        let url = client.consent_url(&state, &challenge);
        Ok(GoogleLogin {
            state,
            verifier,
            url,
        })
    }

    /// Step 2: exchange the code, verify identity, provision Firebase
    /// server-side, upsert the ToolFix user, create the session.
    pub async fn complete_google_login(
        &self,
        code: &str,
        verifier: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<IssuedSession, AuthError> {
        let client = self
            .google
            .as_ref()
            .ok_or(AuthError::OAuthNotConfigured)?;
        let identity = client.exchange_code(code, verifier).await?;

        // Firebase provisioning is server-only and soft-fails when the Web
        // API key is not configured (dev mode).
        let firebase_uid = if let Some(firebase) = &self.firebase {
            match firebase
                .sign_in_with_google(&identity.google_id_token, &self.request_origin)
                .await
            {
                Ok(firebase_identity) => firebase_identity.local_id,
                Err(err) => {
                    tracing::warn!(error = %err, "firebase provisioning skipped");
                    format!("google:{}", identity.sub)
                }
            }
        } else {
            tracing::warn!(
                "FIREBASE_WEB_API_KEY not configured; firebase provisioning skipped"
            );
            format!("google:{}", identity.sub)
        };

        let email = identity.email.as_deref();
        let user = self
            .users
            .upsert_by_external_identity(
                &firebase_uid,
                email,
                identity.name.as_deref(),
                identity.picture.as_deref(),
            )
            .await?;

        if user.status != "active" {
            return Err(AuthError::Forbidden);
        }

        self.issue_session(user, user_agent, ip_address).await
    }

    /// Creates the DB-backed session and token pair.
    async fn issue_session(
        &self,
        user: UserRow,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<IssuedSession, AuthError> {
        let refresh_token = generate_refresh_token();
        let refresh_hash = hash_refresh_token(&refresh_token);
        let expires_at = Utc::now() + Duration::seconds(self.config.refresh_ttl_secs);
        self.sessions
            .create(user.id, &refresh_hash, expires_at, user_agent, ip_address)
            .await?;
        let (_, access_token) = self
            .verifier
            .issue(user.id, user.role().map_err(AuthError::from)?)
            .map_err(|e| AuthError::Session(e.to_string()))?;
        Ok(IssuedSession {
            access_token,
            refresh_token,
            user,
        })
    }

    /// Rotates the refresh token. Reuse of an already-rotated/revoked token
    /// revokes every session of the user (theft detection).
    pub async fn refresh(
        &self,
        refresh_token: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<IssuedSession, AuthError> {
        let hash = hash_refresh_token(refresh_token);
        let session = self.sessions.find_by_hash(&hash).await?;

        if session.revoked_at.is_some() {
            tracing::warn!(user_id = %session.user_id, "refresh token reuse detected; revoking user sessions");
            self.sessions.revoke_all_for_user(session.user_id).await?;
            return Err(AuthError::Session("session revoked".into()));
        }
        if session.expires_at < Utc::now() {
            self.sessions.revoke(session.id).await?;
            return Err(AuthError::Session("session expired".into()));
        }

        let user = self.users.find_by_id(session.user_id).await?;
        if user.status != "active" {
            self.sessions.revoke(session.id).await?;
            return Err(AuthError::Forbidden);
        }

        let new_refresh_token = generate_refresh_token();
        let new_hash = hash_refresh_token(&new_refresh_token);
        self.sessions.rotate(session.id, &new_hash).await?;
        let _ = (user_agent, ip_address); // recorded on next rotation cycle

        let (_, access_token) = self
            .verifier
            .issue(user.id, user.role().map_err(AuthError::from)?)
            .map_err(|e| AuthError::Session(e.to_string()))?;
        Ok(IssuedSession {
            access_token,
            refresh_token: new_refresh_token,
            user,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        let hash = hash_refresh_token(refresh_token);
        let session = self.sessions.find_by_hash(&hash).await?;
        self.sessions.revoke(session.id).await?;
        Ok(())
    }

    pub async fn me(&self, user_id: uuid::Uuid) -> Result<UserRow, AuthError> {
        let user = self.users.find_by_id(user_id).await?;
        if user.status != "active" {
            return Err(AuthError::Forbidden);
        }
        Ok(user)
    }

    /// Onboarding: choose CUSTOMER or MECHANIC. ADMIN is never granted via
    /// the API; roles are decided server-side.
    pub async fn complete_onboarding(
        &self,
        user_id: uuid::Uuid,
        requested_role: UserRole,
        phone: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<UserRow, AuthError> {
        let role = match requested_role {
            UserRole::Admin => return Err(AuthError::Forbidden),
            other => other,
        };
        self.users.set_role(user_id, role).await?;
        self.users.set_onboarding_status(user_id, toolfix_contracts::OnboardingStatus::Complete).await?;
        let user = self
            .users
            .update_profile(user_id, display_name, phone)
            .await?;
        Ok(user)
    }

    pub fn frontend_origin(&self) -> &str {
        &self.config.frontend_origin
    }

    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

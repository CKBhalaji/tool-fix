use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("oauth state mismatch")]
    StateMismatch,
    #[error("google oauth is not configured on this server")]
    OAuthNotConfigured,
    #[error("oauth flow error: {0}")]
    OAuth(String),
    #[error("firebase error: {0}")]
    Firebase(String),
    #[error("session error: {0}")]
    Session(String),
    #[error("user not found")]
    UserNotFound,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("database error: {0}")]
    Database(#[from] toolfix_persistence::PersistenceError),
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        AuthError::InvalidToken(value.to_string())
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(value: reqwest::Error) -> Self {
        AuthError::OAuth(value.to_string())
    }
}

/// Axum rejection rendering for the extractor path.
impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            AuthError::Unauthenticated
            | AuthError::InvalidToken(_)
            | AuthError::StateMismatch
            | AuthError::OAuth(_)
            | AuthError::Session(_)
            | AuthError::OAuthNotConfigured => StatusCode::UNAUTHORIZED,
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            AuthError::UserNotFound => StatusCode::NOT_FOUND,
            AuthError::Firebase(_) | AuthError::Storage(_) | AuthError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (
            status,
            axum::Json(toolfix_contracts::error::ErrorBody {
                code: "auth_error".to_string(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

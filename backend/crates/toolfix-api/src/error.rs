//! API error type: maps domain/service errors onto HTTP responses with a
//! stable JSON body.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use toolfix_contracts::ContractError;
use toolfix_jobs::JobsError;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ContractError,
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ContractError::Validation(message.into()),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ContractError::Conflict(message.into()),
        }
    }

    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ContractError::Internal,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        (status, Json(self.body.body())).into_response()
    }
}

impl From<JobsError> for ApiError {
    fn from(err: JobsError) -> Self {
        let (status, body) = match &err {
            JobsError::NotFound => (StatusCode::NOT_FOUND, ContractError::NotFound),
            JobsError::Forbidden => (StatusCode::FORBIDDEN, ContractError::Forbidden),
            JobsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, ContractError::Validation(err.to_string())),
            JobsError::Conflict(_) => (StatusCode::CONFLICT, ContractError::Conflict(err.to_string())),
            JobsError::Domain(_) => (StatusCode::CONFLICT, ContractError::Conflict(err.to_string())),
            JobsError::Database(_) | JobsError::Agent(_) | JobsError::Payment(_) | JobsError::Pricing(_)
            | JobsError::Storage(_) | JobsError::Matching(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ContractError::Internal)
            }
        };
        Self { status, body }
    }
}

impl From<toolfix_auth::AuthError> for ApiError {
    fn from(err: toolfix_auth::AuthError) -> Self {
        let (status, body) = match &err {
            toolfix_auth::AuthError::Unauthenticated => {
                (StatusCode::UNAUTHORIZED, ContractError::Unauthenticated)
            }
            toolfix_auth::AuthError::Forbidden
            | toolfix_auth::AuthError::OAuthNotConfigured => {
                (StatusCode::FORBIDDEN, ContractError::Forbidden)
            }
            toolfix_auth::AuthError::StateMismatch
            | toolfix_auth::AuthError::InvalidToken(_)
            | toolfix_auth::AuthError::Session(_)
            | toolfix_auth::AuthError::OAuth(_) => (
                StatusCode::UNAUTHORIZED,
                ContractError::Unauthenticated,
            ),
            toolfix_auth::AuthError::UserNotFound => (StatusCode::NOT_FOUND, ContractError::NotFound),
            toolfix_auth::AuthError::Database(_)
            | toolfix_auth::AuthError::Firebase(_)
            | toolfix_auth::AuthError::Storage(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ContractError::Internal)
            }
        };
        Self { status, body }
    }
}

impl From<toolfix_persistence::PersistenceError> for ApiError {
    fn from(err: toolfix_persistence::PersistenceError) -> Self {
        match err {
            toolfix_persistence::PersistenceError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                body: ContractError::NotFound,
            },
            other => {
                tracing::error!(error = %other, "persistence error surfaced to API");
                Self {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    body: ContractError::Internal,
                }
            }
        }
    }
}

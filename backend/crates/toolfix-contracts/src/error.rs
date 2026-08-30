//! Contract-level error codes shared by API responses.

use serde::{Deserialize, Serialize};

#[derive(utoipa::ToSchema, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(utoipa::ToSchema, Debug, thiserror::Error)]
pub enum ContractError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error")]
    Internal,
}

impl ContractError {
    pub fn code(&self) -> &'static str {
        match self {
            ContractError::Validation(_) => "validation_failed",
            ContractError::Unauthenticated => "unauthenticated",
            ContractError::Forbidden => "forbidden",
            ContractError::NotFound => "not_found",
            ContractError::Conflict(_) => "conflict",
            ContractError::Internal => "internal_error",
        }
    }

    pub fn body(&self) -> ErrorBody {
        ErrorBody {
            code: self.code().to_string(),
            message: self.to_string(),
        }
    }
}

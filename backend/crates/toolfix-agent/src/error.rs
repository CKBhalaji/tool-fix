use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent configuration error: {0}")]
    Config(String),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("token error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("provider returned unparseable output: {0}")]
    Parse(String),
    #[error("provider output failed validation: {0}")]
    Validation(String),
    #[error("provider error: {0}")]
    Provider(String),
}

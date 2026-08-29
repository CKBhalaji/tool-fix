use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("row not found")]
    NotFound,
    #[error("migration error: {0}")]
    Migration(String),
    #[error("invalid stored value: {0}")]
    InvalidData(String),
}

impl PersistenceError {
    pub fn is_unique_violation(&self) -> bool {
        matches!(
            self,
            PersistenceError::Database(sqlx::Error::Database(db_err))
                if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation
        )
    }
}

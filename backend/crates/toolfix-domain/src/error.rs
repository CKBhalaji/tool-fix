//! Domain-level errors.

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("illegal job transition {from} -> {to}")]
    IllegalTransition {
        from: toolfix_contracts::JobStatus,
        to: toolfix_contracts::JobStatus,
    },
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("job {0} is not accepting offers")]
    NotAcceptingOffers(uuid::Uuid),
    #[error("offer window for job {0} has closed")]
    OfferWindowClosed(uuid::Uuid),
    #[error("actor is not permitted to perform this action")]
    NotPermitted,
}

//! Mechanic offer entity and validation.

use chrono::{DateTime, Utc};
use toolfix_contracts::{Currency, OfferStatus};
use uuid::Uuid;

use crate::error::DomainError;

/// The marketplace proposal. Distinct from the AI estimate and from the
/// final accepted price — those three concepts must never be merged.
#[derive(Debug, Clone)]
pub struct MechanicOffer {
    pub id: Uuid,
    pub job_id: Uuid,
    pub mechanic_id: Uuid,
    pub quoted_price_minor: i64,
    pub currency: Currency,
    pub estimated_arrival_minutes: i32,
    pub message: Option<String>,
    pub status: OfferStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub const MIN_QUOTED_PRICE_MINOR: i64 = 5_000; // ₹50
pub const MAX_QUOTED_PRICE_MINOR: i64 = 50_000_000; // ₹500,000

pub fn validate_new_offer(
    quoted_price_minor: i64,
    estimated_arrival_minutes: i32,
    message: Option<&str>,
) -> Result<(), DomainError> {
    if !(MIN_QUOTED_PRICE_MINOR..=MAX_QUOTED_PRICE_MINOR).contains(&quoted_price_minor) {
        return Err(DomainError::Validation(format!(
            "quoted_price_minor must be between {MIN_QUOTED_PRICE_MINOR} and {MAX_QUOTED_PRICE_MINOR} paise"
        )));
    }
    if !(1..=600).contains(&estimated_arrival_minutes) {
        return Err(DomainError::Validation(
            "estimated_arrival_minutes must be between 1 and 600".into(),
        ));
    }
    if let Some(msg) = message
        && msg.len() > 1000
    {
        return Err(DomainError::Validation(
            "offer message must be at most 1000 characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonsense_prices_and_etas() {
        assert!(validate_new_offer(100, 20, None).is_err());
        assert!(validate_new_offer(100_000, 0, None).is_err());
        assert!(validate_new_offer(100_000, 20, None).is_ok());
    }
}

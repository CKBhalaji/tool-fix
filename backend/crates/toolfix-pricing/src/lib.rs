//! Pricing service.
//!
//! Keeps the three price concepts strictly separate:
//! 1. **AI estimate** — advisory range produced by the agent layer.
//! 2. **Mechanic offer** — marketplace proposal, created only by mechanics.
//! 3. **Final price** — the accepted offer amount (+ approved extras).
//!
//! The AI never sets the final price and never modifies offers.

use toolfix_contracts::RepairCategory;
use toolfix_persistence::repositories::Pricing;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoricalContext {
    pub sample_count: i64,
    pub avg_amount_minor: Option<i64>,
    pub min_amount_minor: Option<i64>,
    pub max_amount_minor: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum PricingError {
    #[error("database error: {0}")]
    Database(#[from] toolfix_persistence::PersistenceError),
    #[error("pricing validation failed: {0}")]
    Validation(String),
}

#[derive(Clone)]
pub struct PricingService {
    pricing: Pricing,
}

impl PricingService {
    pub fn new(pricing: Pricing) -> Self {
        Self { pricing }
    }

    /// Historical stats to condition an agent estimate on.
    pub async fn historical_context(
        &self,
        category: RepairCategory,
        vehicle_kind: Option<&str>,
        city: Option<&str>,
    ) -> Result<HistoricalContext, PricingError> {
        let stats = self
            .pricing
            .historical_stats(category, vehicle_kind, city)
            .await?;
        Ok(HistoricalContext {
            sample_count: stats.sample_count,
            avg_amount_minor: stats.avg_amount_minor.map(|v| v.round() as i64),
            min_amount_minor: stats.min_amount_minor,
            max_amount_minor: stats.max_amount_minor,
        })
    }

    /// Persists an agent estimate after validating its range. Advisory only.
    pub async fn record_agent_estimate(
        &self,
        breakdown_id: uuid::Uuid,
        job_id: Option<uuid::Uuid>,
        category: RepairCategory,
        min_minor: i64,
        max_minor: i64,
        notes: Option<&str>,
    ) -> Result<(), PricingError> {
        toolfix_domain::pricing::validate_amount_range(min_minor, max_minor)
            .map_err(PricingError::Validation)?;
        self.pricing
            .record_estimate(breakdown_id, job_id, category, min_minor, max_minor, "agent", notes)
            .await?;
        Ok(())
    }

    /// Records the realized (final) price of a completed job — the
    /// marketplace truth the historical layer learns from.
    pub async fn record_final_price(
        &self,
        job_id: uuid::Uuid,
        category: RepairCategory,
        vehicle_kind: Option<&str>,
        city: Option<&str>,
        final_amount_minor: i64,
    ) -> Result<(), PricingError> {
        if final_amount_minor <= 0 {
            return Err(PricingError::Validation(
                "final amount must be positive".into(),
            ));
        }
        self.pricing
            .record_final_price(job_id, category, vehicle_kind, city, final_amount_minor)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn concepts_never_merge() {
        // Compile-time documentation of the model distinction.
        let estimate = toolfix_domain::pricing::PriceConcept::AiEstimate;
        let offer = toolfix_domain::pricing::PriceConcept::MechanicOffer;
        let final_price = toolfix_domain::pricing::PriceConcept::FinalPrice;
        assert_ne!(estimate, offer);
        assert_ne!(offer, final_price);
    }
}

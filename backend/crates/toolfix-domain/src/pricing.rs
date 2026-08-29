//! Pricing domain rules.
//!
//! Three concepts exist and must never be merged:
//! 1. AI estimate — informational range produced by the agent layer.
//! 2. Mechanic offer — a marketplace proposal from a mechanic.
//! 3. Final price — the accepted offer amount (+ explicitly approved extras).

use toolfix_contracts::Currency;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceConcept {
    AiEstimate,
    MechanicOffer,
    FinalPrice,
}

#[derive(Debug, Clone)]
pub struct Money {
    pub amount_minor: i64,
    pub currency: Currency,
}

impl Money {
    pub fn rupees(&self) -> f64 {
        self.amount_minor as f64 / 100.0
    }
}

/// Guardrail used before persisting or displaying any price: amounts must be
/// positive and an estimate range must be ordered and bounded.
pub fn validate_amount_range(min_minor: i64, max_minor: i64) -> Result<(), String> {
    if min_minor <= 0 || max_minor <= 0 {
        return Err("amounts must be positive".into());
    }
    if min_minor > max_minor {
        return Err("min amount must not exceed max amount".into());
    }
    // Sanity ceiling: no roadside repair estimate exceeds ₹5,00,000.
    const CEILING: i64 = 50_000_000;
    if max_minor > CEILING {
        return Err("amount range exceeds sanity ceiling".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inverted_or_negative_ranges() {
        assert!(validate_amount_range(100, 200).is_ok());
        assert!(validate_amount_range(300, 200).is_err());
        assert!(validate_amount_range(-1, 200).is_err());
    }

    #[test]
    fn money_converts_to_rupees() {
        let money = Money {
            amount_minor: 65_000,
            currency: Currency::Inr,
        };
        assert!((money.rupees() - 650.0).abs() < f64::EPSILON);
    }
}

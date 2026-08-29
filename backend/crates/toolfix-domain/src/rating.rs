//! Rating rules.

use crate::error::DomainError;

pub fn validate_score(score: i16) -> Result<(), DomainError> {
    if !(1..=5).contains(&score) {
        return Err(DomainError::Validation(
            "rating score must be between 1 and 5".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_are_enforced() {
        assert!(validate_score(1).is_ok());
        assert!(validate_score(5).is_ok());
        assert!(validate_score(0).is_err());
        assert!(validate_score(6).is_err());
    }
}

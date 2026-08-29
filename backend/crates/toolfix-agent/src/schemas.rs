//! Structured agent outputs. Every field is validated before use —
//! raw AI output never reaches the database directly.

use serde::Deserialize;
use toolfix_contracts::{EstimateSource, RepairCategory, Severity};

use crate::error::AgentError;

pub const MAX_SUMMARY_CHARS: usize = 1000;

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosisOutput {
    pub possible_issue: String,
    pub confidence: f64,
    pub severity: String,
    pub recommended_service: String,
    pub repair_category: String,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    #[serde(default)]
    pub requires_towing: bool,
    pub reasoning_summary: String,
}

impl DiagnosisOutput {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, AgentError> {
        let parsed: DiagnosisOutput = serde_json::from_value(value.clone())
            .map_err(|e| AgentError::Parse(format!("diagnosis schema mismatch: {e}")))?;
        parsed.validate()?;
        Ok(parsed)
    }

    /// Enforces the agent safety contract: bounded confidence, known
    /// categories, ordered positive ranges, concise summaries.
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.possible_issue.trim().is_empty() || self.possible_issue.len() > 120 {
            return Err(AgentError::Validation("possible_issue must be 1-120 chars".into()));
        }
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(AgentError::Validation("confidence must be within [0,1]".into()));
        }
        if Severity::parse(&self.severity).is_none() {
            return Err(AgentError::Validation(format!(
                "unknown severity: {}",
                self.severity
            )));
        }
        if RepairCategory::parse(&self.repair_category).is_none() {
            return Err(AgentError::Validation(format!(
                "unknown repair_category: {}",
                self.repair_category
            )));
        }
        if self.recommended_service.trim().is_empty() || self.recommended_service.len() > 200 {
            return Err(AgentError::Validation(
                "recommended_service must be 1-200 chars".into(),
            ));
        }
        toolfix_domain::pricing::validate_amount_range(
            self.estimated_cost_min_minor,
            self.estimated_cost_max_minor,
        )
        .map_err(AgentError::Validation)?;
        if self.reasoning_summary.len() > MAX_SUMMARY_CHARS {
            return Err(AgentError::Validation(format!(
                "reasoning_summary must be at most {MAX_SUMMARY_CHARS} chars"
            )));
        }
        Ok(())
    }

    pub fn severity(&self) -> Severity {
        Severity::parse(&self.severity).unwrap_or(Severity::Medium)
    }

    pub fn repair_category(&self) -> RepairCategory {
        RepairCategory::parse(&self.repair_category).unwrap_or(RepairCategory::Other)
    }

    pub fn to_dto(&self) -> toolfix_contracts::breakdown::AgentDiagnosisDto {
        toolfix_contracts::breakdown::AgentDiagnosisDto {
            possible_issue: self.possible_issue.clone(),
            confidence: self.confidence,
            severity: self.severity(),
            recommended_service: self.recommended_service.clone(),
            repair_category: self.repair_category(),
            estimated_cost_min_minor: self.estimated_cost_min_minor,
            estimated_cost_max_minor: self.estimated_cost_max_minor,
            requires_towing: self.requires_towing,
            reasoning_summary: self.reasoning_summary.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceEstimateOutput {
    pub repair_category: String,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    #[serde(default)]
    pub notes: Option<String>,
}

impl PriceEstimateOutput {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, AgentError> {
        let parsed: PriceEstimateOutput = serde_json::from_value(value.clone())
            .map_err(|e| AgentError::Parse(format!("price estimate schema mismatch: {e}")))?;
        parsed.validate()?;
        Ok(parsed)
    }

    pub fn validate(&self) -> Result<(), AgentError> {
        if RepairCategory::parse(&self.repair_category).is_none() {
            return Err(AgentError::Validation(format!(
                "unknown repair_category: {}",
                self.repair_category
            )));
        }
        toolfix_domain::pricing::validate_amount_range(
            self.estimated_cost_min_minor,
            self.estimated_cost_max_minor,
        )
        .map_err(AgentError::Validation)?;
        if let Some(notes) = &self.notes
            && notes.len() > MAX_SUMMARY_CHARS
        {
            return Err(AgentError::Validation("notes too long".into()));
        }
        Ok(())
    }

    pub fn repair_category(&self) -> RepairCategory {
        RepairCategory::parse(&self.repair_category).unwrap_or(RepairCategory::Other)
    }

    pub fn source(&self) -> EstimateSource {
        EstimateSource::Agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_diagnosis() -> serde_json::Value {
        json!({
            "possible_issue": "battery_or_ignition_issue",
            "confidence": 0.78,
            "severity": "medium",
            "recommended_service": "diagnostic_and_battery_check",
            "repair_category": "battery",
            "estimated_cost_min_minor": 20_000,
            "estimated_cost_max_minor": 80_000,
            "requires_towing": false,
            "reasoning_summary": "Sudden stop with cranking engine suggests battery or ignition."
        })
    }

    #[test]
    fn accepts_valid_diagnosis() {
        let out = DiagnosisOutput::from_value(&valid_diagnosis()).unwrap();
        assert_eq!(out.severity(), Severity::Medium);
        assert_eq!(out.repair_category(), RepairCategory::Battery);
    }

    #[test]
    fn rejects_out_of_range_confidence() {
        let mut v = valid_diagnosis();
        v["confidence"] = json!(1.5);
        assert!(matches!(
            DiagnosisOutput::from_value(&v),
            Err(AgentError::Validation(_))
        ));
    }

    #[test]
    fn rejects_inverted_cost_range() {
        let mut v = valid_diagnosis();
        v["estimated_cost_min_minor"] = json!(90_000);
        v["estimated_cost_max_minor"] = json!(80_000);
        assert!(matches!(
            DiagnosisOutput::from_value(&v),
            Err(AgentError::Validation(_))
        ));
    }

    #[test]
    fn rejects_unknown_category() {
        let mut v = valid_diagnosis();
        v["repair_category"] = json!("warp_drive");
        assert!(matches!(
            DiagnosisOutput::from_value(&v),
            Err(AgentError::Validation(_))
        ));
    }

    #[test]
    fn accepts_valid_price_estimate() {
        let out = PriceEstimateOutput::from_value(&json!({
            "repair_category": "tyre",
            "estimated_cost_min_minor": 50_000,
            "estimated_cost_max_minor": 150_000,
            "notes": "Includes tube replacement."
        }))
        .unwrap();
        assert_eq!(out.repair_category(), RepairCategory::Tyre);
        assert_eq!(out.source(), EstimateSource::Agent);
    }
}

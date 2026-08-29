//! Prompt construction. Prompts demand strict JSON output and forbid
//! chain-of-thought: only a short, user-safe reasoning summary.

use crate::provider::{DiagnosisInput, PriceEstimateInput};

pub const DIAGNOSIS_INSTRUCTION: &str = r#"
You are ToolFix's roadside breakdown diagnostician for India. Analyze the
reported vehicle problem (and the attached photo/video frame if present) and
respond with ONLY a JSON object — no markdown, no prose outside JSON — with
exactly these fields:

{
  "possible_issue": "short snake_case identifier, e.g. battery_or_ignition_issue",
  "confidence": 0.0,
  "severity": "low" | "medium" | "high" | "critical",
  "recommended_service": "short snake_case service description",
  "repair_category": one of ["battery","tyre","engine","electrical","brakes","clutch",
                             "fuel","chain_drive","cooling","body_damage","lockout",
                             "towing","diagnostic","other"],
  "estimated_cost_min_minor": integer paise (INR * 100),
  "estimated_cost_max_minor": integer paise, >= min,
  "requires_towing": true | false,
  "reasoning_summary": "at most 3 factual sentences a stranded customer can read"
}

Rules:
- Costs are typical Indian roadside/on-site repair ranges in paise.
- Do NOT reveal hidden chain-of-thought; the reasoning_summary is the only
  explanation and must be concise and factual.
- If the photo is unclear, lower confidence and rely on the description.
"#;

pub const PRICE_INSTRUCTION: &str = r#"
You are ToolFix's repair pricing analyst for India. Given the repair category,
vehicle, severity, description and historical price statistics, respond with
ONLY a JSON object — no markdown, no prose outside JSON — with exactly:

{
  "repair_category": one of ["battery","tyre","engine","electrical","brakes","clutch",
                             "fuel","chain_drive","cooling","body_damage","lockout",
                             "towing","diagnostic","other"],
  "estimated_cost_min_minor": integer paise (INR * 100),
  "estimated_cost_max_minor": integer paise, >= min,
  "notes": "optional, at most 3 short factual sentences"
}

Rules:
- Estimate parts + labour + typical travel charge for an on-site repair.
- The historical statistics are the strongest signal when sample_count > 0.
- Your estimate is ADVISORY: mechanics will bid independently. Never assume
  your range is the final price.
"#;

fn vehicle_line(vehicle_kind: Option<toolfix_contracts::VehicleKind>, display: Option<&str>) -> String {
    let kind = vehicle_kind
        .map(|k| k.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    match display {
        Some(name) => format!("Vehicle: {name} (kind: {kind})"),
        None => format!("Vehicle kind: {kind}"),
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        text.chars().take(max).collect()
    }
}

pub fn diagnosis_prompt(input: &DiagnosisInput) -> String {
    let mut prompt = String::new();
    prompt.push_str(DIAGNOSIS_INSTRUCTION);
    prompt.push_str("\n\nInput:\n");
    prompt.push_str(&vehicle_line(input.vehicle_kind, input.vehicle_display.as_deref()));
    prompt.push_str(&format!(
        "\nDescription: {}",
        truncate(&input.description, 4000)
    ));
    if !input.symptoms.is_empty() {
        prompt.push_str(&format!("\nSymptoms: {}", input.symptoms.join("; ")));
    }
    if input.image_base64.is_some() {
        prompt.push_str("\nAn attached photo follows; inspect it for visible faults.");
    }
    prompt
}

pub fn price_prompt(input: &PriceEstimateInput) -> String {
    let mut prompt = String::new();
    prompt.push_str(PRICE_INSTRUCTION);
    prompt.push_str("\n\nInput:\n");
    prompt.push_str(&vehicle_line(
        input.vehicle_kind,
        None,
    ));
    if let Some(category) = input.repair_category {
        prompt.push_str(&format!("\nRepair category: {}", category.as_str()));
    }
    if let Some(severity) = input.severity {
        prompt.push_str(&format!("\nSeverity: {}", severity.as_str()));
    }
    prompt.push_str(&format!(
        "\nDescription: {}",
        truncate(&input.description, 2000)
    ));
    if let Some(distance) = input.travel_distance_km {
        prompt.push_str(&format!("\nMechanic travel distance: {distance:.1} km"));
    }
    if let Some(historical) = &input.historical {
        prompt.push_str(&format!(
            "\nHistorical completed jobs (category+vehicle): count={}, avg_minor={:?}, min_minor={:?}, max_minor={:?}",
            historical.sample_count,
            historical.avg_amount_minor,
            historical.min_amount_minor,
            historical.max_amount_minor,
        ));
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_do_not_leak_chain_of_thought() {
        assert!(DIAGNOSIS_INSTRUCTION.contains("Do NOT reveal hidden chain-of-thought"));
    }

    #[test]
    fn diagnosis_prompt_includes_symptoms() {
        let input = DiagnosisInput {
            description: "Bike suddenly stopped".into(),
            symptoms: vec!["engine turns but does not start".into()],
            ..Default::default()
        };
        let prompt = diagnosis_prompt(&input);
        assert!(prompt.contains("engine turns but does not start"));
    }

    #[test]
    fn long_input_is_truncated() {
        let long = "x".repeat(10_000);
        let input = DiagnosisInput {
            description: long,
            ..Default::default()
        };
        let prompt = diagnosis_prompt(&input);
        assert!(prompt.len() < 10_000);
    }
}

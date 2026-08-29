# Agentic Layer

`toolfix-agent` is the isolated AI subsystem. It **assists** the
deterministic marketplace; it never controls it.

## Responsibilities

- Problem understanding from the customer's description + symptoms
- Image analysis of uploaded breakdown photos (multimodal)
- Fault classification (`possible_issue`, confidence, severity)
- Repair-category classification (feeds deterministic matching)
- Cost estimation (advisory range conditioned on historical prices)
- Job summarization (concise, user-safe)

## Provider abstraction

```rust
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> &str;
    async fn generate_json(&self, prompt: &str, image: Option<(&str, &str)>)
        -> Result<serde_json::Value, AgentError>;
}
```

Two implementations ship, selected by the `AI_PROVIDER` environment value:

| Value | Implementation | Auth |
|---|---|---|
| `gemini_api` (default) | `gemini_api.rs` — `generativelanguage.googleapis.com` REST | `GEMINI_API_KEY` |
| `vertex_ai` | `vertex.rs` — Vertex `generateContent` | Service account (`GOOGLE_APPLICATION_CREDENTIALS`), JWT-bearer token exchange, cached |

Both request strict JSON (`response_mime_type: application/json`), tolerate
markdown fences, and otherwise behave identically — the rest of the
application cannot tell which provider is configured. Test fakes implement
the same trait.

## Workflow

```text
Breakdown created
   → Input assembly (description, symptoms, vehicle, photo bytes from storage)
   → Provider call (prompt from prompts.rs; strict JSON contract)
   → Schema validation (schemas.rs)          ← reject on any deviation
   → Business rules (pricing guardrails)     ← reject out-of-range outputs
   → Persistence: agent_runs / agent_outputs / breakdown_diagnoses / price_estimates
   → Category feeds deterministic matching
```

## Structured output contract

Diagnosis (validated bounds in `schemas.rs`):

```json
{
  "possible_issue": "battery_or_ignition_issue",
  "confidence": 0.0,
  "severity": "low|medium|high|critical",
  "recommended_service": "…",
  "repair_category": "battery|…|other",
  "estimated_cost_min_minor": 20000,
  "estimated_cost_max_minor": 80000,
  "requires_towing": false,
  "reasoning_summary": "at most 3 factual sentences"
}
```

Validation rejects: confidence outside `[0,1]`, unknown severity/category,
inverted or non-positive cost ranges, over-long summaries. No hidden
chain-of-thought is requested or stored — only the concise summary.

## Safety rules (mandatory)

The AI is **advisory**. Raw model output can never:

- charge a customer or move money
- approve or verify a mechanic
- change payment amounts, job ownership, or permissions
- complete/cancel jobs or bypass authorization
- modify database state without validation + business rules

Pipeline: `AI result → schema validation → business rules → application
decision → database`. Provider identity, model, latency, and outputs are
recorded in `agent_runs`/`agent_outputs` for every call, successful or not.

## Price concepts (never merge)

1. **AI estimate** — informational range (`price_estimates.source = 'agent'`).
2. **Mechanic offer** — marketplace proposal created only by mechanics.
3. **Final price** — the accepted offer amount, recorded to `price_history`
   on completion and used as future historical signal.

The AI never sets the final price and never touches mechanic offers.

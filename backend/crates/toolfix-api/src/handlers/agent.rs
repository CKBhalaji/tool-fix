//! Agent handlers: stateless, advisory diagnosis endpoint.

use axum::extract::State;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::breakdown::AgentDiagnosisDto;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct AgentDiagnoseRequest {
    pub description: String,
    pub vehicle_kind: Option<toolfix_contracts::VehicleKind>,
    pub vehicle_display: Option<String>,
    pub symptoms: Vec<String>,
    pub image_base64: Option<String>,
    pub image_mime: Option<String>,
}

/// POST /api/v1/agent/diagnose — stateless advisory diagnosis. Nothing is
/// persisted and nothing in the marketplace changes as a result; the
/// persisted path runs inside the job pipeline where outputs are
/// validated and recorded with an audit trail.
#[utoipa::path(post, path = "/api/v1/agent/diagnose", tag = "agent", operation_id = "agent_diagnose", request_body = AgentDiagnoseRequest, responses((status = 200, body = AgentDiagnosisDto)))]
pub async fn diagnose(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(request): Json<AgentDiagnoseRequest>,
) -> Result<Json<toolfix_contracts::breakdown::AgentDiagnosisDto>, ApiError> {
    let agent = state
        .agent
        .as_ref()
        .ok_or_else(|| ApiError::conflict("AI provider is not configured on this server"))?;
    if request.description.trim().len() < 10 {
        return Err(ApiError::validation(
            "description must be at least 10 characters",
        ));
    }
    let input = toolfix_agent::DiagnosisInput {
        vehicle_kind: request.vehicle_kind,
        vehicle_display: request.vehicle_display,
        description: request.description,
        symptoms: request.symptoms,
        image_base64: request.image_base64,
        image_mime: request.image_mime,
    };
    let output = agent
        .diagnose_breakdown(&input)
        .await
        .map_err(toolfix_jobs::JobsError::from)
        .map_err(ApiError::from)?;
    Ok(Json(output.to_dto()))
}

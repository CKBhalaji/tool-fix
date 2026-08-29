//! Breakdown handlers: create (fires the AI + matching pipeline), read,
//! media upload, cancel.

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use crate::{ApiError, AppState};
use toolfix_auth::AuthUser;
use toolfix_contracts::breakdown::{BreakdownCreateRequest, DiagnosisResponse, PriceEstimateDto};
use toolfix_contracts::breakdown::BreakdownMediaResponse;
use toolfix_contracts::job::JobResponse;
use toolfix_persistence::models::{BreakdownRow, DiagnosisRow};

use crate::handlers::jobs::job_response;

/// Combined view returned by GET /api/v1/breakdowns/{id}.
#[derive(serde::Serialize)]
pub struct BreakdownDetail {
    pub breakdown: BreakdownRowView,
    pub job: JobResponse,
    pub diagnosis: Option<toolfix_contracts::breakdown::AgentDiagnosisDto>,
    pub price_estimate: Option<PriceEstimateDto>,
    pub media: Vec<BreakdownMediaResponse>,
}

/// Serializable breakdown projection (raw rows are internal).
#[derive(serde::Serialize)]
pub struct BreakdownRowView {
    pub id: uuid::Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&BreakdownRow> for BreakdownRowView {
    fn from(row: &BreakdownRow) -> Self {
        Self {
            id: row.id,
            latitude: row.latitude,
            longitude: row.longitude,
            address: row.address.clone(),
            problem_description: row.problem_description.clone(),
            vehicle_symptoms: row.vehicle_symptoms.clone(),
            created_at: row.created_at,
        }
    }
}

fn diagnosis_dto(row: &DiagnosisRow) -> Result<toolfix_contracts::breakdown::AgentDiagnosisDto, ApiError> {
    Ok(toolfix_contracts::breakdown::AgentDiagnosisDto {
        possible_issue: row.possible_issue.clone(),
        confidence: row.confidence as f64,
        severity: row.severity()?,
        recommended_service: row.recommended_service.clone(),
        repair_category: row.category()?,
        estimated_cost_min_minor: row.estimated_cost_min_minor,
        estimated_cost_max_minor: row.estimated_cost_max_minor,
        requires_towing: row.requires_towing,
        reasoning_summary: row.reasoning_summary.clone(),
    })
}

fn estimate_dto(
    row: &toolfix_persistence::models::PriceEstimateRow,
) -> Result<PriceEstimateDto, ApiError> {
    Ok(PriceEstimateDto {
        repair_category: row.category()?,
        estimated_cost_min_minor: row.estimated_cost_min_minor,
        estimated_cost_max_minor: row.estimated_cost_max_minor,
        currency: toolfix_contracts::Currency::Inr,
        source: row.source()?,
        notes: row.notes.clone(),
        created_at: row.created_at,
    })
}

/// POST /api/v1/breakdowns
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<BreakdownCreateRequest>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let (_breakdown, job) = state.jobs.create_breakdown(auth.user_id, request).await?;
    Ok((StatusCode::ACCEPTED, Json(job_response(&job)?)))
}

/// GET /api/v1/breakdowns/{id}
pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(breakdown_id): Path<uuid::Uuid>,
) -> Result<Json<BreakdownDetail>, ApiError> {
    let breakdown = state.jobs.breakdown(breakdown_id).await?;
    if breakdown.user_id != auth.user_id {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    let job = state
        .repos
        .jobs
        .find_by_breakdown(breakdown_id)
        .await?;
    let diagnosis = state.jobs.diagnosis(breakdown_id).await?;
    let estimate = state.jobs.estimate(breakdown_id).await?;
    let media = state
        .jobs
        .media(breakdown_id)
        .await?
        .into_iter()
        .map(|m| {
            Ok(BreakdownMediaResponse {
                id: m.id,
                breakdown_id: m.breakdown_id,
                media_kind: m.kind()?,
                storage_key: m.storage_key,
                content_type: m.content_type,
                created_at: m.created_at,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(BreakdownDetail {
        breakdown: (&breakdown).into(),
        job: job_response(&job)?,
        diagnosis: diagnosis.as_ref().map(diagnosis_dto).transpose()?,
        price_estimate: estimate.as_ref().map(estimate_dto).transpose()?,
        media,
    }))
}

/// POST /api/v1/breakdowns/{id}/media (multipart photo/video upload)
pub async fn upload_media(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(breakdown_id): Path<uuid::Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<BreakdownMediaResponse>>, ApiError> {
    let mut uploaded = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::validation(e.to_string()))?
    {
        let media_kind = match field.name().unwrap_or_default() {
            "photo" | "image" => toolfix_contracts::MediaKind::Photo,
            "video" => toolfix_contracts::MediaKind::Video,
            other => {
                return Err(ApiError::validation(format!(
                    "unexpected upload field '{other}'; use photo or video"
                )))
            }
        };
        let content_type = field.content_type().map(str::to_string);
        let data = field
            .bytes()
            .await
            .map_err(|e| ApiError::validation(format!("upload too large or invalid: {e}")))?;
        if data.is_empty() {
            return Err(ApiError::validation("empty upload"));
        }
        let row = state
            .jobs
            .add_media(auth.user_id, breakdown_id, media_kind, content_type.as_deref(), data)
            .await?;
        uploaded.push(BreakdownMediaResponse {
            id: row.id,
            breakdown_id: row.breakdown_id,
            media_kind: row.kind()?,
            storage_key: row.storage_key,
            content_type: row.content_type,
            created_at: row.created_at,
        });
    }
    if uploaded.is_empty() {
        return Err(ApiError::validation("no media fields found; use photo or video"));
    }
    Ok(Json(uploaded))
}

/// GET /api/v1/breakdowns/{id}/media/{media_id}/content — serves stored
/// bytes through the storage abstraction (local FS first).
pub async fn download_media(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((breakdown_id, _media_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let breakdown = state.jobs.breakdown(breakdown_id).await?;
    if breakdown.user_id != auth.user_id {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    let media = state.jobs.media(breakdown_id).await?;
    let media = media
        .first()
        .ok_or(toolfix_persistence::PersistenceError::NotFound)?;
    let bytes = state.jobs.read_media(&media.storage_key).await?;
    let content_type = media
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".into());
    Ok((
        [(axum::http::header::CONTENT_TYPE, content_type)],
        bytes,
    ))
}

/// GET /api/v1/breakdowns/{id}/diagnosis — advisory AI output only.
pub async fn diagnosis(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(breakdown_id): Path<uuid::Uuid>,
) -> Result<Json<DiagnosisResponse>, ApiError> {
    let breakdown = state.jobs.breakdown(breakdown_id).await?;
    if breakdown.user_id != auth.user_id {
        return Err(ApiError::from(toolfix_auth::AuthError::Forbidden));
    }
    let diagnosis = state
        .jobs
        .diagnosis(breakdown_id)
        .await?
        .ok_or(toolfix_persistence::PersistenceError::NotFound)?;
    let estimate = state
        .jobs
        .estimate(breakdown_id)
        .await?
        .ok_or(toolfix_persistence::PersistenceError::NotFound)?;
    Ok(Json(DiagnosisResponse {
        breakdown_id,
        diagnosis: diagnosis_dto(&diagnosis)?,
        price_estimate: estimate_dto(&estimate)?,
    }))
}

/// POST /api/v1/breakdowns/{id}/cancel
pub async fn cancel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(breakdown_id): Path<uuid::Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = state
        .repos
        .jobs
        .find_by_breakdown(breakdown_id)
        .await?;
    let updated = state.jobs.cancel_job(auth.user_id, job.id).await?;
    Ok(Json(job_response(&updated)?))
}

//! Breakdowns repository (report + media + diagnoses).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use toolfix_contracts::MediaKind;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::{BreakdownMediaRow, BreakdownRow, DiagnosisRow};

#[derive(Clone)]
pub struct Breakdowns {
    pool: PgPool,
}

impl Breakdowns {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        user_id: Uuid,
        vehicle_id: Uuid,
        latitude: f64,
        longitude: f64,
        address: Option<&str>,
        problem_description: &str,
        vehicle_symptoms: &[String],
    ) -> Result<BreakdownRow, PersistenceError> {
        sqlx::query_as::<_, BreakdownRow>(
            r#"
            INSERT INTO breakdowns (id, user_id, vehicle_id, latitude, longitude, address,
                                    problem_description, vehicle_symptoms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(vehicle_id)
        .bind(latitude)
        .bind(longitude)
        .bind(address)
        .bind(problem_description)
        .bind(vehicle_symptoms)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn find_by_id(&self, breakdown_id: Uuid) -> Result<BreakdownRow, PersistenceError> {
        sqlx::query_as::<_, BreakdownRow>("SELECT * FROM breakdowns WHERE id = $1")
            .bind(breakdown_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    pub async fn add_media(
        &self,
        breakdown_id: Uuid,
        media_kind: MediaKind,
        storage_key: &str,
        content_type: Option<&str>,
        size_bytes: Option<i64>,
    ) -> Result<BreakdownMediaRow, PersistenceError> {
        sqlx::query_as::<_, BreakdownMediaRow>(
            r#"
            INSERT INTO breakdown_media (id, breakdown_id, media_kind, storage_key, content_type, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(breakdown_id)
        .bind(media_kind.as_str())
        .bind(storage_key)
        .bind(content_type)
        .bind(size_bytes)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn list_media(
        &self,
        breakdown_id: Uuid,
    ) -> Result<Vec<BreakdownMediaRow>, PersistenceError> {
        sqlx::query_as::<_, BreakdownMediaRow>(
            "SELECT * FROM breakdown_media WHERE breakdown_id = $1 ORDER BY created_at",
        )
        .bind(breakdown_id)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    /// Upserts the (advisory) diagnosis for a breakdown.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_diagnosis(
        &self,
        breakdown_id: Uuid,
        agent_run_id: Option<Uuid>,
        possible_issue: &str,
        confidence: f64,
        severity: toolfix_contracts::Severity,
        recommended_service: &str,
        repair_category: toolfix_contracts::RepairCategory,
        estimated_cost_min_minor: i64,
        estimated_cost_max_minor: i64,
        requires_towing: bool,
        reasoning_summary: &str,
    ) -> Result<DiagnosisRow, PersistenceError> {
        sqlx::query_as::<_, DiagnosisRow>(
            r#"
            INSERT INTO breakdown_diagnoses (id, breakdown_id, agent_run_id, possible_issue, confidence,
                                             severity, recommended_service, repair_category,
                                             estimated_cost_min_minor, estimated_cost_max_minor,
                                             requires_towing, reasoning_summary)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (breakdown_id) DO UPDATE
                SET agent_run_id = EXCLUDED.agent_run_id,
                    possible_issue = EXCLUDED.possible_issue,
                    confidence = EXCLUDED.confidence,
                    severity = EXCLUDED.severity,
                    recommended_service = EXCLUDED.recommended_service,
                    repair_category = EXCLUDED.repair_category,
                    estimated_cost_min_minor = EXCLUDED.estimated_cost_min_minor,
                    estimated_cost_max_minor = EXCLUDED.estimated_cost_max_minor,
                    requires_towing = EXCLUDED.requires_towing,
                    reasoning_summary = EXCLUDED.reasoning_summary
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(breakdown_id)
        .bind(agent_run_id)
        .bind(possible_issue)
        .bind(confidence as f32)
        .bind(severity.as_str())
        .bind(recommended_service)
        .bind(repair_category.as_str())
        .bind(estimated_cost_min_minor)
        .bind(estimated_cost_max_minor)
        .bind(requires_towing)
        .bind(reasoning_summary)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn find_diagnosis(
        &self,
        breakdown_id: Uuid,
    ) -> Result<DiagnosisRow, PersistenceError> {
        sqlx::query_as::<_, DiagnosisRow>(
            "SELECT * FROM breakdown_diagnoses WHERE breakdown_id = $1",
        )
        .bind(breakdown_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::NotFound)
    }

    /// Breakdowns with an actively-notified job inside a bounding box, used
    /// for the mechanic requests feed; exact distance filtering happens in
    /// matching.
    pub async fn recent_open_for_feed(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lng: f64,
        max_lng: f64,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<BreakdownRow>, PersistenceError> {
        sqlx::query_as::<_, BreakdownRow>(
            r#"
            SELECT b.*
              FROM breakdowns b
             WHERE b.created_at >= $5
               AND b.latitude BETWEEN $1 AND $2
               AND b.longitude BETWEEN $3 AND $4
               AND EXISTS (SELECT 1 FROM assistance_jobs j
                            WHERE j.breakdown_id = b.id
                              AND j.status IN ('mechanics_notified', 'offers_received'))
             ORDER BY b.created_at DESC
             LIMIT $6
            "#,
        )
        .bind(min_lat)
        .bind(max_lat)
        .bind(min_lng)
        .bind(max_lng)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }
}

//! Agent run/output persistence (the AI audit trail).

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::AgentRunRow;

#[derive(Clone)]
pub struct Agents {
    pool: PgPool,
}

impl Agents {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn record_run(
        &self,
        breakdown_id: Uuid,
        job_id: Option<Uuid>,
        kind: &str,
        provider: &str,
        model: Option<&str>,
        status: &str,
        error_message: Option<&str>,
        latency_ms: i64,
        output: serde_json::Value,
    ) -> Result<AgentRunRow, PersistenceError> {
        let mut tx = self.pool.begin().await?;
        let run = sqlx::query_as::<_, AgentRunRow>(
            r#"
            INSERT INTO agent_runs (id, breakdown_id, job_id, kind, provider, model, status, error_message, latency_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(breakdown_id)
        .bind(job_id)
        .bind(kind)
        .bind(provider)
        .bind(model)
        .bind(status)
        .bind(error_message)
        .bind(latency_ms as i32)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO agent_outputs (id, agent_run_id, output) VALUES ($1, $2, $3)")
            .bind(Uuid::now_v7())
            .bind(run.id)
            .bind(output)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(run)
    }
}

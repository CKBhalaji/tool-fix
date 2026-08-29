//! Agent run/output persistence (the AI audit trail).

use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::AgentRunRow;
use crate::{dual_tx, Db};

#[derive(Clone)]
pub struct Agents {
    db: Db,
}

impl Agents {
    pub fn new(db: Db) -> Self {
        Self { db }
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
        let mut tx = self.db.begin().await?;
        let run: AgentRunRow = dual_tx!(
            &mut tx,
            |e| sqlx::query_as::<_, AgentRunRow>(
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
            .fetch_one(e)
            .await
        )?;
        dual_tx!(
            &mut tx,
            |e| sqlx::query("INSERT INTO agent_outputs (id, agent_run_id, output) VALUES ($1, $2, $3)")
                .bind(Uuid::now_v7())
                .bind(run.id)
                .bind(output.to_string())
                .execute(e)
                .await
            .map(|r| r.rows_affected())
        )?;
        tx.commit().await?;
        Ok(run)
    }
}

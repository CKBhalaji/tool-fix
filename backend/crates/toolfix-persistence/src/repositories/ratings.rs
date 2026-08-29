//! Ratings/reviews repository.

use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::RatingRow;
use crate::{dual, Db};

#[derive(Clone)]
pub struct Ratings {
    db: Db,
}

impl Ratings {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        job_id: Uuid,
        mechanic_id: Uuid,
        customer_user_id: Uuid,
        score: i16,
        comment: Option<&str>,
    ) -> Result<RatingRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, RatingRow>(
                r#"
                INSERT INTO ratings (id, job_id, mechanic_id, customer_user_id, score, comment)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(job_id)
            .bind(mechanic_id)
            .bind(customer_user_id)
            .bind(score)
            .bind(comment)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn list_by_mechanic(
        &self,
        mechanic_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RatingRow>, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, RatingRow>(
                r#"
                SELECT r.* FROM ratings r
                  JOIN reviews v ON v.rating_id = r.id AND v.visible
                 WHERE r.mechanic_id = $1
                 ORDER BY r.created_at DESC
                 LIMIT $2
                "#,
            )
            .bind(mechanic_id)
            .bind(limit)
            .fetch_all(e)
            .await
        )?)
    }

    pub async fn find_for_job(&self, job_id: Uuid) -> Result<RatingRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, RatingRow>("SELECT * FROM ratings WHERE job_id = $1")
                .bind(job_id)
                .fetch_one(e)
                .await
        )?)
    }

    /// Average score used to refresh `mechanics.rating_average` (computed in
    /// Rust so the SQL stays portable).
    pub async fn average_for_mechanic(
        &self,
        mechanic_id: Uuid,
    ) -> Result<Option<f32>, PersistenceError> {
        let row: (Option<f64>,) = dual!(
            &self.db,
            |e| sqlx::query_as("SELECT AVG(score) FROM ratings WHERE mechanic_id = $1")
                .bind(mechanic_id)
                .fetch_one(e)
                .await
        )?;
        Ok(row.0.map(|v| v as f32))
    }
}

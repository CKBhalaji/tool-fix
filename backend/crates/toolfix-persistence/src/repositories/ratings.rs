//! Ratings/reviews repository.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::RatingRow;

#[derive(Clone)]
pub struct Ratings {
    pool: PgPool,
}

impl Ratings {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        job_id: Uuid,
        mechanic_id: Uuid,
        customer_user_id: Uuid,
        score: i16,
        comment: Option<&str>,
    ) -> Result<RatingRow, PersistenceError> {
        sqlx::query_as::<_, RatingRow>(
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
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn list_by_mechanic(
        &self,
        mechanic_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RatingRow>, PersistenceError> {
        sqlx::query_as::<_, RatingRow>(
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
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn find_for_job(&self, job_id: Uuid) -> Result<RatingRow, PersistenceError> {
        sqlx::query_as::<_, RatingRow>("SELECT * FROM ratings WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }
}

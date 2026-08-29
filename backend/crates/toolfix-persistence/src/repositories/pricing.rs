//! Pricing persistence: advisory estimates and realized price history.

use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::{PriceEstimateRow, PriceHistoryStatsRow};
use crate::{dual, Db};
use toolfix_contracts::RepairCategory;

#[derive(Clone)]
pub struct Pricing {
    db: Db,
}

impl Pricing {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn record_estimate(
        &self,
        breakdown_id: Uuid,
        job_id: Option<Uuid>,
        repair_category: RepairCategory,
        min_minor: i64,
        max_minor: i64,
        source: &str,
        notes: Option<&str>,
    ) -> Result<PriceEstimateRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PriceEstimateRow>(
                r#"
                INSERT INTO price_estimates (id, breakdown_id, job_id, repair_category,
                                             estimated_cost_min_minor, estimated_cost_max_minor,
                                             source, notes)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(breakdown_id)
            .bind(job_id)
            .bind(repair_category.as_str())
            .bind(min_minor)
            .bind(max_minor)
            .bind(source)
            .bind(notes)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn latest_estimate_for_breakdown(
        &self,
        breakdown_id: Uuid,
    ) -> Result<PriceEstimateRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PriceEstimateRow>(
                r#"
                SELECT * FROM price_estimates WHERE breakdown_id = $1
                 ORDER BY created_at DESC LIMIT 1
                "#,
            )
            .bind(breakdown_id)
            .fetch_one(e)
            .await
        )?)
    }

    /// Historical stats per repair category and vehicle kind — the learning
    /// data the pricing agent conditions on.
    pub async fn historical_stats(
        &self,
        repair_category: RepairCategory,
        vehicle_kind: Option<&str>,
        city: Option<&str>,
    ) -> Result<PriceHistoryStatsRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, PriceHistoryStatsRow>(
                r#"
                SELECT COUNT(*) AS sample_count,
                       AVG(final_amount_minor) AS avg_amount_minor,
                       MIN(final_amount_minor) AS min_amount_minor,
                       MAX(final_amount_minor) AS max_amount_minor
                  FROM price_history
                 WHERE repair_category = $1
                   AND ($2 IS NULL OR vehicle_kind = $2)
                   AND ($3 IS NULL OR city = $3)
                "#,
            )
            .bind(repair_category.as_str())
            .bind(vehicle_kind)
            .bind(city)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn record_final_price(
        &self,
        job_id: Uuid,
        repair_category: RepairCategory,
        vehicle_kind: Option<&str>,
        city: Option<&str>,
        final_amount_minor: i64,
    ) -> Result<(), PersistenceError> {
        dual!(
            &self.db,
            |e| sqlx::query(
                r#"
                INSERT INTO price_history (id, job_id, repair_category, vehicle_kind, city, final_amount_minor)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(job_id)
            .bind(repair_category.as_str())
            .bind(vehicle_kind)
            .bind(city)
            .bind(final_amount_minor)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }
}

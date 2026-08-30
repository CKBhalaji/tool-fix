//! Mechanics repository, including the geo candidate query used by matching.
//!
//! Candidate search is a bounding-box + capability pre-filter; exact
//! haversine distance is computed in Rust (`toolfix_location`) so the SQL
//! stays dialect-neutral (SQLite has no trig functions).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::MechanicRow;
use crate::{dual, dual_tx, Db};
use toolfix_contracts::{AvailabilityStatus, RepairCategory, VehicleKind};

#[derive(Clone)]
pub struct Mechanics {
    db: Db,
}

impl Mechanics {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<MechanicRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>("SELECT * FROM mechanics WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(e)
                .await
        )?)
    }

    /// Resolves the login user behind a mechanic profile.
    pub async fn user_id_of(&self, mechanic_id: Uuid) -> Result<Uuid, PersistenceError> {
        let row: (Uuid,) = dual!(
            &self.db,
            |e| sqlx::query_as("SELECT user_id FROM mechanics WHERE id = $1")
                .bind(mechanic_id)
                .fetch_one(e)
                .await
        )?;
        Ok(row.0)
    }

    /// Mechanic profiles are created lazily on first onboarding.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_for_user(
        &self,
        user_id: Uuid,
        display_name: Option<&str>,
        phone: Option<&str>,
        city: Option<&str>,
        service_area_km: f64,
        supported_vehicle_kinds: &[VehicleKind],
        repair_categories: &[RepairCategory],
        experience_years: Option<i16>,
    ) -> Result<MechanicRow, PersistenceError> {
        let kinds = crate::list_to_json(
            &supported_vehicle_kinds
                .iter()
                .map(|k| k.as_str().to_string())
                .collect::<Vec<String>>(),
        );
        let categories = crate::list_to_json(
            &repair_categories
                .iter()
                .map(|c| c.as_str().to_string())
                .collect::<Vec<String>>(),
        );
        let now = Utc::now();
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>(
                r#"
                INSERT INTO mechanics (id, user_id, display_name, phone, city, service_area_km,
                                       supported_vehicle_kinds, repair_categories, experience_years, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
                ON CONFLICT (user_id) DO UPDATE
                    SET display_name = COALESCE(EXCLUDED.display_name, mechanics.display_name),
                        phone = COALESCE(EXCLUDED.phone, mechanics.phone),
                        city = COALESCE(EXCLUDED.city, mechanics.city),
                        service_area_km = COALESCE(EXCLUDED.service_area_km, mechanics.service_area_km),
                        supported_vehicle_kinds = EXCLUDED.supported_vehicle_kinds,
                        repair_categories = EXCLUDED.repair_categories,
                        experience_years = COALESCE(EXCLUDED.experience_years, mechanics.experience_years),
                        updated_at = $10
                RETURNING *
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(user_id)
            .bind(display_name)
            .bind(phone)
            .bind(city)
            .bind(service_area_km as f32)
            .bind(kinds)
            .bind(categories)
            .bind(experience_years)
            .bind(now)
            .fetch_one(e)
            .await
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_profile(
        &self,
        mechanic_id: Uuid,
        display_name: Option<&str>,
        city: Option<&str>,
        service_area_km: Option<f64>,
        supported_vehicle_kinds: Option<&[VehicleKind]>,
        repair_categories: Option<&[RepairCategory]>,
        experience_years: Option<i16>,
    ) -> Result<MechanicRow, PersistenceError> {
        let kinds = supported_vehicle_kinds.map(|ks| {
            crate::list_to_json(&ks.iter().map(|k| k.as_str().to_string()).collect::<Vec<String>>())
        });
        let categories = repair_categories.map(|cs| {
            crate::list_to_json(&cs.iter().map(|c| c.as_str().to_string()).collect::<Vec<String>>())
        });
        let now = Utc::now();
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>(
                r#"
                UPDATE mechanics
                   SET display_name = COALESCE($2, display_name),
                       city = COALESCE($3, city),
                       service_area_km = COALESCE($4, service_area_km),
                       supported_vehicle_kinds = COALESCE($5, supported_vehicle_kinds),
                       repair_categories = COALESCE($6, repair_categories),
                       experience_years = COALESCE($7, experience_years),
                       updated_at = $8
                 WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(mechanic_id)
            .bind(display_name)
            .bind(city)
            .bind(service_area_km.map(|v| v as f32))
            .bind(kinds)
            .bind(categories)
            .bind(experience_years)
            .bind(now)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn set_availability(
        &self,
        mechanic_id: Uuid,
        status: AvailabilityStatus,
    ) -> Result<MechanicRow, PersistenceError> {
        let now = Utc::now();
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>(
                "UPDATE mechanics SET availability_status = $2, updated_at = $3 WHERE id = $1 RETURNING *"
            )
            .bind(mechanic_id)
            .bind(status.as_str())
            .bind(now)
            .fetch_one(e)
            .await
        )?)
    }

    /// Updates current position (fast-path columns + mechanic_locations).
    pub async fn update_location(
        &self,
        mechanic_id: Uuid,
        latitude: f64,
        longitude: f64,
        accuracy_m: Option<f64>,
        now: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        let mut tx = self.db.begin().await?;
        dual_tx!(
            &mut tx,
            |e| sqlx::query(
                r#"
                UPDATE mechanics
                   SET current_latitude = $2, current_longitude = $3, location_updated_at = $4,
                       updated_at = $4
                 WHERE id = $1
                "#,
            )
            .bind(mechanic_id)
            .bind(latitude)
            .bind(longitude)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        dual_tx!(
            &mut tx,
            |e| sqlx::query(
                r#"
                INSERT INTO mechanic_locations (mechanic_id, latitude, longitude, accuracy_m, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (mechanic_id) DO UPDATE
                    SET latitude = EXCLUDED.latitude,
                        longitude = EXCLUDED.longitude,
                        accuracy_m = EXCLUDED.accuracy_m,
                        updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(mechanic_id)
            .bind(latitude)
            .bind(longitude)
            .bind(accuracy_m)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        tx.commit().await?;
        Ok(())
    }

    /// Verified, available mechanics with a known position inside a
    /// bounding box. Exact distance ranking happens in Rust.
    pub async fn find_online_in_box(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lng: f64,
        max_lng: f64,
        limit: i64,
    ) -> Result<Vec<MechanicRow>, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>(
                r#"
                SELECT m.*
                  FROM mechanics m
                 WHERE m.availability_status IN ('online', 'busy')
                   AND m.is_verified
                   AND m.current_latitude IS NOT NULL
                   AND m.current_longitude IS NOT NULL
                   AND m.current_latitude BETWEEN $1 AND $2
                   AND m.current_longitude BETWEEN $3 AND $4
                 LIMIT $5
                "#,
            )
            .bind(min_lat)
            .bind(max_lat)
            .bind(min_lng)
            .bind(max_lng)
            .bind(limit)
            .fetch_all(e)
            .await
        )?)
    }

    /// Verification is an admin/operator decision; matching only considers
    /// verified mechanics.
    pub async fn set_verified(
        &self,
        mechanic_id: Uuid,
        verified: bool,
    ) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query("UPDATE mechanics SET is_verified = $2, updated_at = $3 WHERE id = $1")
                .bind(mechanic_id)
                .bind(verified)
                .bind(now)
                .execute(e)
                .await
                .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    pub async fn count_active_jobs(&self, mechanic_id: Uuid) -> Result<i64, PersistenceError> {
        let row: (i64,) = dual!(
            &self.db,
            |e| sqlx::query_as(
                r#"
                SELECT COUNT(*)
                  FROM assistance_jobs
                 WHERE selected_mechanic_id = $1
                   AND status IN ('mechanic_selected', 'mechanic_en_route', 'mechanic_arrived',
                                  'repair_in_progress')
                "#,
            )
            .bind(mechanic_id)
            .fetch_one(e)
            .await
        )?;
        Ok(row.0)
    }

    pub async fn bump_completed_jobs(&self, mechanic_id: Uuid) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query(
                "UPDATE mechanics SET completed_jobs = completed_jobs + 1, updated_at = $2 WHERE id = $1"
            )
            .bind(mechanic_id)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }

    /// Rating refresh with the computed average supplied by the caller
    /// (portable across dialects).
    pub async fn store_rating_average(
        &self,
        mechanic_id: Uuid,
        average: Option<f32>,
    ) -> Result<(), PersistenceError> {
        let now = Utc::now();
        dual!(
            &self.db,
            |e| sqlx::query(
                "UPDATE mechanics SET rating_average = $2, updated_at = $3 WHERE id = $1"
            )
            .bind(mechanic_id)
            .bind(average)
            .bind(now)
            .execute(e)
            .await
            .map(|r| r.rows_affected())
        )?;
        Ok(())
    }
}

impl Mechanics {
    /// Admin listing of every mechanic profile.
    pub async fn list_all(&self, limit: i64) -> Result<Vec<MechanicRow>, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, MechanicRow>(
                "SELECT * FROM mechanics ORDER BY created_at DESC LIMIT $1"
            )
            .bind(limit)
            .fetch_all(e)
            .await
        )?)
    }
}

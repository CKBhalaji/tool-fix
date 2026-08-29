//! Mechanics repository, including the geo candidate query used by matching.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use toolfix_contracts::{AvailabilityStatus, RepairCategory, VehicleKind};
use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::{MechanicCandidateRow, MechanicRow};

#[derive(Clone)]
pub struct Mechanics {
    pool: PgPool,
}

impl Mechanics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<MechanicRow, PersistenceError> {
        sqlx::query_as::<_, MechanicRow>("SELECT * FROM mechanics WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::NotFound)
    }

    /// Resolves the login user behind a mechanic profile.
    pub async fn user_id_of(&self, mechanic_id: Uuid) -> Result<Uuid, PersistenceError> {
        let (user_id,): (Uuid,) =
            sqlx::query_as("SELECT user_id FROM mechanics WHERE id = $1")
                .bind(mechanic_id)
                .fetch_optional(&self.pool)
                .await?
                .ok_or(PersistenceError::NotFound)?;
        Ok(user_id)
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
        let kinds: Vec<String> = supported_vehicle_kinds.iter().map(|k| k.as_str().to_string()).collect();
        let categories: Vec<String> = repair_categories.iter().map(|c| c.as_str().to_string()).collect();
        sqlx::query_as::<_, MechanicRow>(
            r#"
            INSERT INTO mechanics (id, user_id, display_name, phone, city, service_area_km,
                                   supported_vehicle_kinds, repair_categories, experience_years)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id) DO UPDATE
                SET display_name = COALESCE(EXCLUDED.display_name, mechanics.display_name),
                    phone = COALESCE(EXCLUDED.phone, mechanics.phone),
                    city = COALESCE(EXCLUDED.city, mechanics.city),
                    service_area_km = COALESCE(EXCLUDED.service_area_km, mechanics.service_area_km),
                    supported_vehicle_kinds = EXCLUDED.supported_vehicle_kinds,
                    repair_categories = EXCLUDED.repair_categories,
                    experience_years = COALESCE(EXCLUDED.experience_years, mechanics.experience_years),
                    updated_at = now()
            RETURNING *
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(display_name)
        .bind(phone)
        .bind(city)
        .bind(service_area_km as f32)
        .bind(&kinds)
        .bind(&categories)
        .bind(experience_years)
        .fetch_one(&self.pool)
        .await.map_err(crate::PersistenceError::from)
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
        let kinds = supported_vehicle_kinds
            .map(|ks| ks.iter().map(|k| k.as_str().to_string()).collect::<Vec<String>>());
        let categories = repair_categories
            .map(|cs| cs.iter().map(|c| c.as_str().to_string()).collect::<Vec<String>>());
        sqlx::query_as::<_, MechanicRow>(
            r#"
            UPDATE mechanics
               SET display_name = COALESCE($2, display_name),
                   city = COALESCE($3, city),
                   service_area_km = COALESCE($4, service_area_km),
                   supported_vehicle_kinds = COALESCE($5, supported_vehicle_kinds),
                   repair_categories = COALESCE($6, repair_categories),
                   experience_years = COALESCE($7, experience_years),
                   updated_at = now()
             WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(mechanic_id)
        .bind(display_name)
        .bind(city)
        .bind(service_area_km.map(|v| v as f32))
        .bind(kinds.as_ref())
        .bind(categories.as_ref())
        .bind(experience_years)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::NotFound)
    }

    pub async fn set_availability(
        &self,
        mechanic_id: Uuid,
        status: AvailabilityStatus,
    ) -> Result<MechanicRow, PersistenceError> {
        sqlx::query_as::<_, MechanicRow>(
            "UPDATE mechanics SET availability_status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(mechanic_id)
        .bind(status.as_str())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::NotFound)
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
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            UPDATE mechanics
               SET current_latitude = $2, current_longitude = $3, location_updated_at = $4,
                   updated_at = now()
             WHERE id = $1
            "#,
        )
        .bind(mechanic_id)
        .bind(latitude)
        .bind(longitude)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
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
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Nearby online mechanics inside a bounding box (exact distance
    /// returned for Rust-side ranking). Candidates are not yet filtered on
    /// vehicle/repair capability — matching does that.
    #[allow(clippy::too_many_arguments)]
    pub async fn find_candidates_in_box(
        &self,
        center_lat: f64,
        center_lng: f64,
        min_lat: f64,
        max_lat: f64,
        min_lng: f64,
        max_lng: f64,
        max_radius_m: f64,
        limit: i64,
    ) -> Result<Vec<MechanicCandidateRow>, PersistenceError> {
        // Haversine in SQL; bounding box pre-filter keeps the scan small.
        sqlx::query_as::<_, MechanicCandidateRow>(
            r#"
            SELECT m.*,
                   6371000.0 * acos(least(1.0,
                       cos(radians($1)) * cos(radians(m.current_latitude)) *
                       cos(radians(m.current_longitude) - radians($2)) +
                       sin(radians($1)) * sin(radians(m.current_latitude)))) AS distance_m
              FROM mechanics m
             WHERE m.availability_status IN ('online', 'busy')
               AND m.is_verified
               AND m.current_latitude IS NOT NULL
               AND m.current_longitude IS NOT NULL
               AND m.current_latitude BETWEEN $3 AND $4
               AND m.current_longitude BETWEEN $5 AND $6
               AND 6371000.0 * acos(least(1.0,
                       cos(radians($1)) * cos(radians(m.current_latitude)) *
                       cos(radians(m.current_longitude) - radians($2)) +
                       sin(radians($1)) * sin(radians(m.current_latitude)))) <= $7
             ORDER BY distance_m
             LIMIT $8
            "#,
        )
        .bind(center_lat)
        .bind(center_lng)
        .bind(min_lat)
        .bind(max_lat)
        .bind(min_lng)
        .bind(max_lng)
        .bind(max_radius_m)
        .bind(limit)
        .fetch_all(&self.pool)
        .await.map_err(crate::PersistenceError::from)
    }

    pub async fn count_active_jobs(&self, mechanic_id: Uuid) -> Result<i64, PersistenceError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
              FROM assistance_jobs
             WHERE selected_mechanic_id = $1
               AND status IN ('mechanic_selected', 'mechanic_en_route', 'mechanic_arrived',
                              'repair_in_progress')
            "#,
        )
        .bind(mechanic_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    pub async fn bump_completed_jobs(&self, mechanic_id: Uuid) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE mechanics SET completed_jobs = completed_jobs + 1, updated_at = now() WHERE id = $1")
            .bind(mechanic_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Recomputes the cached rating average after a new rating.
    pub async fn refresh_rating(&self, mechanic_id: Uuid) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            UPDATE mechanics
               SET rating_average = sub.avg_score, updated_at = now()
              FROM (SELECT AVG(score)::REAL AS avg_score FROM ratings WHERE mechanic_id = $1) sub
             WHERE id = $1
            "#,
        )
        .bind(mechanic_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

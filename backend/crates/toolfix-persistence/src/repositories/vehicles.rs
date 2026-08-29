//! Vehicles repository.

use uuid::Uuid;

use crate::error::PersistenceError;
use crate::models::VehicleRow;
use crate::{dual, Db};
use toolfix_contracts::VehicleKind;

#[derive(Clone)]
pub struct Vehicles {
    db: Db,
}

impl Vehicles {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        owner_user_id: Uuid,
        vehicle_kind: VehicleKind,
        make: Option<&str>,
        model: Option<&str>,
        year: Option<i16>,
        registration_number: Option<&str>,
    ) -> Result<VehicleRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, VehicleRow>(
                r#"
                INSERT INTO vehicles (id, owner_user_id, vehicle_kind, make, model, year, registration_number)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING *
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(owner_user_id)
            .bind(vehicle_kind.as_str())
            .bind(make)
            .bind(model)
            .bind(year)
            .bind(registration_number)
            .fetch_one(e)
            .await
        )?)
    }

    pub async fn list_by_owner(
        &self,
        owner_user_id: Uuid,
    ) -> Result<Vec<VehicleRow>, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, VehicleRow>(
                "SELECT * FROM vehicles WHERE owner_user_id = $1 ORDER BY created_at DESC"
            )
            .bind(owner_user_id)
            .fetch_all(e)
            .await
        )?)
    }

    pub async fn find_by_id(&self, vehicle_id: Uuid) -> Result<VehicleRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, VehicleRow>("SELECT * FROM vehicles WHERE id = $1")
                .bind(vehicle_id)
                .fetch_one(e)
                .await
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        vehicle_id: Uuid,
        owner_user_id: Uuid,
        vehicle_kind: Option<VehicleKind>,
        make: Option<&str>,
        model: Option<&str>,
        year: Option<i16>,
        registration_number: Option<&str>,
    ) -> Result<VehicleRow, PersistenceError> {
        Ok(dual!(
            &self.db,
            |e| sqlx::query_as::<_, VehicleRow>(
                r#"
                UPDATE vehicles
                   SET vehicle_kind = COALESCE($3, vehicle_kind),
                       make = COALESCE($4, make),
                       model = COALESCE($5, model),
                       year = COALESCE($6, year),
                       registration_number = COALESCE($7, registration_number)
                 WHERE id = $1 AND owner_user_id = $2
                RETURNING *
                "#,
            )
            .bind(vehicle_id)
            .bind(owner_user_id)
            .bind(vehicle_kind.map(|k| k.as_str().to_string()))
            .bind(make)
            .bind(model)
            .bind(year)
            .bind(registration_number)
            .fetch_one(e)
            .await
        )?)
    }

    /// Deletes only if the vehicle belongs to the owner; returns whether a
    /// row was removed.
    pub async fn delete_owned(
        &self,
        vehicle_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<bool, PersistenceError> {
        let result = dual!(
            &self.db,
            |e| sqlx::query("DELETE FROM vehicles WHERE id = $1 AND owner_user_id = $2")
                .bind(vehicle_id)
                .bind(owner_user_id)
                .execute(e)
                .await
            .map(|r| r.rows_affected())
        )?;
        Ok(result > 0)
    }
}

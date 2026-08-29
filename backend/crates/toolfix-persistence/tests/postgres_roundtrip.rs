//! Persistence round-trip tests against a real PostgreSQL.
//!
//! Ignored by default (like CI-only integration tests). Run with:
//! ```text
//! TOOLFIX_TEST_DATABASE_URL=postgres://toolfix:toolfix@localhost:5432/toolfix \
//!   cargo test -p toolfix-persistence --test postgres_roundtrip -- --ignored
//! ```

use toolfix_contracts::{Currency, JobStatus, MediaKind, PaymentMethod, VehicleKind};
use toolfix_persistence::Repositories;
use toolfix_persistence::{connect, run_migrations};

async fn test_pool() -> sqlx::PgPool {
    let url = std::env::var("TOOLFIX_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("set TOOLFIX_TEST_DATABASE_URL to run this test");
    connect(&url).await.expect("connect + migrate")
}

#[tokio::test]
#[ignore = "requires a PostgreSQL database (TOOLFIX_TEST_DATABASE_URL)"]
async fn users_vehicles_breakdowns_jobs_roundtrip() {
    let pool = test_pool().await;
    run_migrations(&pool).await.expect("migrations idempotent");
    let repos = Repositories::new(pool.clone());

    let user = repos
        .users
        .upsert_by_external_identity(
            &format!("test-{}", uuid::Uuid::now_v7()),
            Some("roundtrip@example.com"),
            Some("Round Trip"),
            None,
        )
        .await
        .expect("user upsert");
    assert_eq!(user.role().unwrap(), toolfix_contracts::UserRole::Customer);

    let vehicle = repos
        .vehicles
        .create(user.id, VehicleKind::Motorcycle, Some("Honda"), Some("Activa"), Some(2022), None)
        .await
        .expect("vehicle insert");
    assert_eq!(vehicle.kind().unwrap(), VehicleKind::Motorcycle);

    let breakdown = repos
        .breakdowns
        .create(
            user.id,
            vehicle.id,
            12.9716,
            77.5946,
            Some("Near MG Road"),
            "Bike stopped suddenly and will not start",
            &["no crank".to_string()],
        )
        .await
        .expect("breakdown insert");

    repos
        .breakdowns
        .add_media(breakdown.id, MediaKind::Photo, "breakdowns/test/photo.jpg", Some("image/jpeg"), Some(4))
        .await
        .expect("media insert");
    assert_eq!(repos.breakdowns.list_media(breakdown.id).await.unwrap().len(), 1);

    let job = repos
        .jobs
        .create(breakdown.id, user.id, vehicle.id)
        .await
        .expect("job insert");
    assert_eq!(job.status().unwrap(), JobStatus::Created);

    // State-machine transition with history.
    let mut tx = pool.begin().await.unwrap();
    let analyzing = repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::Created, JobStatus::Analyzing, Some(user.id), Some("test"))
        .await
        .expect("transition");
    tx.commit().await.unwrap();
    assert_eq!(analyzing.status().unwrap(), JobStatus::Analyzing);
    assert_eq!(repos.jobs.status_history(job.id, 10).await.unwrap().len(), 1);

    // Illegal transition must be rejected by the Rust state-machine guard.
    let mut tx = pool.begin().await.unwrap();
    assert!(repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::Analyzing, JobStatus::Completed, None, None)
        .await
        .is_err());
    tx.rollback().await.unwrap();

    let _ = Currency::Inr;
}

#[tokio::test]
#[ignore = "requires a PostgreSQL database (TOOLFIX_TEST_DATABASE_URL)"]
async fn offers_acceptance_transaction_roundtrip() {
    let pool = test_pool().await;
    let repos = Repositories::new(pool.clone());

    let customer = repos
        .users
        .upsert_by_external_identity(&format!("cust-{}", uuid::Uuid::now_v7()), None, None, None)
        .await
        .unwrap();
    let mechanic_user_a = repos
        .users
        .upsert_by_external_identity(&format!("mecha-{}", uuid::Uuid::now_v7()), None, None, None)
        .await
        .unwrap();
    let mechanic_user_b = repos
        .users
        .upsert_by_external_identity(&format!("mechb-{}", uuid::Uuid::now_v7()), None, None, None)
        .await
        .unwrap();

    repos.users.set_role(mechanic_user_a.id, toolfix_contracts::UserRole::Mechanic).await.unwrap();
    repos.users.set_role(mechanic_user_b.id, toolfix_contracts::UserRole::Mechanic).await.unwrap();

    let mech_a = repos
        .mechanics
        .upsert_for_user(
            mechanic_user_a.id,
            Some("A"),
            None,
            Some("Bengaluru"),
            10.0,
            &[VehicleKind::Scooter],
            &[toolfix_contracts::RepairCategory::Battery],
            None,
        )
        .await
        .unwrap();
    let mech_b = repos
        .mechanics
        .upsert_for_user(
            mechanic_user_b.id,
            Some("B"),
            None,
            Some("Bengaluru"),
            10.0,
            &[VehicleKind::Scooter],
            &[toolfix_contracts::RepairCategory::Battery],
            None,
        )
        .await
        .unwrap();
    repos.mechanics.set_availability(mech_a.id, toolfix_contracts::AvailabilityStatus::Online).await.unwrap();
    repos.mechanics.set_availability(mech_b.id, toolfix_contracts::AvailabilityStatus::Online).await.unwrap();

    let vehicle = repos
        .vehicles
        .create(customer.id, VehicleKind::Scooter, None, None, None, None)
        .await
        .unwrap();
    let breakdown = repos
        .breakdowns
        .create(
            customer.id,
            vehicle.id,
            12.9716,
            77.5946,
            None,
            "Scooter will not start after rain",
            &[],
        )
        .await
        .unwrap();
    repos
        .jobs
        .set_offer_window(repos.jobs.create(breakdown.id, customer.id, vehicle.id).await.unwrap().id, None)
        .await
        .unwrap();
    let job = repos.jobs.find_by_breakdown(breakdown.id).await.unwrap();

    // Advance to mechanics_notified.
    let mut tx = pool.begin().await.unwrap();
    repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::Created, JobStatus::Analyzing, None, None)
        .await
        .unwrap();
    repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::Analyzing, JobStatus::MechanicsSearching, None, None)
        .await
        .unwrap();
    repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::MechanicsSearching, JobStatus::MechanicsNotified, None, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let deadline = chrono::Utc::now() + chrono::Duration::minutes(10);
    let offer_a = repos
        .offers
        .create(job.id, mech_a.id, 65_000, 15, Some("quick"), deadline)
        .await
        .unwrap();
    repos
        .offers
        .create(job.id, mech_b.id, 50_000, 30, None, deadline)
        .await
        .unwrap();
    {
        let mut tx = pool.begin().await.unwrap();
        repos
            .jobs
            .transition_job(&mut tx, job.id, JobStatus::MechanicsNotified, JobStatus::OffersReceived, None, None)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    // Accept offer A: B must be expired atomically.
    let mut tx = pool.begin().await.unwrap();
    let (updated, expired) = repos
        .offers
        .accept_offer_transaction(&mut tx, offer_a.id, job.id, customer.id, JobStatus::OffersReceived, JobStatus::MechanicSelected)
        .await
        .expect("acceptance");
    tx.commit().await.unwrap();
    assert_eq!(updated.status().unwrap(), JobStatus::MechanicSelected);
    assert_eq!(expired, 1);
    assert_eq!(updated.selected_mechanic_id, Some(mech_a.id));

    let _ = PaymentMethod::Cash;
}

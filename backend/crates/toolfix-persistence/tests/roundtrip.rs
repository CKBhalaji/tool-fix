//! Persistence round-trip tests.
//!
//! - `sqlite_roundtrip` runs everywhere (the database file is created in a
//!   temp directory; no services required).
//! - `postgres_roundtrip` is ignored by default and runs in CI against a
//!   PostgreSQL service:
//! ```text
//! TOOLFIX_TEST_DATABASE_URL=postgres://toolfix:toolfix@localhost:5432/toolfix \
//!   cargo test -p toolfix-persistence --test roundtrip -- --ignored
//! ```

use toolfix_contracts::{JobStatus, MediaKind, VehicleKind};
use toolfix_persistence::Repositories;
use toolfix_persistence::{connect_with_driver, DbDriver};

async fn make_repos(driver: DbDriver, url: &str) -> Repositories {
    let db = connect_with_driver(driver, url).await.expect("connect + migrate");
    Repositories::new(db)
}

/// Shared scenario: user → vehicle → breakdown (+media) → job → transitions
/// → two mechanic offers → transactional acceptance.
async fn marketplace_roundtrip(repos: &Repositories) {
    let customer = repos
        .users
        .upsert_by_external_identity(
            &format!("cust-{}", uuid::Uuid::now_v7()),
            Some("roundtrip@example.com"),
            Some("Round Trip"),
            None,
        )
        .await
        .expect("customer upsert");
    assert_eq!(customer.role().unwrap(), toolfix_contracts::UserRole::Customer);

    let vehicle = repos
        .vehicles
        .create(
            customer.id,
            VehicleKind::Scooter,
            Some("Honda"),
            Some("Activa"),
            Some(2022),
            None,
        )
        .await
        .expect("vehicle insert");
    assert_eq!(vehicle.kind().unwrap(), VehicleKind::Scooter);

    let breakdown = repos
        .breakdowns
        .create(
            customer.id,
            vehicle.id,
            12.9716,
            77.5946,
            Some("Near MG Road"),
            "Scooter stopped suddenly and will not start",
            &["no crank".to_string(), "warning light".to_string()],
        )
        .await
        .expect("breakdown insert");
    assert_eq!(breakdown.symptoms().len(), 2);

    repos
        .breakdowns
        .add_media(
            breakdown.id,
            MediaKind::Photo,
            "breakdowns/test/photo.jpg",
            Some("image/jpeg"),
            Some(4),
        )
        .await
        .expect("media insert");
    assert_eq!(repos.breakdowns.list_media(breakdown.id).await.unwrap().len(), 1);

    let job = repos
        .jobs
        .create(breakdown.id, customer.id, vehicle.id)
        .await
        .expect("job insert");
    assert_eq!(job.status().unwrap(), JobStatus::Created);

    // Advance to mechanics_notified with history rows.
    let mut tx = repos.begin().await.expect("begin");
    for (from, to) in [
        (JobStatus::Created, JobStatus::Analyzing),
        (JobStatus::Analyzing, JobStatus::MechanicsSearching),
        (JobStatus::MechanicsSearching, JobStatus::MechanicsNotified),
    ] {
        repos
            .jobs
            .transition_job(&mut tx, job.id, from, to, Some(customer.id), Some("test"))
            .await
            .expect("legal transition");
    }
    tx.commit().await.expect("commit");
    assert_eq!(repos.jobs.status_history(job.id, 10).await.unwrap().len(), 3);

    // Optimistic locking: a stale expected-status must be rejected and
    // leave no history behind. (Legality of transitions itself is enforced
    // by the domain state machine inside the jobs service.)
    let mut tx = repos.begin().await.expect("begin");
    assert!(repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::Created, JobStatus::Completed, None, None)
        .await
        .is_err());
    tx.rollback().await.expect("rollback");
    assert_eq!(repos.jobs.status_history(job.id, 10).await.unwrap().len(), 3);

    // Two mechanics bid; accepting one atomically expires the other.
    for label in ["mecha", "mechb"] {
        let mechanic_user = repos
            .users
            .upsert_by_external_identity(&format!("{label}-{}", uuid::Uuid::now_v7()), None, None, None)
            .await
            .unwrap();
        repos
            .users
            .set_role(mechanic_user.id, toolfix_contracts::UserRole::Mechanic)
            .await
            .unwrap();
        let mechanic = repos
            .mechanics
            .upsert_for_user(
                mechanic_user.id,
                Some(label),
                None,
                Some("Bengaluru"),
                10.0,
                &[VehicleKind::Scooter],
                &[toolfix_contracts::RepairCategory::Battery],
                None,
            )
            .await
            .unwrap();
        repos
            .mechanics
            .set_availability(mechanic.id, toolfix_contracts::AvailabilityStatus::Online)
            .await
            .unwrap();
        repos.mechanics.set_verified(mechanic.id, true).await.unwrap();
        repos
            .mechanics
            .update_location(mechanic.id, 12.9720, 77.5950, Some(10.0), chrono::Utc::now())
            .await
            .unwrap();
    }

    let mechanics = repos
        .mechanics
        .find_online_in_box(12.9, 13.0, 77.5, 77.7, 10)
        .await
        .expect("candidates");
    assert_eq!(mechanics.len(), 2, "both online mechanics are candidates");

    let deadline = chrono::Utc::now() + chrono::Duration::minutes(10);
    let mut offer_ids = Vec::new();
    for mechanic in &mechanics {
        let offer = repos
            .offers
            .create(job.id, mechanic.id, 65_000, 15, Some("quick"), deadline)
            .await
            .expect("offer insert");
        offer_ids.push(offer.id);
    }
    // First offer moves the job into offers_received.
    let mut tx = repos.begin().await.expect("begin");
    repos
        .jobs
        .transition_job(&mut tx, job.id, JobStatus::MechanicsNotified, JobStatus::OffersReceived, None, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut tx = repos.begin().await.expect("begin");
    let (updated, expired) = repos
        .offers
        .accept_offer_transaction(
            &mut tx,
            offer_ids[0],
            job.id,
            customer.id,
            JobStatus::OffersReceived,
            JobStatus::MechanicSelected,
        )
        .await
        .expect("acceptance");
    tx.commit().await.expect("commit");

    assert_eq!(updated.status().unwrap(), JobStatus::MechanicSelected);
    assert_eq!(expired, 1, "exactly one competing offer expired");
    let statuses: Vec<String> = repos
        .offers
        .list_by_job(job.id)
        .await
        .unwrap()
        .into_iter()
        .map(|o| o.status)
        .collect();
    assert!(statuses.contains(&"accepted".to_string()));
    assert!(statuses.contains(&"expired".to_string()));
}

#[tokio::test]
async fn sqlite_marketplace_roundtrip() {
    let dir = std::env::temp_dir().join(format!("toolfix-test-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    let url = format!("sqlite://{}/dev.db?mode=rwc", dir.display());
    let repos = make_repos(DbDriver::Sqlite, &url).await;
    marketplace_roundtrip(&repos).await;
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
#[ignore = "requires a PostgreSQL database (TOOLFIX_TEST_DATABASE_URL)"]
async fn postgres_marketplace_roundtrip() {
    let url = std::env::var("TOOLFIX_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("set TOOLFIX_TEST_DATABASE_URL to run this test");
    let repos = make_repos(DbDriver::Postgres, &url).await;
    marketplace_roundtrip(&repos).await;
}

#[tokio::test]
async fn sqlite_database_file_is_created_automatically() {
    let dir = std::env::temp_dir().join(format!("toolfix-create-{}", uuid::Uuid::now_v7()));
    let db_path = dir.join("nested/dev.db");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    // The nested directory does not exist yet; connect must create both the
    // directory and the file.
    let repos = make_repos(DbDriver::Sqlite, &url).await;
    assert!(db_path.exists(), "sqlite file must be created at startup");
    repos.ping().await.expect("ping after creation");
    std::fs::remove_dir_all(&dir).ok();
}

//! Assistance job entity and lifecycle application.

use chrono::{DateTime, Utc};
use toolfix_contracts::{Currency, JobStatus};
use uuid::Uuid;

use crate::error::DomainError;
use crate::state_machine::validate_transition;

/// The lifecycle of one roadside assistance request. Persisted in
/// `assistance_jobs`; every transition is validated by the state machine.
#[derive(Debug, Clone)]
pub struct AssistanceJob {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub customer_user_id: Uuid,
    pub vehicle_id: Uuid,
    pub status: JobStatus,
    pub selected_mechanic_id: Option<Uuid>,
    pub final_amount_minor: Option<i64>,
    pub currency: Currency,
    pub offer_window_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AssistanceJob {
    /// Applies a status change, enforcing the state machine.
    pub fn transition_to(&mut self, to: JobStatus, now: DateTime<Utc>) -> Result<(), DomainError> {
        validate_transition(self.status, to).map_err(|e| DomainError::IllegalTransition {
            from: e.from,
            to: e.to,
        })?;
        self.status = to;
        self.updated_at = now;
        Ok(())
    }

    /// True while mechanics may submit new offers.
    pub fn accepting_offers(&self, now: DateTime<Utc>) -> bool {
        is_accepting_offers(self.status, self.offer_window_expires_at, now)
    }

    pub fn is_terminal(&self) -> bool {
        crate::state_machine::is_terminal(self.status)
    }
}

/// Free-function form so callers holding raw (status, window) values do not
/// need to reconstruct the whole entity.
pub fn is_accepting_offers(
    status: JobStatus,
    offer_window_expires_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let open_status =
        matches!(status, JobStatus::MechanicsNotified | JobStatus::OffersReceived);
    let window_open = offer_window_expires_at
        .map(|deadline| now < deadline)
        .unwrap_or(true);
    open_status && window_open
}

#[derive(Debug, Clone)]
pub struct NewJob {
    pub breakdown_id: Uuid,
    pub customer_user_id: Uuid,
    pub vehicle_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn job_with(status: JobStatus, window_expires: Option<DateTime<Utc>>) -> AssistanceJob {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        AssistanceJob {
            id: Uuid::now_v7(),
            breakdown_id: Uuid::now_v7(),
            customer_user_id: Uuid::now_v7(),
            vehicle_id: Uuid::now_v7(),
            status,
            selected_mechanic_id: None,
            final_amount_minor: None,
            currency: Currency::Inr,
            offer_window_expires_at: window_expires,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn accepting_offers_respects_window() {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let open = job_with(
            JobStatus::MechanicsNotified,
            Some(now + chrono::Duration::minutes(10)),
        );
        assert!(open.accepting_offers(now));

        let closed = job_with(
            JobStatus::OffersReceived,
            Some(now - chrono::Duration::minutes(1)),
        );
        assert!(!closed.accepting_offers(now));
    }

    #[test]
    fn transition_rejects_illegal_moves() {
        let mut job = job_with(JobStatus::Created, None);
        assert!(job.transition_to(JobStatus::Completed, Utc::now()).is_err());
        assert!(job.transition_to(JobStatus::Analyzing, Utc::now()).is_ok());
    }
}

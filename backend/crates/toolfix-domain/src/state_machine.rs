//! The assistance-job state machine.
//!
//! Every job status change must pass through [`validate_transition`].
//! Arbitrary client-supplied status strings are never accepted.

use std::fmt;

use toolfix_contracts::JobStatus;

/// Legal transitions of the assistance job lifecycle.
pub const TRANSITIONS: &[(JobStatus, &[JobStatus])] = &[
    (
        JobStatus::Created,
        &[
            JobStatus::Analyzing,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ],
    ),
    (
        JobStatus::Analyzing,
        &[
            JobStatus::MechanicsSearching,
            JobStatus::NoMechanicAvailable,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ],
    ),
    (
        JobStatus::MechanicsSearching,
        &[
            JobStatus::MechanicsNotified,
            JobStatus::NoMechanicAvailable,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ],
    ),
    (
        JobStatus::MechanicsNotified,
        &[
            JobStatus::OffersReceived,
            JobStatus::NoMechanicAvailable,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ],
    ),
    (
        JobStatus::OffersReceived,
        &[
            JobStatus::MechanicSelected,
            JobStatus::NoMechanicAvailable,
            JobStatus::Cancelled,
            JobStatus::Expired,
        ],
    ),
    (
        JobStatus::MechanicSelected,
        &[JobStatus::MechanicEnRoute, JobStatus::Cancelled],
    ),
    (
        JobStatus::MechanicEnRoute,
        &[JobStatus::MechanicArrived, JobStatus::Cancelled],
    ),
    (
        JobStatus::MechanicArrived,
        &[JobStatus::RepairInProgress, JobStatus::Cancelled],
    ),
    (
        JobStatus::RepairInProgress,
        &[
            JobStatus::RepairCompleted,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ],
    ),
    (
        JobStatus::RepairCompleted,
        &[JobStatus::PaymentPending],
    ),
    (JobStatus::PaymentPending, &[JobStatus::Completed]),
];

pub const TERMINAL_STATUSES: &[JobStatus] = &[
    JobStatus::Completed,
    JobStatus::Cancelled,
    JobStatus::Expired,
    JobStatus::Failed,
    JobStatus::NoMechanicAvailable,
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("illegal job transition {from} -> {to}")]
pub struct TransitionError {
    pub from: JobStatus,
    pub to: JobStatus,
}

pub fn legal_targets(from: JobStatus) -> &'static [JobStatus] {
    TRANSITIONS
        .iter()
        .find(|(status, _)| *status == from)
        .map(|(_, targets)| *targets)
        .unwrap_or(&[])
}

pub fn can_transition(from: JobStatus, to: JobStatus) -> bool {
    legal_targets(from).contains(&to)
}

pub fn validate_transition(from: JobStatus, to: JobStatus) -> Result<(), TransitionError> {
    if can_transition(from, to) {
        Ok(())
    } else {
        Err(TransitionError { from, to })
    }
}

pub fn is_terminal(status: JobStatus) -> bool {
    TERMINAL_STATUSES.contains(&status)
}

impl fmt::Display for TransitionErrorDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "illegal transition")
    }
}

/// Helper marker so callers can format transition errors without depending
/// on the error's private shape.
pub struct TransitionErrorDisplay<'a>(pub &'a TransitionError);

#[cfg(test)]
mod tests {
    use super::*;
    use toolfix_contracts::JobStatus;

    #[test]
    fn happy_path_is_legal_end_to_end() {
        let path = [
            JobStatus::Created,
            JobStatus::Analyzing,
            JobStatus::MechanicsSearching,
            JobStatus::MechanicsNotified,
            JobStatus::OffersReceived,
            JobStatus::MechanicSelected,
            JobStatus::MechanicEnRoute,
            JobStatus::MechanicArrived,
            JobStatus::RepairInProgress,
            JobStatus::RepairCompleted,
            JobStatus::PaymentPending,
            JobStatus::Completed,
        ];
        for pair in path.windows(2) {
            assert!(
                can_transition(pair[0], pair[1]),
                "{} -> {} must be legal",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn illegal_jumps_are_rejected() {
        assert!(validate_transition(JobStatus::Created, JobStatus::Completed).is_err());
        assert!(validate_transition(JobStatus::MechanicsNotified, JobStatus::RepairInProgress).is_err());
        assert!(validate_transition(JobStatus::PaymentPending, JobStatus::MechanicSelected).is_err());
    }

    #[test]
    fn terminal_states_have_no_outgoing_transitions() {
        for status in TERMINAL_STATUSES {
            assert!(
                legal_targets(*status).is_empty(),
                "{status} must be terminal"
            );
        }
    }

    #[test]
    fn cancellation_is_allowed_mid_lifecycle_but_not_after_completion() {
        assert!(can_transition(JobStatus::MechanicEnRoute, JobStatus::Cancelled));
        assert!(!can_transition(JobStatus::Completed, JobStatus::Cancelled));
    }
}

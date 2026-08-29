//! ToolFix business domain: entities and lifecycle rules.
//!
//! The domain depends only on `toolfix-contracts`. It must never know about
//! Axum, cookies, Firebase, PostgreSQL, or any other infrastructure.

pub mod assistance_job;
pub mod breakdown;
pub mod error;
pub mod location;
pub mod mechanic;
pub mod notification;
pub mod offer;
pub mod payment;
pub mod pricing;
pub mod rating;
pub mod state_machine;
pub mod user;
pub mod vehicle;

pub use error::DomainError;
pub use state_machine::{can_transition, validate_transition, TransitionError};

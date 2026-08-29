//! Application-level job orchestration. Sits between the HTTP layer and
//! the lower-level services; owns the lifecycle operations and the
//! transactional marketplace rules.

pub mod events;
pub mod service;

pub use events::EventHub;
pub use service::{JobService, JobsError};

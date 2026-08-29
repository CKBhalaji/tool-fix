//! Shared wire/domain primitive types for the ToolFix platform.
//!
//! This crate is the bottom of the dependency graph: it holds the shared
//! enums, identifiers, and request/response DTOs used by both the domain
//! layer and the HTTP API. It must not depend on any other `toolfix-*`
//! crate, nor on any database or HTTP framework.

pub mod breakdown;
pub mod enums;
pub mod error;
pub mod job;
pub mod location;
pub mod offer;
pub mod payment;
pub mod rating;
pub mod request;
pub mod response;
pub mod user;
pub mod vehicle;

pub use enums::*;
pub use error::ContractError;
pub use location::LatLng;

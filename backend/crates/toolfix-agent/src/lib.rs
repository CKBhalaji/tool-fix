//! The agentic layer.
//!
//! The AI is ADVISORY: it interprets the problem, classifies faults, and
//! recommends price ranges. Its output must pass schema validation and
//! business rules before anything is persisted, and it can never charge,
//! approve, or mutate marketplace state.

pub mod diagnosis;
pub mod error;
pub mod gemini_api;
pub mod nvidia;
pub mod pricing;
pub mod prompts;
pub mod provider;
pub mod schemas;
pub mod vertex;
pub mod workflow;

pub use error::AgentError;
pub use provider::{AgentProvider, DiagnosisInput, PriceEstimateInput};
pub use schemas::{DiagnosisOutput, PriceEstimateOutput};
pub use workflow::AgentWorkflow;

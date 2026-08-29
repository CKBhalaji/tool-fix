//! Router assembly and middleware.

pub mod error;
pub mod routes;
pub mod state;
pub mod ws;

pub mod handlers {
    pub mod agent;
    pub mod auth;
    pub mod breakdowns;
    pub mod jobs;
    pub mod mechanics;
    pub mod notifications;
    pub mod offers;
    pub mod payments;
    pub mod ratings;
    pub mod users;
    pub mod vehicles;
}

pub use error::ApiError;
pub use routes::router;
pub use state::AppState;

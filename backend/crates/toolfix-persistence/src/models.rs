//! Database row models. Statuses are stored as TEXT and converted to the
//! contract enums at this boundary.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use toolfix_contracts::{
    AvailabilityStatus, Currency, EstimateSource, JobStatus, MediaKind, NotificationChannel,
    NotificationKind, NotificationStatus, OfferStatus, OnboardingStatus, PaymentMethod,
    PaymentStatus, RepairCategory, Severity, UserRole, UserStatus, VehicleKind,
};
use uuid::Uuid;

use crate::error::PersistenceError;

macro_rules! db_status_helpers {
    ($fn_name:ident, $enum_ty:ty) => {
        fn $fn_name(value: &str) -> Result<$enum_ty, PersistenceError> {
            <$enum_ty>::parse(value).ok_or_else(|| {
                PersistenceError::InvalidData(format!("unknown {} value: {}", stringify!($enum_ty), value))
            })
        }
    };
}

db_status_helpers!(parse_job_status, JobStatus);
db_status_helpers!(parse_user_role, UserRole);
db_status_helpers!(parse_user_status, UserStatus);
db_status_helpers!(parse_onboarding_status, OnboardingStatus);
db_status_helpers!(parse_vehicle_kind, VehicleKind);
db_status_helpers!(parse_availability, AvailabilityStatus);
db_status_helpers!(parse_offer_status, OfferStatus);
db_status_helpers!(parse_media_kind, MediaKind);
db_status_helpers!(parse_severity, Severity);
db_status_helpers!(parse_repair_category, RepairCategory);
db_status_helpers!(parse_notification_kind, NotificationKind);
db_status_helpers!(parse_notification_channel, NotificationChannel);
db_status_helpers!(parse_notification_status, NotificationStatus);
db_status_helpers!(parse_payment_status, PaymentStatus);
db_status_helpers!(parse_payment_method, PaymentMethod);
db_status_helpers!(parse_estimate_source, EstimateSource);

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub status: String,
    pub onboarding_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRow {
    pub fn role(&self) -> Result<UserRole, PersistenceError> {
        parse_user_role(&self.role)
    }
    pub fn status(&self) -> Result<UserStatus, PersistenceError> {
        parse_user_status(&self.status)
    }
    pub fn onboarding_status(&self) -> Result<OnboardingStatus, PersistenceError> {
        parse_onboarding_status(&self.onboarding_status)
    }
}

#[derive(Debug, FromRow)]
pub struct AuthSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct VehicleRow {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub vehicle_kind: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i16>,
    pub registration_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl VehicleRow {
    pub fn kind(&self) -> Result<VehicleKind, PersistenceError> {
        parse_vehicle_kind(&self.vehicle_kind)
    }
}

#[derive(Debug, FromRow)]
pub struct MechanicRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub service_area_km: f32,
    pub supported_vehicle_kinds: Vec<String>,
    pub repair_categories: Vec<String>,
    pub experience_years: Option<i16>,
    pub availability_status: String,
    pub rating_average: Option<f32>,
    pub completed_jobs: i64,
    pub is_verified: bool,
    pub current_latitude: Option<f64>,
    pub current_longitude: Option<f64>,
    pub location_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MechanicRow {
    pub fn availability(&self) -> Result<AvailabilityStatus, PersistenceError> {
        parse_availability(&self.availability_status)
    }
    pub fn vehicle_kinds(&self) -> Result<Vec<VehicleKind>, PersistenceError> {
        self.supported_vehicle_kinds
            .iter()
            .map(|s| parse_vehicle_kind(s))
            .collect()
    }
    pub fn categories(&self) -> Result<Vec<RepairCategory>, PersistenceError> {
        self.repair_categories
            .iter()
            .map(|s| parse_repair_category(s))
            .collect()
    }
}

/// Mechanic row joined with distance from a search center.
#[derive(Debug, FromRow)]
pub struct MechanicCandidateRow {
    #[sqlx(flatten)]
    pub mechanic: MechanicRow,
    pub distance_m: f64,
}

#[derive(Debug, FromRow)]
pub struct BreakdownRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct BreakdownMediaRow {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub media_kind: String,
    pub storage_key: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl BreakdownMediaRow {
    pub fn kind(&self) -> Result<MediaKind, PersistenceError> {
        parse_media_kind(&self.media_kind)
    }
}

#[derive(Debug, FromRow)]
pub struct DiagnosisRow {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub agent_run_id: Option<Uuid>,
    pub possible_issue: String,
    pub confidence: f32,
    pub severity: String,
    pub recommended_service: String,
    pub repair_category: String,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    pub requires_towing: bool,
    pub reasoning_summary: String,
    pub created_at: DateTime<Utc>,
}

impl DiagnosisRow {
    pub fn severity(&self) -> Result<Severity, PersistenceError> {
        parse_severity(&self.severity)
    }
    pub fn category(&self) -> Result<RepairCategory, PersistenceError> {
        parse_repair_category(&self.repair_category)
    }
}

#[derive(Debug, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub customer_user_id: Uuid,
    pub vehicle_id: Uuid,
    pub status: String,
    pub selected_mechanic_id: Option<Uuid>,
    pub final_amount_minor: Option<i64>,
    pub currency: String,
    pub offer_window_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JobRow {
    pub fn status(&self) -> Result<JobStatus, PersistenceError> {
        parse_job_status(&self.status)
    }
    pub fn currency(&self) -> Result<Currency, PersistenceError> {
        match self.currency.as_str() {
            "INR" => Ok(Currency::Inr),
            other => Err(PersistenceError::InvalidData(format!(
                "unknown currency: {other}"
            ))),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct JobStatusHistoryRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub from_status: Option<String>,
    pub to_status: String,
    pub changed_by_user_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl JobStatusHistoryRow {
    pub fn from_status(&self) -> Result<Option<JobStatus>, PersistenceError> {
        self.from_status.as_deref().map(parse_job_status).transpose()
    }
    pub fn to_status(&self) -> Result<JobStatus, PersistenceError> {
        parse_job_status(&self.to_status)
    }
}

#[derive(Debug, FromRow)]
pub struct OfferRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub mechanic_id: Uuid,
    pub quoted_price_minor: i64,
    pub currency: String,
    pub estimated_arrival_minutes: i32,
    pub message: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // joined
    #[sqlx(default)]
    pub mechanic_name: Option<String>,
    #[sqlx(default)]
    pub mechanic_rating: Option<f32>,
    #[sqlx(default)]
    pub mechanic_completed_jobs: i64,
}

impl OfferRow {
    pub fn status(&self) -> Result<OfferStatus, PersistenceError> {
        parse_offer_status(&self.status)
    }
    pub fn currency(&self) -> Result<Currency, PersistenceError> {
        match self.currency.as_str() {
            "INR" => Ok(Currency::Inr),
            other => Err(PersistenceError::InvalidData(format!(
                "unknown currency: {other}"
            ))),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct NotificationRow {
    pub id: Uuid,
    pub recipient_user_id: Uuid,
    pub kind: String,
    pub channel: String,
    pub job_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub status: String,
    pub provider: Option<String>,
    pub delivery_detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

impl NotificationRow {
    pub fn kind(&self) -> Result<NotificationKind, PersistenceError> {
        parse_notification_kind(&self.kind)
    }
    pub fn channel(&self) -> Result<NotificationChannel, PersistenceError> {
        parse_notification_channel(&self.channel)
    }
    pub fn status(&self) -> Result<NotificationStatus, PersistenceError> {
        parse_notification_status(&self.status)
    }
}

#[derive(Debug, FromRow)]
pub struct PriceEstimateRow {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub job_id: Option<Uuid>,
    pub repair_category: String,
    pub estimated_cost_min_minor: i64,
    pub estimated_cost_max_minor: i64,
    pub currency: String,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PriceEstimateRow {
    pub fn category(&self) -> Result<RepairCategory, PersistenceError> {
        parse_repair_category(&self.repair_category)
    }
    pub fn source(&self) -> Result<EstimateSource, PersistenceError> {
        parse_estimate_source(&self.source)
    }
}

#[derive(Debug, FromRow)]
pub struct PriceHistoryStatsRow {
    pub sample_count: i64,
    pub avg_amount_minor: Option<f64>,
    pub min_amount_minor: Option<i64>,
    pub max_amount_minor: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct PaymentRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub amount_minor: i64,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub receipt_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

impl PaymentRow {
    pub fn method(&self) -> Result<PaymentMethod, PersistenceError> {
        parse_payment_method(&self.method)
    }
    pub fn status(&self) -> Result<PaymentStatus, PersistenceError> {
        parse_payment_status(&self.status)
    }
}

#[derive(Debug, FromRow)]
pub struct RatingRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub mechanic_id: Uuid,
    pub customer_user_id: Uuid,
    pub score: i16,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct AgentRunRow {
    pub id: Uuid,
    pub breakdown_id: Uuid,
    pub job_id: Option<Uuid>,
    pub kind: String,
    pub provider: String,
    pub model: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub latency_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct LatestLocationRow {
    pub latitude: f64,
    pub longitude: f64,
    pub recorded_at: DateTime<Utc>,
}

/// Joined job+breakdown row for the mechanic feed.
#[derive(Debug, FromRow)]
pub struct FeedJobRow {
    pub job_id: Uuid,
    pub job_status: String,
    pub offer_window_expires_at: Option<DateTime<Utc>>,
    pub breakdown_id: Uuid,
    pub vehicle_id: Uuid,
    pub problem_description: String,
    pub vehicle_symptoms: Vec<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub breakdown_created_at: DateTime<Utc>,
}

impl FeedJobRow {
    pub fn job_status(&self) -> Result<JobStatus, PersistenceError> {
        parse_job_status(&self.job_status)
    }
}

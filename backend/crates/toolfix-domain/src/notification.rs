//! Notification domain record.

use chrono::{DateTime, Utc};
use toolfix_contracts::{NotificationKind, NotificationStatus};
use uuid::Uuid;

/// A persisted notification intent. Delivery is attempted by a provider;
/// the row (not the provider call) is the durable record.
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_user_id: Uuid,
    pub kind: NotificationKind,
    pub channel: toolfix_contracts::NotificationChannel,
    pub job_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

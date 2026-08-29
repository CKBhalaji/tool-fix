//! Notification delivery abstraction.
//!
//! Business logic depends on the [`NotificationProvider`] trait only. The
//! initial provider logs deliveries (and they are always persisted by the
//! caller); FCM/SMS/email providers can be added later without redesign.

use async_trait::async_trait;
use toolfix_contracts::{NotificationChannel, NotificationKind};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DeliveryRequest {
    pub notification_id: Uuid,
    pub recipient_user_id: Uuid,
    pub kind: NotificationKind,
    pub channel: NotificationChannel,
    pub title: String,
    pub body: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub delivered: bool,
    pub detail: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("delivery failed: {0}")]
    Delivery(String),
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Human-readable provider name (used in delivery audit rows).
    fn name(&self) -> &'static str;

    /// Channels this provider can serve.
    fn channels(&self) -> &'static [NotificationChannel];

    async fn send(&self, request: DeliveryRequest) -> Result<DeliveryResult, NotificationError>;
}

/// Development provider: logs the notification and reports success.
pub struct LoggingProvider;

#[async_trait]
impl NotificationProvider for LoggingProvider {
    fn name(&self) -> &'static str {
        "logging"
    }

    fn channels(&self) -> &'static [NotificationChannel] {
        &[
            NotificationChannel::Push,
            NotificationChannel::Sms,
            NotificationChannel::Email,
            NotificationChannel::InApp,
        ]
    }

    async fn send(&self, request: DeliveryRequest) -> Result<DeliveryResult, NotificationError> {
        tracing::info!(
            notification_id = %request.notification_id,
            recipient = %request.recipient_user_id,
            kind = %request.kind,
            channel = %request.channel,
            title = %request.title,
            body = %request.body,
            "notification dispatched (logging provider)"
        );
        Ok(DeliveryResult {
            delivered: true,
            detail: "logged".to_string(),
        })
    }
}

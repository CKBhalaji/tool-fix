//! In-process job event hub (WebSocket fan-out).
//!
//! Transport only: PostgreSQL remains the durable source of truth, and a
//! restart drops subscribers — never state.

use std::sync::Arc;

use tokio::sync::broadcast;
use toolfix_contracts::job::JobEvent;

#[derive(Clone)]
pub struct EventHub {
    sender: Arc<broadcast::Sender<JobEvent>>,
}

impl Default for EventHub {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl EventHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(&self, job_id: uuid::Uuid, kind: &str, payload: serde_json::Value) {
        let event = JobEvent {
            job_id,
            kind: kind.to_string(),
            payload,
            at: chrono::Utc::now(),
        };
        // A dropped send simply means no live subscribers.
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<JobEvent> {
        self.sender.subscribe()
    }
}

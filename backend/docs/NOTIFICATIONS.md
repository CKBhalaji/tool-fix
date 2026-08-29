# Notifications

`toolfix-notifications` abstracts delivery behind one trait:

```rust
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn channels(&self) -> &'static [NotificationChannel];
    async fn send(&self, request: DeliveryRequest) -> Result<DeliveryResult, NotificationError>;
}
```

Channels: `push`, `sms`, `email`, `in_app`. The initial provider is
`LoggingProvider` (traces the delivery, reports success). FCM, SMS, and email
providers implement the same trait later without touching business logic.

## Flow

```text
business event (e.g. breakdown created)
      ↓
notifications row created (intent, kind, channel, job, payload)   ← durable record
      ↓
provider.send(...)                      ← dispatch (matching service or runtime loop)
      ↓
row updated: sent (provider + detail) or failed
```

`toolfix-runtime` also runs a dispatch loop that picks up `pending` rows
(at-least-once; a single dispatcher runs per process in this phase) and marks
them sent/failed. Customers are notified in-app on offers received, offer
acceptance, mechanic en-route/arrived, completion, and receipts; mechanics on
new nearby breakdowns.

Notification kinds: `new_breakdown_nearby`, `offer_received`,
`offer_accepted`, `offer_expired`, `mechanic_en_route`, `mechanic_arrived`,
`job_completed`, `job_cancelled`, `job_no_mechanic`, `payment_receipt`.

Delivery is never critical-path: a notification failure never rolls back a
job state change — the row simply records the failure for retry/audit.

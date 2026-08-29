//! Background runtime: recurring sweeps and dispatch loops.
//!
//! Recurring work never lives in HTTP handlers. Every loop exits promptly
//! when the shutdown token is cancelled, so the composition root can stop
//! the process cleanly.

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use toolfix_jobs::JobService;
use toolfix_notifications::NotificationProvider;
use toolfix_persistence::Repositories;

#[derive(Clone)]
pub struct RuntimeConfig {
    /// How often pending notifications are retried.
    pub notification_interval: Duration,
    /// How often stale offers / expired jobs are swept.
    pub expiry_interval: Duration,
    /// How often old location samples are pruned.
    pub location_cleanup_interval: Duration,
    /// Location sample retention.
    pub location_retention: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            notification_interval: Duration::from_secs(30),
            expiry_interval: Duration::from_secs(30),
            location_cleanup_interval: Duration::from_secs(600),
            location_retention: Duration::from_secs(24 * 3600),
        }
    }
}

/// One loop that exits on cancellation; joins are collected by the caller.
pub struct Runtime {
    config: RuntimeConfig,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }

    /// Spawns all background loops. Returns a join handle per task.
    pub fn spawn_all(
        &self,
        repos: Repositories,
        jobs: JobService,
        provider: std::sync::Arc<dyn NotificationProvider>,
        shutdown: CancellationToken,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        vec![
            self.spawn_offer_expiry(jobs.clone(), shutdown.clone()),
            self.spawn_notification_dispatch(repos.clone(), provider, shutdown.clone()),
            self.spawn_location_cleanup(repos, shutdown),
        ]
    }

    fn spawn_offer_expiry(&self, jobs: JobService, shutdown: CancellationToken) -> tokio::task::JoinHandle<()> {
        let interval = self.config.expiry_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        match jobs.expire_stale_offers().await {
                            Ok((offers, jobs_expired)) if offers > 0 || jobs_expired > 0 => {
                                tracing::info!(offers, jobs_expired, "expiry sweep completed");
                            }
                            Ok(_) => {}
                            Err(err) => tracing::warn!(error = %err, "offer expiry sweep failed"),
                        }
                    }
                    _ = shutdown.cancelled() => break,
                }
            }
            tracing::info!("offer expiry loop stopped");
        })
    }

    fn spawn_notification_dispatch(
        &self,
        repos: Repositories,
        provider: std::sync::Arc<dyn NotificationProvider>,
        shutdown: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        let interval = self.config.notification_interval;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let pending = match repos.notifications.claim_pending(100).await {
                            Ok(rows) => rows,
                            Err(err) => {
                                tracing::warn!(error = %err, "notification claim failed");
                                continue;
                            }
                        };
                        for notification in pending {
                            let kind = notification.kind().ok();
                            let channel = notification.channel().ok();
                            let (Some(kind), Some(channel)) = (kind, channel) else {
                                let _ = repos
                                    .notifications
                                    .mark_failed(notification.id, provider.name(), "invalid kind/channel")
                                    .await;
                                continue;
                            };
                            let request = toolfix_notifications::DeliveryRequest {
                                notification_id: notification.id,
                                recipient_user_id: notification.recipient_user_id,
                                kind,
                                channel,
                                title: kind.as_str().replace('_', " "),
                                body: notification
                                    .payload
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("You have a ToolFix update")
                                    .to_string(),
                                payload: notification.payload.clone(),
                            };
                            match provider.send(request).await {
                                Ok(delivery) => {
                                    let _ = repos
                                        .notifications
                                        .mark_sent(notification.id, provider.name(), &delivery.detail, chrono::Utc::now())
                                        .await;
                                }
                                Err(err) => {
                                    tracing::warn!(error = %err, "background delivery failed");
                                    let _ = repos
                                        .notifications
                                        .mark_failed(notification.id, provider.name(), &err.to_string())
                                        .await;
                                }
                            }
                        }
                    }
                    _ = shutdown.cancelled() => break,
                }
            }
            tracing::info!("notification dispatch loop stopped");
        })
    }

    fn spawn_location_cleanup(
        &self,
        repos: Repositories,
        shutdown: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        let interval = self.config.location_cleanup_interval;
        let retention = self.config.location_retention;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(retention).unwrap_or_else(|_| chrono::Duration::hours(24));
                        match repos.locations.delete_older_than(cutoff).await {
                            Ok(removed) if removed > 0 => {
                                tracing::debug!(removed, "location retention sweep");
                            }
                            Ok(_) => {}
                            Err(err) => tracing::warn!(error = %err, "location cleanup failed"),
                        }
                    }
                    _ = shutdown.cancelled() => break,
                }
            }
            tracing::info!("location cleanup loop stopped");
        })
    }
}

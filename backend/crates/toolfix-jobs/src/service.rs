//! The JobService: orchestrates breakdown analysis, matching, offers,
//! lifecycle transitions, payments, and ratings.

use std::sync::Arc;

use chrono::{Duration, Utc};
use toolfix_contracts::breakdown::BreakdownCreateRequest;
use toolfix_contracts::offer::OfferCreateRequest;
use toolfix_contracts::response::MechanicFeedItem;
use toolfix_contracts::{
    Currency, JobStatus, MediaKind, NotificationChannel, NotificationKind, PaymentMethod,
    RepairCategory,
};
use toolfix_location::events::LocationSubject;
use toolfix_persistence::models::{BreakdownRow, JobRow, OfferRow};
use toolfix_persistence::Repositories;

use crate::events::EventHub;

#[derive(Debug, thiserror::Error)]
pub enum JobsError {
    #[error("database error: {0}")]
    Database(#[from] toolfix_persistence::PersistenceError),
    #[error("domain rule violated: {0}")]
    Domain(#[from] toolfix_domain::DomainError),
    #[error("matching error: {0}")]
    Matching(#[from] toolfix_matching::MatchingError),
    #[error("pricing error: {0}")]
    Pricing(#[from] toolfix_pricing::PricingError),
    #[error("agent error: {0}")]
    Agent(#[from] toolfix_agent::AgentError),
    #[error("storage error: {0}")]
    Storage(#[from] toolfix_storage::StorageError),
    #[error("payment error: {0}")]
    Payment(#[from] toolfix_payments::PaymentError),
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
}

impl From<toolfix_domain::state_machine::TransitionError> for JobsError {
    fn from(e: toolfix_domain::state_machine::TransitionError) -> Self {
        JobsError::Domain(toolfix_domain::DomainError::IllegalTransition {
            from: e.from,
            to: e.to,
        })
    }
}

pub struct OfferOutcome {
    pub job: JobRow,
    pub accepted: OfferRow,
    pub expired_competing: u64,
}

#[derive(Clone)]
pub struct JobService {
    repos: Repositories,
    matching: toolfix_matching::MatchingService,
    pricing: toolfix_pricing::PricingService,
    agent: Option<toolfix_agent::AgentWorkflow>,
    storage: Arc<dyn toolfix_storage::StorageBackend>,
    payments: Arc<dyn toolfix_payments::PaymentProvider>,
    events: EventHub,
    offer_expiry_secs: i64,
}

impl JobService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repos: Repositories,
        matching: toolfix_matching::MatchingService,
        pricing: toolfix_pricing::PricingService,
        agent: Option<toolfix_agent::AgentWorkflow>,
        storage: Arc<dyn toolfix_storage::StorageBackend>,
        payments: Arc<dyn toolfix_payments::PaymentProvider>,
        events: EventHub,
        offer_expiry_secs: i64,
    ) -> Self {
        Self {
            repos,
            matching,
            pricing,
            agent,
            storage,
            payments,
            events,
            offer_expiry_secs,
        }
    }

    pub fn events(&self) -> &EventHub {
        &self.events
    }

    fn publish(&self, job_id: uuid::Uuid, kind: &str, payload: serde_json::Value) {
        self.events.publish(job_id, kind, payload);
    }

    // ------------------------------------------------------------------
    // Breakdown creation + AI analysis + matching pipeline
    // ------------------------------------------------------------------

    /// Creates the breakdown + job, then spawns the analyze/match pipeline
    /// in the background so the emergency request returns immediately.
    pub async fn create_breakdown(
        &self,
        user_id: uuid::Uuid,
        request: BreakdownCreateRequest,
    ) -> Result<(BreakdownRow, JobRow), JobsError> {
        let location = toolfix_contracts::LatLng::new(request.latitude, request.longitude)
            .map_err(JobsError::Validation)?;
        toolfix_domain::breakdown::validate_new_breakdown(
            &request.problem_description,
            &request.vehicle_symptoms,
            location,
        )?;

        // The vehicle must belong to the requesting user.
        let vehicle = self.repos.vehicles.find_by_id(request.vehicle_id).await?;
        if vehicle.owner_user_id != user_id {
            return Err(JobsError::NotFound);
        }

        let breakdown = self
            .repos
            .breakdowns
            .create(
                user_id,
                vehicle.id,
                request.latitude,
                request.longitude,
                request.address.as_deref(),
                &request.problem_description,
                &request.vehicle_symptoms,
            )
            .await?;
        let job = self
            .repos
            .jobs
            .create(breakdown.id, user_id, vehicle.id)
            .await?;

        self.publish(job.id, "job.created", serde_json::json!({ "status": job.status }));

        let this = self.clone();
        let job_id = job.id;
        tokio::spawn(async move {
            if let Err(err) = this.run_analysis_pipeline(job_id).await {
                tracing::error!(job_id = %job_id, error = %err, "analysis pipeline failed");
                let _ = this.mark_failed(job_id, &err.to_string()).await;
            }
        });

        Ok((breakdown, job))
    }

    /// Background pipeline: analyze -> price -> match -> notify.
    async fn run_analysis_pipeline(&self, job_id: uuid::Uuid) -> Result<(), JobsError> {
        self.transition(job_id, JobStatus::Analyzing, None, "analysis started").await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        let breakdown = self.repos.breakdowns.find_by_id(job.breakdown_id).await?;
        let vehicle = self.repos.vehicles.find_by_id(job.vehicle_id).await?;

        // ---- Agentic analysis (advisory) ----
        let mut category: Option<RepairCategory> = None;
        let mut severity = None;
        if let Some(agent) = &self.agent {
            let media = self.repos.breakdowns.list_media(breakdown.id).await?;
            let photo = media.iter().find(|m| m.kind().ok() == Some(MediaKind::Photo));
            let image: Option<(String, toolfix_storage::Blob)> = match photo {
                Some(m) => match self.storage.get(&m.storage_key).await {
                    Ok(bytes) => Some((
                        m.content_type
                            .clone()
                            .unwrap_or_else(|| "image/jpeg".into()),
                        bytes,
                    )),
                    Err(err) => {
                        tracing::warn!(error = %err, "could not load breakdown photo for agent");
                        None
                    }
                },
                None => None,
            };

            let input = toolfix_agent::DiagnosisInput {
                vehicle_kind: vehicle.kind().ok(),
                vehicle_display: Some(vehicle_display(&vehicle)),
                description: breakdown.problem_description.clone(),
                symptoms: breakdown.symptoms(),
                image_base64: image.as_ref().map(|(_, blob)| blob_base64(blob)),
                image_mime: image.as_ref().map(|(mime, _)| mime.clone()),
            };

            let started = std::time::Instant::now();
            match agent.diagnose_breakdown(&input).await {
                Ok(diagnosis) => {
                    let dto = diagnosis.to_dto();
                    // Audit trail first (record even the model metadata).
                    let run = self
                        .repos
                        .agents
                        .record_run(
                            breakdown.id,
                            Some(job.id),
                            "diagnosis",
                            agent.provider_name(),
                            Some(agent.model()),
                            "succeeded",
                            None,
                            started.elapsed().as_millis() as i64,
                            serde_json::to_value(&dto).unwrap_or_default(),
                        )
                        .await?;
                    self.repos
                        .breakdowns
                        .upsert_diagnosis(
                            breakdown.id,
                            Some(run.id),
                            &dto.possible_issue,
                            dto.confidence,
                            dto.severity,
                            &dto.recommended_service,
                            dto.repair_category,
                            dto.estimated_cost_min_minor,
                            dto.estimated_cost_max_minor,
                            dto.requires_towing,
                            &dto.reasoning_summary,
                        )
                        .await?;
                    self.pricing
                        .record_agent_estimate(
                            breakdown.id,
                            Some(job.id),
                            dto.repair_category,
                            dto.estimated_cost_min_minor,
                            dto.estimated_cost_max_minor,
                            None,
                        )
                        .await?;
                    category = Some(dto.repair_category);
                    severity = Some(dto.severity);
                    self.publish(
                        job.id,
                        "job.diagnosis_ready",
                        serde_json::to_value(&dto).unwrap_or_default(),
                    );
                }
                Err(err) => {
                    tracing::warn!(error = %err, "agent diagnosis failed; continuing deterministically");
                    let _ = self
                        .repos
                        .agents
                        .record_run(
                            breakdown.id,
                            Some(job.id),
                            "diagnosis",
                            agent.provider_name(),
                            Some(agent.model()),
                            "failed",
                            Some(&err.to_string()),
                            0,
                            serde_json::json!({}),
                        )
                        .await;
                }
            }
        }

        // ---- Deterministic matching + notification ----
        self.transition(job_id, JobStatus::MechanicsSearching, None, "searching mechanics").await?;
        let candidates = self
            .matching
            .find_mechanics(&breakdown, vehicle.kind().ok(), category)
            .await?;
        if candidates.is_empty() {
            self.transition(
                job_id,
                JobStatus::NoMechanicAvailable,
                None,
                "no mechanics found within radius",
            )
            .await?;
            self.publish(job_id, "job.no_mechanic", serde_json::json!({}));
            return Ok(());
        }
        self.matching
            .notify_mechanics(job_id, &breakdown, &candidates)
            .await?;

        let deadline = Utc::now() + Duration::seconds(self.offer_expiry_secs);
        self.repos.jobs.set_offer_window(job_id, Some(deadline)).await?;
        self.transition(
            job_id,
            JobStatus::MechanicsNotified,
            None,
            &format!("{} mechanics notified", candidates.len()),
        )
        .await?;
        self.publish(
            job_id,
            "job.mechanics_notified",
            serde_json::json!({ "count": candidates.len(), "offer_window_expires_at": deadline }),
        );
        let _ = severity;
        Ok(())
    }

    async fn mark_failed(&self, job_id: uuid::Uuid, reason: &str) -> Result<(), JobsError> {
        let job = self.repos.jobs.find_by_id(job_id).await?;
        if toolfix_domain::can_transition(job.status()?, JobStatus::Failed) {
            self.transition(job_id, JobStatus::Failed, None, reason).await?;
            self.publish(job_id, "job.failed", serde_json::json!({ "reason": reason }));
        }
        Ok(())
    }

    /// State-machine-guarded transition wrapper (validates in the domain
    /// first, then applies atomically with a history row).
    async fn transition(
        &self,
        job_id: uuid::Uuid,
        to: JobStatus,
        changed_by: Option<uuid::Uuid>,
        reason: &str,
    ) -> Result<JobRow, JobsError> {
        let job = self.repos.jobs.find_by_id(job_id).await?;
        let from = job.status()?;
        toolfix_domain::validate_transition(from, to)?;
        let mut tx = self.repos.begin().await?;
        let updated = self
            .repos
            .jobs
            .transition_job(&mut tx, job_id, from, to, changed_by, Some(reason))
            .await?;
        tx.commit().await?;
        Ok(updated)
    }

    // ------------------------------------------------------------------
    // Offers
    // ------------------------------------------------------------------

    pub async fn create_offer(
        &self,
        mechanic_user_id: uuid::Uuid,
        job_id: uuid::Uuid,
        request: OfferCreateRequest,
    ) -> Result<OfferRow, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        let job_status = job.status()?;

        // The offer must respect the state machine's offer window.
        let now = Utc::now();
        if !toolfix_domain::assistance_job::is_accepting_offers(
            job_status,
            job.offer_window_expires_at,
            now,
        ) {
            return Err(JobsError::Domain(
                toolfix_domain::DomainError::NotAcceptingOffers(job.id),
            ));
        }

        toolfix_domain::offer::validate_new_offer(
            request.quoted_price_minor,
            request.estimated_arrival_minutes,
            request.message.as_deref(),
        )?;

        let expires_at = job
            .offer_window_expires_at
            .unwrap_or(now + Duration::seconds(self.offer_expiry_secs));
        let offer = match self
            .repos
            .offers
            .create(
                job_id,
                mechanic.id,
                request.quoted_price_minor,
                request.estimated_arrival_minutes,
                request.message.as_deref(),
                expires_at,
            )
            .await
        {
            Ok(offer) => offer,
            Err(err) if err.is_unique_violation() => {
                return Err(JobsError::Conflict("mechanic already bid on this job".into()))
            }
            Err(err) => return Err(err.into()),
        };
        self.repos
            .offers
            .insert_offer_event(offer.id, "created", Some(mechanic_user_id), None)
            .await?;

        // First offer moves the job into offers_received.
        if job_status == JobStatus::MechanicsNotified {
            let _ = self
                .transition(job_id, JobStatus::OffersReceived, Some(mechanic_user_id), "first offer received")
                .await;
        }

        // Notify the customer (best-effort).
        let customer_payload = serde_json::json!({ "job_id": job_id.to_string(), "offer_id": offer.id.to_string() });
        if let Ok(notification) = self
            .repos
            .notifications
            .create(
                job.customer_user_id,
                NotificationKind::OfferReceived.as_str(),
                NotificationChannel::InApp.as_str(),
                Some(job_id),
                customer_payload.clone(),
            )
            .await
        {
            let _ = self
                .repos
                .notifications
                .mark_sent(notification.id, "in_app", "queued", Utc::now())
                .await;
        }

        self.publish(job_id, "offer.created", customer_payload);
        Ok(offer)
    }

    pub async fn list_offers(&self, job_id: uuid::Uuid) -> Result<Vec<OfferRow>, JobsError> {
        Ok(self.repos.offers.list_by_job(job_id).await?)
    }

    /// Customer accepts an offer — the transactional marketplace moment.
    pub async fn select_offer(
        &self,
        customer_user_id: uuid::Uuid,
        offer_id: uuid::Uuid,
    ) -> Result<OfferOutcome, JobsError> {
        let offer = self.repos.offers.find_by_id(offer_id).await?;
        let job = self.repos.jobs.find_by_id(offer.job_id).await?;
        let from = job.status()?;
        toolfix_domain::validate_transition(from, JobStatus::MechanicSelected)?;

        let mut tx = self.repos.begin().await?;
        let (updated_job, expired) = self
            .repos
            .offers
            .accept_offer_transaction(&mut tx, offer_id, offer.job_id, customer_user_id, from, JobStatus::MechanicSelected)
            .await?;
        tx.commit().await?;

        self.publish(
            offer.job_id,
            "offer.accepted",
            serde_json::json!({ "offer_id": offer_id.to_string(), "mechanic_id": offer.mechanic_id.to_string() }),
        );
        let accepted = self.repos.offers.find_by_id(offer_id).await?;
        Ok(OfferOutcome {
            job: updated_job,
            accepted,
            expired_competing: expired,
        })
    }

    pub async fn withdraw_offer(
        &self,
        mechanic_user_id: uuid::Uuid,
        offer_id: uuid::Uuid,
    ) -> Result<(), JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let removed = self.repos.offers.withdraw(offer_id, mechanic.id).await?;
        if !removed {
            return Err(JobsError::NotFound);
        }
        let offer = self.repos.offers.find_by_id(offer_id).await?;
        self.publish(offer.job_id, "offer.withdrawn", serde_json::json!({ "offer_id": offer_id.to_string() }));
        Ok(())
    }

    // ------------------------------------------------------------------
    // Mechanic execution path
    // ------------------------------------------------------------------

    pub async fn start_travel(&self, mechanic_user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        self.ensure_selected_mechanic(&job, mechanic.id)?;
        let updated = self
            .transition(job_id, JobStatus::MechanicEnRoute, Some(mechanic_user_id), "mechanic travelling")
            .await?;
        self.notify_customer(&job, NotificationKind::MechanicEnRoute).await?;
        self.publish(job_id, "job.en_route", serde_json::json!({}));
        Ok(updated)
    }

    pub async fn mark_arrived(&self, mechanic_user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        self.ensure_selected_mechanic(&job, mechanic.id)?;
        let updated = self
            .transition(job_id, JobStatus::MechanicArrived, Some(mechanic_user_id), "mechanic arrived")
            .await?;
        self.notify_customer(&job, NotificationKind::MechanicArrived).await?;
        self.publish(job_id, "job.arrived", serde_json::json!({}));
        Ok(updated)
    }

    pub async fn start_repair(&self, mechanic_user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        self.ensure_selected_mechanic(&job, mechanic.id)?;
        let updated = self
            .transition(job_id, JobStatus::RepairInProgress, Some(mechanic_user_id), "repair started")
            .await?;
        self.publish(job_id, "job.repair_started", serde_json::json!({}));
        Ok(updated)
    }

    pub async fn complete_repair(&self, mechanic_user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        let job = self.repos.jobs.find_by_id(job_id).await?;
        self.ensure_selected_mechanic(&job, mechanic.id)?;
        let updated = self
            .transition(job_id, JobStatus::RepairCompleted, Some(mechanic_user_id), "repair completed")
            .await?;
        self.publish(job_id, "job.repair_completed", serde_json::json!({}));
        Ok(updated)
    }

    // ------------------------------------------------------------------
    // Payments
    // ------------------------------------------------------------------

    /// Customer pays the accepted offer amount. The provider is initiated
    /// and confirmed here (cash/stub); the final price becomes history.
    pub async fn pay_job(
        &self,
        customer_user_id: uuid::Uuid,
        job_id: uuid::Uuid,
        method: PaymentMethod,
    ) -> Result<toolfix_contracts::payment::PaymentResponse, JobsError> {
        let job = self.repos.jobs.find_by_id(job_id).await?;
        if job.customer_user_id != customer_user_id {
            return Err(JobsError::NotFound);
        }
        let amount = job.final_amount_minor.ok_or_else(|| {
            JobsError::Conflict("job has no accepted amount yet".into())
        })?;

        self.transition(job_id, JobStatus::PaymentPending, Some(customer_user_id), "payment initiated").await?;

        let intent = self
            .payments
            .initiate(job_id, amount, Currency::Inr, method)
            .await?;
        let mut payment = self
            .repos
            .payments
            .create(job_id, amount, Currency::Inr, method, self.payments.name(), Some(&intent.provider_payment_id))
            .await?;
        self.repos.payments.add_transaction(payment.id, "initiated", Some(&intent.provider_payment_id)).await?;

        let receipt = self.payments.confirm(&intent.provider_payment_id, amount).await?;
        payment = self.repos.payments.mark_confirmed(payment.id, &receipt.receipt_number).await?;
        self.repos.payments.add_transaction(payment.id, "confirmed", Some(&receipt.receipt_number)).await?;

        self.transition(job_id, JobStatus::Completed, Some(customer_user_id), "payment confirmed").await?;
        self.repos.mechanics.bump_completed_jobs(job.selected_mechanic_id.expect("selected")).await?;

        // Learning data: realized price per category/vehicle.
        let category = self
            .repos
            .breakdowns
            .find_diagnosis(job.breakdown_id)
            .await
            .ok()
            .and_then(|d| d.category().ok());
        let vehicle = self.repos.vehicles.find_by_id(job.vehicle_id).await.ok();
        self.pricing
            .record_final_price(
                job_id,
                category.unwrap_or(RepairCategory::Other),
                vehicle.as_ref().and_then(|v| v.kind().ok().map(|k| k.as_str().to_string())).as_deref(),
                None,
                amount,
            )
            .await?;

        self.notify_customer(&job, NotificationKind::PaymentReceipt).await?;
        self.publish(job_id, "job.completed", serde_json::json!({ "amount_minor": amount }));

        Ok(toolfix_contracts::payment::PaymentResponse {
            id: payment.id,
            job_id: payment.job_id,
            amount_minor: payment.amount_minor,
            currency: Currency::Inr,
            method: payment.method()?,
            status: payment.status()?,
            provider: payment.provider.clone(),
            receipt_number: payment.receipt_number.clone(),
            created_at: payment.created_at,
            confirmed_at: payment.confirmed_at,
        })
    }

    // ------------------------------------------------------------------
    // Cancellation / expiry
    // ------------------------------------------------------------------

    pub async fn cancel_job(&self, user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let job = self.repos.jobs.find_by_id(job_id).await?;
        if job.customer_user_id != user_id {
            return Err(JobsError::NotFound);
        }
        let updated = self.transition(job_id, JobStatus::Cancelled, Some(user_id), "cancelled by customer").await?;
        self.publish(job_id, "job.cancelled", serde_json::json!({}));
        Ok(updated)
    }

    /// Runtime sweep: expire stale offers; jobs whose window closed with no
    /// remaining offers become no_mechanic_available.
    pub async fn expire_stale_offers(&self) -> Result<(u64, u64), JobsError> {
        let job_ids = self.repos.offers.expire_stale(Utc::now()).await?;
        let expired_offers = job_ids.len() as u64;
        let mut expired_jobs = 0u64;
        for job_id in job_ids {
            let remaining = self
                .repos
                .offers
                .list_by_job(job_id)
                .await?
                .iter()
                .filter(|o| o.status == "pending")
                .count();
            if remaining == 0 {
                let job = self.repos.jobs.find_by_id(job_id).await?;
                let status = job.status()?;
                if matches!(status, JobStatus::MechanicsNotified | JobStatus::OffersReceived)
                    && job
                        .offer_window_expires_at
                        .map(|deadline| deadline < Utc::now())
                        .unwrap_or(false)
                    && self
                        .transition(job_id, JobStatus::NoMechanicAvailable, None, "offer window closed")
                        .await
                        .is_ok()
                {
                    expired_jobs += 1;
                    self.publish(job_id, "job.no_mechanic", serde_json::json!({}));
                }
            }
        }
        Ok((expired_offers, expired_jobs))
    }

    // ------------------------------------------------------------------
    // Location
    // ------------------------------------------------------------------

    pub async fn submit_mechanic_location(
        &self,
        mechanic_user_id: uuid::Uuid,
        job_id: Option<uuid::Uuid>,
        latitude: f64,
        longitude: f64,
        accuracy_m: Option<f64>,
    ) -> Result<(), JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        toolfix_contracts::LatLng::new(latitude, longitude).map_err(JobsError::Validation)?;
        let now = Utc::now();
        self.repos.mechanics.update_location(mechanic.id, latitude, longitude, accuracy_m, now).await?;
        self.repos
            .locations
            .insert_event(LocationSubject::Mechanic, mechanic.id, job_id, latitude, longitude, accuracy_m, now)
            .await?;
        if let Some(job_id) = job_id {
            self.publish(
                job_id,
                "mechanic.location",
                serde_json::json!({ "latitude": latitude, "longitude": longitude, "at": now }),
            );
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Reads
    // ------------------------------------------------------------------

    pub async fn job_for_customer(&self, user_id: uuid::Uuid, job_id: uuid::Uuid) -> Result<JobRow, JobsError> {
        let job = self.repos.jobs.find_by_id(job_id).await?;
        if job.customer_user_id != user_id {
            return Err(JobsError::NotFound);
        }
        Ok(job)
    }

    pub async fn breakdown(&self, breakdown_id: uuid::Uuid) -> Result<BreakdownRow, JobsError> {
        Ok(self.repos.breakdowns.find_by_id(breakdown_id).await?)
    }

    pub async fn diagnosis(
        &self,
        breakdown_id: uuid::Uuid,
    ) -> Result<Option<toolfix_persistence::models::DiagnosisRow>, JobsError> {
        Ok(self.repos.breakdowns.find_diagnosis(breakdown_id).await.ok())
    }

    pub async fn estimate(
        &self,
        breakdown_id: uuid::Uuid,
    ) -> Result<Option<toolfix_persistence::models::PriceEstimateRow>, JobsError> {
        Ok(self
            .repos
            .pricing
            .latest_estimate_for_breakdown(breakdown_id)
            .await
            .ok())
    }

    pub async fn media(&self, breakdown_id: uuid::Uuid) -> Result<Vec<toolfix_persistence::models::BreakdownMediaRow>, JobsError> {
        Ok(self.repos.breakdowns.list_media(breakdown_id).await?)
    }

    pub async fn read_media(&self, key: &str) -> Result<toolfix_storage::Blob, JobsError> {
        Ok(self.storage.get(key).await?)
    }

    pub async fn status_history(
        &self,
        job_id: uuid::Uuid,
    ) -> Result<Vec<toolfix_persistence::models::JobStatusHistoryRow>, JobsError> {
        Ok(self.repos.jobs.status_history(job_id, 100).await?)
    }

    pub async fn my_jobs(&self, user_id: uuid::Uuid, limit: i64) -> Result<Vec<JobRow>, JobsError> {
        Ok(self.repos.jobs.list_by_customer(user_id, limit).await?)
    }

    pub async fn mechanic_jobs(&self, user_id: uuid::Uuid) -> Result<Vec<JobRow>, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(user_id).await?;
        Ok(self.repos.jobs.list_by_mechanic(mechanic.id, 50).await?)
    }

    /// Nearby request feed for a mechanic, enriched with vehicle kind,
    /// diagnosis, and estimate.
    pub async fn mechanic_feed(
        &self,
        mechanic_user_id: uuid::Uuid,
        latitude: f64,
        longitude: f64,
    ) -> Result<Vec<MechanicFeedItem>, JobsError> {
        let mechanic = self.repos.mechanics.find_by_user_id(mechanic_user_id).await?;
        toolfix_contracts::LatLng::new(latitude, longitude).map_err(JobsError::Validation)?;
        let max_radius = mechanic.service_area_km as f64 * 1_000.0;
        let (min_lat, max_lat, min_lng, max_lng) = toolfix_location::bounding_box(
            toolfix_contracts::LatLng::new(latitude, longitude).expect("validated"),
            max_radius,
        );
        let since = Utc::now() - Duration::seconds(self.matching.config().feed_lookback_secs);
        let rows = self
            .repos
            .jobs
            .list_open_with_breakdowns_in_box(min_lat, max_lat, min_lng, max_lng, since, 50)
            .await?;

        let center = toolfix_contracts::LatLng::new(latitude, longitude).expect("validated");
        let mut items = Vec::new();
        for row in rows {
            let distance = toolfix_location::haversine_distance_m(
                center,
                toolfix_contracts::LatLng::new(row.latitude, row.longitude)
                    .map_err(JobsError::Validation)?,
            );
            if distance > max_radius {
                continue;
            }
            let vehicle = self.repos.vehicles.find_by_id(row.vehicle_id).await.ok();
            let diagnosis = self
                .repos
                .breakdowns
                .find_diagnosis(row.breakdown_id)
                .await
                .ok()
                .map(|d| toolfix_contracts::breakdown::AgentDiagnosisDto {
                    possible_issue: d.possible_issue.clone(),
                    confidence: d.confidence as f64,
                    severity: d.severity().unwrap_or(toolfix_contracts::Severity::Medium),
                    recommended_service: d.recommended_service.clone(),
                    repair_category: d.category().unwrap_or(RepairCategory::Other),
                    estimated_cost_min_minor: d.estimated_cost_min_minor,
                    estimated_cost_max_minor: d.estimated_cost_max_minor,
                    requires_towing: d.requires_towing,
                    reasoning_summary: d.reasoning_summary.clone(),
                });
            let estimate = self
                .repos
                .pricing
                .latest_estimate_for_breakdown(row.breakdown_id)
                .await
                .ok()
                .map(|e| toolfix_contracts::breakdown::PriceEstimateDto {
                    repair_category: e.category().unwrap_or(RepairCategory::Other),
                    estimated_cost_min_minor: e.estimated_cost_min_minor,
                    estimated_cost_max_minor: e.estimated_cost_max_minor,
                    currency: Currency::Inr,
                    source: e.source().unwrap_or(toolfix_contracts::EstimateSource::Historical),
                    notes: e.notes.clone(),
                    created_at: e.created_at,
                });
            items.push(MechanicFeedItem {
                job_id: row.job_id,
                breakdown_id: row.breakdown_id,
                problem_description: row.problem_description.clone(),
                vehicle_symptoms: row.symptoms(),
                latitude: row.latitude,
                longitude: row.longitude,
                address: row.address.clone(),
                distance_km: (distance / 1000.0 * 10.0).round() / 10.0,
                vehicle_kind: vehicle.as_ref().and_then(|v| v.kind().ok()),
                diagnosis,
                price_estimate: estimate,
                offer_window_expires_at: row.offer_window_expires_at,
                notified: false,
            });
        }
        Ok(items)
    }

    // ------------------------------------------------------------------
    // Ratings
    // ------------------------------------------------------------------

    pub async fn rate_job(
        &self,
        customer_user_id: uuid::Uuid,
        request: toolfix_contracts::rating::RatingCreateRequest,
    ) -> Result<toolfix_persistence::models::RatingRow, JobsError> {
        toolfix_domain::rating::validate_score(request.score)?;
        let job = self.repos.jobs.find_by_id(request.job_id).await?;
        if job.customer_user_id != customer_user_id {
            return Err(JobsError::NotFound);
        }
        if job.status()? != JobStatus::Completed {
            return Err(JobsError::Conflict("job is not completed yet".into()));
        }
        let mechanic_id = job
            .selected_mechanic_id
            .ok_or_else(|| JobsError::Conflict("job has no selected mechanic".into()))?;

        let rating = match self
            .repos
            .ratings
            .create(request.job_id, mechanic_id, customer_user_id, request.score, request.comment.as_deref())
            .await
        {
            Ok(r) => r,
            Err(err) if err.is_unique_violation() => {
                return Err(JobsError::Conflict("job already rated".into()))
            }
            Err(err) => return Err(err.into()),
        };
        let average = self.repos.ratings.average_for_mechanic(mechanic_id).await?;
        self.repos.mechanics.store_rating_average(mechanic_id, average).await?;
        self.publish(request.job_id, "job.rated", serde_json::json!({ "score": request.score }));
        Ok(rating)
    }

    pub async fn mechanic_reviews(
        &self,
        mechanic_id: uuid::Uuid,
    ) -> Result<Vec<toolfix_persistence::models::RatingRow>, JobsError> {
        Ok(self.repos.ratings.list_by_mechanic(mechanic_id, 50).await?)
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn ensure_selected_mechanic(&self, job: &JobRow, mechanic_id: uuid::Uuid) -> Result<(), JobsError> {
        if job.selected_mechanic_id == Some(mechanic_id) {
            Ok(())
        } else {
            Err(JobsError::Forbidden)
        }
    }

    async fn notify_customer(&self, job: &JobRow, kind: NotificationKind) -> Result<(), JobsError> {
        let payload = serde_json::json!({ "job_id": job.id.to_string() });
        if let Ok(notification) = self
            .repos
            .notifications
            .create(
                job.customer_user_id,
                kind.as_str(),
                NotificationChannel::InApp.as_str(),
                Some(job.id),
                payload,
            )
            .await
        {
            let _ = self
                .repos
                .notifications
                .mark_sent(notification.id, "in_app", "queued", Utc::now())
                .await;
        }
        Ok(())
    }

    /// Uploads breakdown media to the storage backend and records metadata.
    pub async fn add_media(
        &self,
        user_id: uuid::Uuid,
        breakdown_id: uuid::Uuid,
        media_kind: MediaKind,
        content_type: Option<&str>,
        data: toolfix_storage::Blob,
    ) -> Result<toolfix_persistence::models::BreakdownMediaRow, JobsError> {
        let breakdown = self.repos.breakdowns.find_by_id(breakdown_id).await?;
        if breakdown.user_id != user_id {
            return Err(JobsError::NotFound);
        }
        let extension = match media_kind {
            MediaKind::Photo => "jpg",
            MediaKind::Video => "mp4",
        };
        let key = format!("breakdowns/{breakdown_id}/{}.{extension}", uuid::Uuid::now_v7());
        let data_len = data.len() as i64;
        self.storage.put(&key, content_type, data).await?;
        let row = self
            .repos
            .breakdowns
            .add_media(breakdown_id, media_kind, &key, content_type, Some(data_len))
            .await?;
        Ok(row)
    }
}

fn vehicle_display(vehicle: &toolfix_persistence::models::VehicleRow) -> String {
    match (&vehicle.make, &vehicle.model) {
        (Some(make), Some(model)) => format!("{make} {model}"),
        (Some(make), None) => make.clone(),
        (None, Some(model)) => model.clone(),
        (None, None) => vehicle.vehicle_kind.clone(),
    }
}

fn blob_base64(blob: &toolfix_storage::Blob) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(blob)
}

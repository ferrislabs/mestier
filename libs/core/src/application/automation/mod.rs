use common::CoreError;
use mestier_macros::transactional;

use chrono::Utc;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        ports::{
            DeliveryHandler, DeliveryOutcome, DeliveryRepository, DispatchOutcome, DueDelivery,
            EventDispatchRepository,
        },
        settings::with_jitter,
    },
};

/// What one delivery pass did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PassOutcome {
    pub claimed: usize,
    pub succeeded: usize,
    pub failed: usize,
}

mod credential;
mod tests;

impl MestierUseCase {
    /// Runs one fan-out pass: read undispatched events, create a delivery per
    /// interested subscription, mark them done.
    ///
    /// Bounded by `batch` and safe to run concurrently — the claim uses
    /// `FOR UPDATE SKIP LOCKED`, so a second caller picks up other events
    /// rather than waiting.
    #[transactional(event_dispatch)]
    pub async fn dispatch_pending_events(&self, batch: i64) -> Result<DispatchOutcome, CoreError> {
        let mut repository = event_dispatch_repository;
        repository.dispatch_pending(batch).await
    }

    /// Runs one delivery pass: claim what is due, execute it, record what
    /// happened.
    ///
    /// **The handler runs outside every transaction.** Executing a delivery
    /// means an outbound HTTP call, and holding a Postgres transaction open
    /// across it would pin a connection for the length of a stranger's
    /// timeout. Claiming and settling are each their own short transaction;
    /// the network sits between them.
    ///
    /// A handler that panics or a process that dies between the two leaves the
    /// delivery `in_flight`; `recover_lost_deliveries` brings it back.
    pub async fn run_delivery_pass<H: DeliveryHandler>(
        &self,
        handler: &H,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> Result<PassOutcome, CoreError> {
        let claimed = self.claim_deliveries(worker, batch, per_org).await?;
        let mut outcome = PassOutcome {
            claimed: claimed.len(),
            ..PassOutcome::default()
        };

        for delivery in &claimed {
            match handler.deliver(delivery).await {
                DeliveryOutcome::Succeeded => {
                    self.settle_succeeded(delivery.id).await?;
                    outcome.succeeded += 1;
                }
                DeliveryOutcome::Failed { error } => {
                    self.settle_failed(delivery, &error).await?;
                    outcome.failed += 1;
                }
            }
        }

        Ok(outcome)
    }

    #[transactional(delivery)]
    async fn claim_deliveries(
        &self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> Result<Vec<DueDelivery>, CoreError> {
        let mut repository = delivery_repository;
        repository.claim_due(worker, batch, per_org).await
    }

    #[transactional(delivery)]
    async fn settle_succeeded(&self, delivery_id: uuid::Uuid) -> Result<(), CoreError> {
        let mut repository = delivery_repository;
        repository.settle_succeeded(delivery_id).await
    }

    /// Applies the organization's schedule to a failed attempt, and disables
    /// the target when its failures have piled up.
    #[transactional(delivery)]
    async fn settle_failed(&self, delivery: &DueDelivery, error: &str) -> Result<(), CoreError> {
        let mut repository = delivery_repository;
        let settings = repository.settings_for(delivery.org_id).await?;

        // `None` means the schedule ran out, which is what makes it dead.
        let next_attempt_at = settings
            .backoff_after(delivery.attempts)
            .map(|interval| with_jitter(interval, delivery.id.as_u128() as u64))
            .and_then(|interval| chrono::Duration::from_std(interval).ok())
            .map(|interval| Utc::now() + interval);

        repository
            .settle_failed(delivery.id, error, next_attempt_at)
            .await?;

        if let Some(threshold) = settings.disable_target_after {
            repository
                .disable_target_if_exhausted(delivery.subscription_id, threshold)
                .await?;
        }

        Ok(())
    }

    /// Brings back deliveries a worker claimed and never settled.
    #[transactional(delivery)]
    pub async fn recover_lost_deliveries(
        &self,
        older_than: chrono::DateTime<Utc>,
    ) -> Result<u64, CoreError> {
        let mut repository = delivery_repository;
        repository.release_stale_claims(older_than).await
    }
}

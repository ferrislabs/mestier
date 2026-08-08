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

use base64::{Engine, engine::general_purpose::STANDARD};
use common::{OrganizationId, generate_uuid_v7};
use uuid::Uuid;

use crate::domain::automation::{
    catalogue::validate_event_names,
    endpoint::{
        CreateWebhookEndpointCommand, UpdateWebhookEndpointCommand, WebhookEndpoint, validate_url,
    },
    event_catalogue,
    ports::{
        AutomationSettingsRepository, DeliveryLogRepository, DeliveryRecord,
        SubscriptionRepository, WebhookEndpointRepository,
    },
    settings::{AutomationSettings, SettingsBounds},
};

/// A freshly created endpoint and the secret its owner must copy now.
///
/// The secret is a return value rather than a field on the endpoint: it exists
/// for exactly one response, and a type that cannot carry it later cannot leak
/// it later.
pub struct CreatedWebhookEndpoint {
    pub endpoint: WebhookEndpoint,
    pub secret: String,
}

impl MestierUseCase {
    /// Creates an endpoint and its subscription in one transaction.
    ///
    /// Neither can exist without the other: an endpoint subscribed to nothing
    /// never fires and looks healthy, which is the worst way for this to fail.
    #[transactional(webhook_endpoint, subscription)]
    pub async fn create_webhook_endpoint(
        &self,
        command: CreateWebhookEndpointCommand,
    ) -> Result<CreatedWebhookEndpoint, CoreError> {
        validate_url(&command.url)?;
        validate_event_names(&command.event_names, &event_catalogue())?;

        let cipher = self.cipher()?;
        let secret = cipher.generate_secret()?;
        let sealed = cipher.seal(&secret)?;

        let now = chrono::Utc::now();
        let endpoint = WebhookEndpoint {
            id: generate_uuid_v7(),
            org_id: command.org_id,
            url: command.url.trim().to_owned(),
            description: command.description,
            enabled: true,
            created_at: now,
            updated_at: now,
            disabled_at: None,
        };

        let mut endpoints = webhook_endpoint_repository;
        let created = endpoints.insert(&endpoint, &sealed).await?;

        let mut subscriptions = subscription_repository;
        subscriptions
            .upsert_for_target(command.org_id, created.id, &command.event_names, true)
            .await?;

        Ok(CreatedWebhookEndpoint {
            endpoint: created,
            secret: STANDARD.encode(&secret),
        })
    }

    #[transactional(webhook_endpoint)]
    pub async fn list_webhook_endpoints(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<WebhookEndpoint>, CoreError> {
        let mut repository = webhook_endpoint_repository;
        repository.list_by_organization(org_id).await
    }

    #[transactional(webhook_endpoint, subscription)]
    pub async fn get_webhook_endpoint(
        &self,
        id: Uuid,
    ) -> Result<(WebhookEndpoint, Vec<String>), CoreError> {
        let mut repository = webhook_endpoint_repository;
        let endpoint = repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mut subscriptions = subscription_repository;
        let events = subscriptions.event_names_for_target(id).await?;

        Ok((endpoint, events))
    }

    #[transactional(webhook_endpoint, subscription)]
    pub async fn update_webhook_endpoint(
        &self,
        command: UpdateWebhookEndpointCommand,
    ) -> Result<WebhookEndpoint, CoreError> {
        validate_url(&command.url)?;
        validate_event_names(&command.event_names, &event_catalogue())?;

        let mut repository = webhook_endpoint_repository;
        let existing = repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let updated = repository
            .update(&WebhookEndpoint {
                url: command.url.trim().to_owned(),
                description: command.description,
                enabled: command.enabled,
                ..existing
            })
            .await?;

        let mut subscriptions = subscription_repository;
        subscriptions
            .upsert_for_target(
                updated.org_id,
                updated.id,
                &command.event_names,
                command.enabled,
            )
            .await?;

        Ok(updated)
    }

    #[transactional(webhook_endpoint, subscription)]
    pub async fn delete_webhook_endpoint(&self, id: Uuid) -> Result<(), CoreError> {
        let mut subscriptions = subscription_repository;
        subscriptions.delete_for_target(id).await?;

        let mut repository = webhook_endpoint_repository;
        repository.delete(id).await
    }

    /// Rotation is regeneration. There is no read side, so a lost secret is
    /// replaced rather than recovered — and the new one is shown once, like
    /// the first.
    #[transactional(webhook_endpoint)]
    pub async fn regenerate_webhook_secret(&self, id: Uuid) -> Result<String, CoreError> {
        let cipher = self.cipher()?;
        let secret = cipher.generate_secret()?;
        let sealed = cipher.seal(&secret)?;

        let mut repository = webhook_endpoint_repository;
        repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;
        repository.reseal(id, &sealed).await?;

        Ok(STANDARD.encode(&secret))
    }

    #[transactional(automation_settings)]
    pub async fn get_automation_settings(
        &self,
        org_id: OrganizationId,
    ) -> Result<AutomationSettings, CoreError> {
        let mut repository = automation_settings_repository;
        repository.get(org_id).await
    }

    /// Refuses a value outside the instance bounds rather than clamping it: an
    /// operator who asked for a one-second retry and silently got sixty would
    /// have no way to find out.
    #[transactional(automation_settings)]
    pub async fn update_automation_settings(
        &self,
        org_id: OrganizationId,
        settings: AutomationSettings,
    ) -> Result<AutomationSettings, CoreError> {
        settings.validate(&SettingsBounds::default())?;

        let mut repository = automation_settings_repository;
        repository.upsert(org_id, &settings).await
    }

    #[transactional(delivery_log)]
    pub async fn list_deliveries(
        &self,
        org_id: OrganizationId,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<DeliveryRecord>, i64), CoreError> {
        let mut repository = delivery_log_repository;
        repository.list_by_organization(org_id, limit, offset).await
    }

    #[transactional(delivery_log)]
    pub async fn replay_delivery(
        &self,
        org_id: OrganizationId,
        delivery_id: Uuid,
    ) -> Result<(), CoreError> {
        let mut repository = delivery_log_repository;

        if repository.replay(org_id, delivery_id).await? {
            Ok(())
        } else {
            Err(CoreError::NotFound)
        }
    }
}

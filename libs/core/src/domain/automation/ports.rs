use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use events::EventEnvelope;
use uuid::Uuid;

use crate::domain::automation::settings::AutomationSettings;

/// Appends events to the durable log.
///
/// There is no read side and no update: the log is append-only, and the only
/// column that ever changes afterwards is `dispatched_at`, which belongs to
/// the dispatcher rather than to any emitter.
#[cfg_attr(test, mockall::automock)]
pub trait EventLogRepository: Send {
    /// Append a whole transaction's events in one statement.
    ///
    /// Called from inside the business transaction, immediately before it
    /// commits, so that a rollback takes the events with it.
    fn append(
        &mut self,
        events: &[EventEnvelope],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// What one pass of the fan-out did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DispatchOutcome {
    /// Events read and marked dispatched, including those nobody subscribed to.
    pub events: u64,
    /// Delivery rows created. Lower than `events` when subscriptions are
    /// sparse, higher when several subscribers want the same event.
    pub deliveries: u64,
}

/// Turns events into one delivery per interested subscriber.
///
/// Deliberately separate from executing a delivery: this does no external I/O,
/// so it cannot hang, and a subscriber whose endpoint is down cannot delay
/// anyone else's fan-out.
#[cfg_attr(test, mockall::automock)]
pub trait EventDispatchRepository: Send {
    fn dispatch_pending(
        &mut self,
        batch: i64,
    ) -> impl Future<Output = Result<DispatchOutcome, CoreError>> + Send;
}

/// A delivery the worker has claimed, with everything needed to execute it.
#[derive(Debug, Clone)]
pub struct DueDelivery {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub subscription_id: Uuid,
    /// `webhook` today. The workflow engine becomes another value.
    pub kind: String,
    pub target_id: Uuid,
    /// How many attempts have already failed. Indexes into the retry schedule.
    pub attempts: u32,
    pub event: EventEnvelope,
}

/// What executing a delivery produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Succeeded,
    Failed { error: String },
}

/// Executes a claimed delivery. Implemented per `subscription.kind` — webhooks
/// in W5, the workflow engine later.
///
/// Returns an outcome rather than a `Result`: a failed delivery is ordinary,
/// expected, and belongs in the retry schedule, not in the error channel of
/// the worker that runs it.
pub trait DeliveryHandler: Send + Sync {
    fn deliver(&self, delivery: &DueDelivery) -> impl Future<Output = DeliveryOutcome> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait DeliveryRepository: Send {
    /// Claim deliveries that are due, marking them `in_flight` and stamping
    /// the worker so a crashed one can be recovered.
    ///
    /// `per_org` caps how many of the batch a single organization may take.
    /// Without it, one tenant flooding the queue starves every other tenant,
    /// because the claim is ordered by due date and nothing else.
    fn claim_due(
        &mut self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> impl Future<Output = Result<Vec<DueDelivery>, CoreError>> + Send;

    /// Record a success: the delivery is done and its subscription's failure
    /// streak resets.
    fn settle_succeeded(
        &mut self,
        delivery_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Record a failure. `next_attempt_at` is `None` when the retry schedule
    /// is exhausted, which is what makes the delivery dead.
    fn settle_failed(
        &mut self,
        delivery_id: Uuid,
        error: &str,
        next_attempt_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Disable a subscription whose consecutive failures reached the
    /// organization's threshold. Returns whether it disabled anything.
    fn disable_target_if_exhausted(
        &mut self,
        subscription_id: Uuid,
        threshold: u32,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Release deliveries a worker claimed and never settled — it died in
    /// flight — so they become claimable again instead of stranded.
    fn release_stale_claims(
        &mut self,
        older_than: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn settings_for(
        &mut self,
        org_id: OrganizationId,
    ) -> impl Future<Output = Result<AutomationSettings, CoreError>> + Send;
}

use crate::domain::automation::endpoint::{SealedSecret, WebhookEndpoint};

#[cfg_attr(test, mockall::automock)]
pub trait WebhookEndpointRepository: Send {
    fn insert(
        &mut self,
        endpoint: &WebhookEndpoint,
        secret: &SealedSecret,
    ) -> impl Future<Output = Result<WebhookEndpoint, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<WebhookEndpoint>, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<WebhookEndpoint>, CoreError>> + Send;

    fn update(
        &mut self,
        endpoint: &WebhookEndpoint,
    ) -> impl Future<Output = Result<WebhookEndpoint, CoreError>> + Send;

    /// Replaces the sealed secret. Rotation is regeneration: there is no read
    /// side, so a lost secret is replaced rather than recovered.
    fn reseal(
        &mut self,
        id: Uuid,
        secret: &SealedSecret,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete(&mut self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait SubscriptionRepository: Send {
    /// Creates or replaces the subscription pointing at `target_id`. An
    /// endpoint and its subscription are written in one transaction: an
    /// endpoint subscribed to nothing is a support ticket waiting to happen.
    fn upsert_for_target(
        &mut self,
        org_id: OrganizationId,
        target_id: Uuid,
        event_names: &[String],
        enabled: bool,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn event_names_for_target(
        &mut self,
        target_id: Uuid,
    ) -> impl Future<Output = Result<Vec<String>, CoreError>> + Send;

    fn delete_for_target(
        &mut self,
        target_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait AutomationSettingsRepository: Send {
    fn get(
        &mut self,
        org_id: OrganizationId,
    ) -> impl Future<Output = Result<AutomationSettings, CoreError>> + Send;

    fn upsert(
        &mut self,
        org_id: OrganizationId,
        settings: &AutomationSettings,
    ) -> impl Future<Output = Result<AutomationSettings, CoreError>> + Send;
}

/// One row of the delivery log, as an operator reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event_name: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg_attr(test, mockall::automock)]
pub trait DeliveryLogRepository: Send {
    fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
        limit: i64,
        offset: i64,
    ) -> impl Future<Output = Result<(Vec<DeliveryRecord>, i64), CoreError>> + Send;

    /// Puts a finished delivery back in the queue. Scoped by `org_id` so an
    /// organization cannot replay somebody else's.
    fn replay(
        &mut self,
        org_id: OrganizationId,
        delivery_id: Uuid,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

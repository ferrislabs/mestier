use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use events::EventEnvelope;
use uuid::Uuid;

use crate::domain::automation::credential::Credential;
use crate::domain::automation::secret::SealedSecret;
use crate::domain::automation::settings::AutomationSettings;
use crate::domain::automation::workflow::{Graph, Workflow, WorkflowReference, WorkflowVersion};

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

/// Organization-scoped storage for credentials: what an organization stores
/// so Mestier can authenticate itself elsewhere, and the secret Mestier
/// generates to sign what it sends out. `org_id` is a parameter of every
/// method that can reach someone else's row, not a promise the caller
/// already checked — a cross-organization lookup must read back as absent,
/// not merely refused.
#[cfg_attr(test, mockall::automock)]
pub trait CredentialRepository: Send {
    fn insert(
        &mut self,
        credential: &Credential,
        sealed: &SealedSecret,
    ) -> impl Future<Output = Result<Credential, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Credential>, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Credential>, CoreError>> + Send;

    /// `sealed = None` leaves the sealed bytes untouched (`COALESCE` in the
    /// Postgres adapter), which is what lets a rename skip re-supplying the
    /// secret.
    ///
    /// The lifetime is named, not elided: `mockall::automock`'s expansion
    /// cannot infer it for a reference nested in `Option`, only clippy's
    /// non-test build sees it as needless.
    #[allow(clippy::needless_lifetimes)]
    fn update<'a>(
        &mut self,
        org_id: OrganizationId,
        credential: &Credential,
        sealed: Option<&'a SealedSecret>,
    ) -> impl Future<Output = Result<Credential, CoreError>> + Send;

    fn delete(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Workflows and their immutable versions. `org_id` is a parameter of every
/// method that can reach someone else's row, not a promise the caller
/// already checked — a cross-organization lookup must read back as absent,
/// the same discipline as [`CredentialRepository`].
#[cfg_attr(test, mockall::automock)]
pub trait WorkflowRepository: Send {
    fn insert(
        &mut self,
        workflow: &Workflow,
    ) -> impl Future<Output = Result<Workflow, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Workflow>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Workflow>, CoreError>> + Send;

    /// Name, description and `enabled` only — `current_version_id` is moved
    /// exclusively by [`Self::insert_version`].
    fn update(
        &mut self,
        org_id: OrganizationId,
        workflow: &Workflow,
    ) -> impl Future<Output = Result<Workflow, CoreError>> + Send;

    fn delete(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Inserts a new immutable version and moves `current_version_id` to
    /// point at it, in the same transaction — never a rewrite of a version a
    /// run (#200) might be executing. The version number is computed here,
    /// under an advisory lock keyed by `workflow_id` (the same technique
    /// `PgQuoteRepository::next_reference` uses for its own per-organization
    /// counter), so two concurrent saves can never mint the same number.
    fn insert_version(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        graph: &Graph,
        created_by: Option<Uuid>,
    ) -> impl Future<Output = Result<WorkflowVersion, CoreError>> + Send;

    fn find_version(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        version: i32,
    ) -> impl Future<Output = Result<Option<WorkflowVersion>, CoreError>> + Send;

    fn list_versions(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> impl Future<Output = Result<Vec<WorkflowVersion>, CoreError>> + Send;

    /// Workflows in this organization with a version — any version, not only
    /// the current one, since an older one can still be executing a run
    /// (#200) — whose graph references `credential_id`. What guards
    /// `application::automation::credential::delete_credential`: `.len()`
    /// counts, `.name` on each entry names.
    fn workflows_referencing_credential(
        &mut self,
        org_id: OrganizationId,
        credential_id: Uuid,
    ) -> impl Future<Output = Result<Vec<WorkflowReference>, CoreError>> + Send;
}

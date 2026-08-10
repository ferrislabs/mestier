use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use events::EventEnvelope;
use uuid::Uuid;

use crate::domain::automation::credential::Credential;
use crate::domain::automation::run::{DueRun, Run, RunSettlement, RunStep};
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
    /// Run rows created. Lower than `events` when subscriptions are sparse or
    /// a workflow has no current version, higher when several workflow
    /// subscriptions want the same event.
    pub runs: u64,
}

/// Turns events into workflow runs, one per interested subscription.
///
/// Deliberately separate from executing a run: this does no connector I/O, so
/// it cannot hang, and a run stuck mid-execution cannot delay anyone else's
/// fan-out.
#[cfg_attr(test, mockall::automock)]
pub trait EventDispatchRepository: Send {
    fn dispatch_pending(
        &mut self,
        batch: i64,
    ) -> impl Future<Output = Result<DispatchOutcome, CoreError>> + Send;
}

/// Reads an organization's `automation.settings` row: the retry schedule and
/// the other knobs that apply across the whole automation subsystem, not to
/// any one delivery mechanism. The delivery pipeline #201 replaces used to
/// own this port (it read the same row to schedule a webhook retry); the run
/// engine (`application::automation::run::retry_schedule_for`) reuses it
/// verbatim for the same reason a retry needs backoff, rather than
/// duplicating the query on its own.
#[cfg_attr(test, mockall::automock)]
pub trait AutomationSettingsRepository: Send {
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

    /// Reads a credential's sealed bytes alongside its metadata — the one
    /// read that carries them back out of the database, for the automation
    /// worker (#202) to open with the process cipher. No API handler calls
    /// this: a credential's data crosses exactly one boundary, from "stored,
    /// sealed" to "used to authenticate an outbound call", never through a
    /// response.
    fn find_sealed_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<(Credential, SealedSecret)>, CoreError>> + Send;

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

/// Runs and their steps — see `domain::automation::run` for the model this
/// persists, and its module doc for the at-least-once, never-replay-a-
/// succeeded-step discipline every method here exists to uphold. `org_id` is
/// a parameter of every method that can reach someone else's row, not a
/// promise the caller already checked — a cross-organization lookup must
/// read back as absent, the same discipline as [`CredentialRepository`] and
/// [`WorkflowRepository`].
#[cfg_attr(test, mockall::automock)]
pub trait RunRepository: Send {
    /// Creates the run, `trigger_payload` and all — a run has no step until
    /// the engine executes its first real connector.
    fn create_run(&mut self, run: &Run) -> impl Future<Output = Result<Run, CoreError>> + Send;

    /// Claims runs that are due, marking them `running` and stamping the
    /// worker so a crashed one can be recovered later — `FOR UPDATE SKIP
    /// LOCKED` under a per-organization quota, the same shape the delivery
    /// pipeline #201 replaces used for its own claim.
    fn claim_due(
        &mut self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> impl Future<Output = Result<Vec<DueRun>, CoreError>> + Send;

    /// Every step recorded for a run so far, in no particular order — the
    /// engine indexes them by `(connector_id, iteration_path)` itself to
    /// reconstruct where it left off.
    fn steps_for_run(
        &mut self,
        run_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RunStep>, CoreError>> + Send;

    /// The graph a specific, frozen workflow version executes.
    /// [`WorkflowRepository::find_version`] looks a version up by its
    /// *number* within a workflow; a run instead pins the version's own id
    /// (`Run::workflow_version_id`), so this looks it up directly — editing
    /// the workflow (saving a new version) afterwards moves
    /// `Workflow::current_version_id`, never this one.
    fn graph_for_version(
        &mut self,
        org_id: OrganizationId,
        workflow_version_id: Uuid,
    ) -> impl Future<Output = Result<Option<Graph>, CoreError>> + Send;

    /// Inserts a step, or updates it in place when one already exists for
    /// `(run_id, connector_id, iteration_path)` — the same row moves from
    /// `in_flight` to `succeeded`/`failed` rather than a second one being
    /// written, which is what the table's unique constraint enforces and
    /// what makes a loop's 500 iterations each their own reprensable row
    /// instead of 500 attempts at inserting the same one.
    fn upsert_step(
        &mut self,
        step: &RunStep,
    ) -> impl Future<Output = Result<RunStep, CoreError>> + Send;

    /// Applies the run-level outcome of one slice: finished, terminally
    /// failed, or rescheduled (retry backoff or simply "more to do").
    fn settle_run(
        &mut self,
        run_id: Uuid,
        settlement: RunSettlement,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Releases runs a worker claimed and never settled — it died in flight
    /// — so they become claimable again instead of stranded. The same
    /// recovery the delivery pipeline #201 replaces used for its own claims.
    fn release_stale_claims(
        &mut self,
        older_than: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

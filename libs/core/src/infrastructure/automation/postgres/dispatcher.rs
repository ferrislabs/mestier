use common::CoreError;
use mestier_macros::repository;

use crate::{
    domain::automation::ports::{DispatchOutcome, EventDispatchRepository},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = EventDispatch, backend = Postgres)]
pub struct PgEventDispatchRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgEventDispatchRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> EventDispatchRepository for PgEventDispatchRepository<'tx> {
    async fn dispatch_pending(&mut self, batch: i64) -> Result<DispatchOutcome, CoreError> {
        let mut tx = self.tx.lock().await;

        // `SKIP LOCKED` so a second dispatcher works on other events instead of
        // waiting; `FOR UPDATE` so it cannot pick the same ones.
        let claimed: Vec<uuid::Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM automation.event
            WHERE dispatched_at IS NULL
            ORDER BY occurred_at
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            batch,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if claimed.is_empty() {
            return Ok(DispatchOutcome::default());
        }

        // Set-based: one statement whatever the batch size, rather than a
        // query per event to find its subscribers.
        //
        // `gen_random_uuid` rather than v7 as everywhere else — Postgres 17
        // has no `uuidv7()`, and generating ids in Rust would mean giving up
        // the single statement. Run rows are read by their indexes, not by
        // key order, so nothing depends on it.
        //
        // The guard: `r.id IS NULL` is what lets an event dispatch even when
        // its emitting run has since been purged by retention — there is no
        // FK to enforce here, `event.actor_id` is a polymorphic reference,
        // and an event old enough for its run to be gone was dispatched long
        // ago anyway. `s.kind <> 'workflow'` is dead today (the join above
        // already requires it) but kept so the guard reads as "this identity
        // test only concerns workflow subscriptions" independently of how
        // the join happens to be shaped. `r.workflow_id <> s.target_id` is
        // the actual identity test: a run is refused only from an event its
        // *own* workflow emitted — workflow A triggering B, which
        // retriggers A, is a different workflow each time and stays allowed.
        let runs = sqlx::query!(
            r#"
            INSERT INTO automation.run
                (id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                 trigger_payload, status, next_attempt_at, created_at)
            SELECT gen_random_uuid(), e.org_id, w.id, w.current_version_id, e.id, e.payload,
                   'pending', now(), now()
            FROM automation.event e
            JOIN automation.subscription s
              ON s.org_id = e.org_id
             AND s.enabled
             AND e.name = ANY(s.event_names)
             AND s.kind = 'workflow'
            JOIN automation.workflow w
              ON w.id = s.target_id
             AND w.current_version_id IS NOT NULL
            LEFT JOIN automation.run r
                   ON e.actor_kind = 'automation' AND r.id = e.actor_id
            WHERE e.id = ANY($1)
              AND (r.id IS NULL OR s.kind <> 'workflow' OR r.workflow_id <> s.target_id)
            ON CONFLICT ON CONSTRAINT uq_automation_run_trigger_workflow DO NOTHING
            "#,
            &claimed,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        // Marked whether or not anyone wanted them: an event with no
        // subscriber is done with, not re-read on every pass.
        let events = sqlx::query!(
            "UPDATE automation.event SET dispatched_at = now() WHERE id = ANY($1)",
            &claimed,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(DispatchOutcome { events, runs })
    }
}

/// Fan-out is global by design — it claims every pending event, not one
/// organization's. Tests that assert on counts therefore cannot run
/// concurrently against a shared database, so they take this lock.
#[cfg(test)]
pub(crate) static DISPATCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::{
        domain::automation::ports::EventLogRepository,
        infrastructure::automation::postgres::PgEventLogRepository,
        infrastructure::postgres::with_tx,
    };

    struct QuoteAccepted;

    impl DomainEvent for QuoteAccepted {
        fn name(&self) -> &'static str {
            "quote.accepted"
        }
        fn version(&self) -> u16 {
            1
        }
        fn subject(&self) -> EventSubject {
            EventSubject::new("quote", Uuid::from_u128(1))
        }
        fn payload(&self) -> Value {
            json!({})
        }
    }

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    /// A workflow with a saved current version — the FK `automation.run`
    /// carries requires one, and a workflow with none must never produce a
    /// run (see `a_workflow_with_no_current_version_produces_no_run`).
    async fn seed_workflow(pool: &PgPool, org_id: OrganizationId) -> Uuid {
        use crate::domain::automation::workflow::{Graph, PlacedConnector};
        use crate::{application::default_authorizer, infrastructure::realtime::EventHub};

        // Routed through the use cases (rather than raw SQL) so the workflow
        // always has a real, valid graph a `graph_for_version` read could
        // later succeed on — the same fixture the run engine's own tests use.
        let usecase = crate::application::MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id,
                name: "Dispatcher test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        let graph = Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
            }],
            edges: Vec::new(),
        };
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph,
                    created_by: None,
                },
            )
            .await
            .unwrap();

        workflow.id
    }

    async fn seed_workflow_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        workflow_id: Uuid,
        event_names: &[&str],
        enabled: bool,
    ) -> Uuid {
        let id = generate_uuid_v7();
        let names: Vec<String> = event_names.iter().map(|n| (*n).to_owned()).collect();
        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names, enabled)
               VALUES ($1, $2, 'workflow', $3, $4, $5)"#,
            id,
            org_id.0,
            workflow_id,
            &names,
            enabled,
        )
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_event(pool: &PgPool, org_id: OrganizationId, actor: Actor) -> Uuid {
        let envelope = EventEnvelope::from_event(
            &QuoteAccepted,
            &EmissionContext {
                org_id,
                actor,
                correlation_id: None,
            },
        );
        let id = envelope.id;
        with_tx(pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&envelope)).await
        })
        .await
        .unwrap();
        id
    }

    async fn dispatch(pool: &PgPool) -> DispatchOutcome {
        with_tx(pool, async |tx| {
            let mut repo = PgEventDispatchRepository::new(&tx);
            repo.dispatch_pending(100).await
        })
        .await
        .unwrap()
    }

    async fn runs_for(pool: &PgPool, event_id: Uuid) -> i64 {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.run WHERE trigger_event_id = $1",
            event_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    async fn runs_for_workflow(pool: &PgPool, event_id: Uuid, workflow_id: Uuid) -> i64 {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.run WHERE trigger_event_id = $1 AND workflow_id = $2",
            event_id,
            workflow_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    async fn is_dispatched(pool: &PgPool, event_id: Uuid) -> bool {
        sqlx::query_scalar!(
            "SELECT dispatched_at IS NOT NULL FROM automation.event WHERE id = $1",
            event_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(false)
    }

    /// A run inserted directly rather than through `start_run`: what
    /// `seed_event`'s `Actor::automation(run_id)` needs to reference, and
    /// what the "purged run" test needs *not* to reference.
    async fn seed_run(pool: &PgPool, org_id: OrganizationId, workflow_id: Uuid) -> Uuid {
        let version_id = sqlx::query_scalar!(
            "SELECT current_version_id FROM automation.workflow WHERE id = $1",
            workflow_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .expect("workflow has a version");

        let run_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO automation.run
                   (id, org_id, workflow_id, workflow_version_id, status, created_at)
               VALUES ($1, $2, $3, $4, 'succeeded', now())"#,
            run_id,
            org_id.0,
            workflow_id,
            version_id,
        )
        .execute(pool)
        .await
        .unwrap();
        run_id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_matching_a_workflow_subscription_creates_exactly_one_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 1);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_matching_two_workflow_subscriptions_creates_a_run_for_each() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_a = seed_workflow(&pool, org_id).await;
        let workflow_b = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_a, &["quote.accepted"], true).await;
        seed_workflow_subscription(&pool, org_id, workflow_b, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for_workflow(&pool, event_id, workflow_a).await, 1);
        assert_eq!(runs_for_workflow(&pool, event_id, workflow_b).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_subscription_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], false).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert!(
            is_dispatched(&pool, event_id).await,
            "an event nobody wants is still done with, not re-read forever"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_of_another_organization_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        let their_workflow = seed_workflow(&pool, theirs).await;
        seed_workflow_subscription(&pool, theirs, their_workflow, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, mine, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_to_another_event_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.declined"], true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
    }

    /// A workflow can be subscribed without ever having a saved version (an
    /// editor draft, say) — that must not produce a run the engine could
    /// never execute (`graph_for_version` would find nothing to run).
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_with_no_current_version_produces_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = crate::application::MestierUseCase::new(
            pool.clone(),
            crate::application::default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id,
                name: "Never saved".to_string(),
                description: None,
            })
            .await
            .unwrap();
        seed_workflow_subscription(&pool, org_id, workflow.id, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert!(
            is_dispatched(&pool, event_id).await,
            "still done with, not retried forever waiting on a version that never comes"
        );
    }

    /// The identity guard: a workflow that writes and thereby emits an event
    /// it is itself subscribed to must not retrigger itself. The refusal is
    /// made observable, not merely absent, by first proving (in
    /// `an_event_matching_a_workflow_subscription_creates_exactly_one_run`)
    /// that this exact subscription shape *does* produce a run for an
    /// ordinary event — so the zero runs asserted here can only be the
    /// guard, not a subscription that never matched in the first place.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_does_not_retrigger_itself_from_its_own_emitted_event() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], true).await;
        let run_id = seed_run(&pool, org_id, workflow_id).await;
        let event_id = seed_event(&pool, org_id, Actor::automation(run_id)).await;

        dispatch(&pool).await;

        assert_eq!(
            runs_for(&pool, event_id).await,
            0,
            "the workflow's own run must not retrigger it"
        );
        assert!(
            is_dispatched(&pool, event_id).await,
            "refusing to retrigger still marks the event handled, not stuck retrying forever"
        );
    }

    /// Not a cycle detector: workflow A triggering B, which retriggers A, is
    /// two different workflows each time and stays allowed.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_triggering_a_different_workflow_that_retriggers_it_is_allowed() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_a = seed_workflow(&pool, org_id).await;
        let workflow_b = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_a, &["b.done"], true).await;
        // Run of B emits "b.done", which A is subscribed to.
        let run_of_b = seed_run(&pool, org_id, workflow_b).await;
        let event_id = seed_event(&pool, org_id, Actor::automation(run_of_b)).await;
        sqlx::query!(
            "UPDATE automation.event SET name = 'b.done' WHERE id = $1",
            event_id,
        )
        .execute(&pool)
        .await
        .unwrap();

        dispatch(&pool).await;

        assert_eq!(
            runs_for_workflow(&pool, event_id, workflow_a).await,
            1,
            "B retriggering A is a different workflow, so it is allowed"
        );
    }

    /// A run purged by retention leaves `event.actor_id` orphaned — there is
    /// no FK, the reference is polymorphic. That must not block dispatch:
    /// an event old enough for its run to be gone was dispatched long ago in
    /// practice, and refusing it here would mean a phantom guard nobody can
    /// observe or clear.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_whose_emitting_run_was_purged_still_dispatches() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], true).await;
        let purged_run_id = generate_uuid_v7();
        let event_id = seed_event(&pool, org_id, Actor::automation(purged_run_id)).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 1);
    }

    /// Fan-out has to survive being replayed — after a crash, or raced by a
    /// second dispatcher. Re-running it over an event that was already fanned
    /// out must converge, not duplicate.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_the_fan_out_creates_no_duplicate_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;

        // Simulate a crash between the insert and the mark.
        sqlx::query!(
            "UPDATE automation.event SET dispatched_at = NULL WHERE id = $1",
            event_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 1);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_dispatched_event_is_not_read_again() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        // Drain whatever earlier tests left behind, so the second pass is
        // measuring this test's event and nothing else.
        dispatch(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, &["quote.accepted"], true).await;
        seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;

        let second = dispatch(&pool).await;

        assert_eq!(second.events, 0);
    }

    #[test]
    fn the_default_outcome_creates_no_run() {
        assert_eq!(DispatchOutcome::default().runs, 0);
    }
}

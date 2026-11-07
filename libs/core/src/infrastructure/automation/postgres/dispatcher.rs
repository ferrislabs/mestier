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
            WITH candidate AS (
                SELECT e.org_id, w.id AS workflow_id, w.current_version_id,
                       e.id AS event_id, e.payload, s.trigger_id, e.occurred_at
                FROM automation.event e
                JOIN automation.subscription s
                  ON s.org_id = e.org_id
                 AND s.enabled
                 AND e.name = ANY(s.event_names)
                 AND s.kind = 'workflow'
                JOIN automation.workflow w
                  ON w.id = s.target_id
                 AND w.org_id = e.org_id
                 AND w.current_version_id IS NOT NULL
                 AND w.enabled
                 AND w.archived_at IS NULL
                JOIN automation.workflow_version wv
                  ON wv.id = w.current_version_id
                 AND EXISTS (
                       SELECT 1
                       FROM jsonb_array_elements(wv.graph -> 'triggers') AS trig
                       WHERE trig ->> 'id' = s.trigger_id
                         AND jsonb_typeof(trig -> 'kind') = 'object'
                     )
                LEFT JOIN automation.run r
                       ON e.actor_kind = 'automation' AND r.id = e.actor_id
                WHERE e.id = ANY($1)
                  AND (r.id IS NULL OR s.kind <> 'workflow' OR r.workflow_id <> s.target_id)
                  AND NOT EXISTS (
                        SELECT 1
                        FROM automation.run existing
                        WHERE existing.trigger_event_id = e.id
                          AND existing.workflow_id = w.id
                          AND existing.trigger_id IS NOT DISTINCT FROM s.trigger_id
                      )
            ),
            numbered AS (
                SELECT candidate.*,
                       row_number() OVER (
                           PARTITION BY org_id
                           ORDER BY occurred_at, event_id, workflow_id, trigger_id
                       ) AS position,
                       count(*) OVER (PARTITION BY org_id) AS batch_size
                FROM candidate
            ),
            reserved AS (
                INSERT INTO automation.run_counter AS counter (org_id, last_value)
                SELECT org_id, count(*)
                FROM candidate
                GROUP BY org_id
                ORDER BY org_id
                ON CONFLICT (org_id) DO UPDATE
                SET last_value = counter.last_value + EXCLUDED.last_value
                RETURNING counter.org_id, counter.last_value
            )
            INSERT INTO automation.run
                (id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                 trigger_payload, trigger_id, status, next_attempt_at, created_at, run_number)
            SELECT gen_random_uuid(), n.org_id, n.workflow_id, n.current_version_id, n.event_id,
                   n.payload, n.trigger_id, 'pending', now(), now(),
                   reserved.last_value - n.batch_size + n.position
            FROM numbered n
            JOIN reserved ON reserved.org_id = n.org_id
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
        use crate::domain::automation::ports::WorkflowRepository;
        use crate::domain::automation::workflow::{
            Edge, Graph, PlacedConnector, PlacedTrigger, TriggerKind, Workflow,
        };
        use crate::infrastructure::automation::postgres::PgWorkflowRepository;

        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        let graph = Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
                name: None,
                description: None,
                disabled: false,
            }],
            edges: vec![Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Events(vec!["quote.accepted".to_string()]),
                name: None,
                description: None,
            }],
        };

        with_tx(pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let now = chrono::Utc::now();
            let workflow = repo
                .insert(&Workflow {
                    id: generate_uuid_v7(),
                    org_id,
                    name: "Dispatcher test workflow".to_string(),
                    description: None,
                    enabled: true,
                    current_version_id: None,
                    layout: None,
                    folder_id: None,
                    tags: Default::default(),
                    archived_at: None,
                    created_at: now,
                    updated_at: now,
                })
                .await?;
            repo.insert_version(org_id, workflow.id, &graph, None)
                .await?;
            Ok::<_, CoreError>(workflow.id)
        })
        .await
        .unwrap()
    }

    async fn seed_manual_only_workflow(pool: &PgPool, org_id: OrganizationId) -> Uuid {
        use crate::domain::automation::workflow::{
            Edge, Graph, PlacedConnector, PlacedTrigger, TriggerKind,
        };
        use crate::{application::default_authorizer, infrastructure::realtime::EventHub};

        let usecase = crate::application::MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id,
                name: "Manual-only dispatcher test workflow".to_string(),
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
                name: None,
                description: None,
                disabled: false,
            }],
            edges: vec![Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Manual,
                name: None,
                description: None,
            }],
        };
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph,
                    layout: None,
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
        trigger_id: &str,
        event_names: &[&str],
        enabled: bool,
    ) -> Uuid {
        let id = generate_uuid_v7();
        let names: Vec<String> = event_names.iter().map(|n| (*n).to_owned()).collect();
        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, trigger_id, event_names, enabled)
               VALUES ($1, $2, 'workflow', $3, $4, $5, $6)"#,
            id,
            org_id.0,
            workflow_id,
            trigger_id,
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

        use crate::domain::automation::ports::RunRepository;

        let now = chrono::Utc::now();
        with_tx(pool, async |tx| {
            let mut repo = crate::infrastructure::automation::postgres::PgRunRepository::new(&tx);
            repo.create_run(&crate::domain::automation::run::Run {
                id: generate_uuid_v7(),
                org_id,
                workflow_id,
                workflow_version_id: version_id,
                trigger_event_id: None,
                trigger_payload: None,
                trigger_id: None,
                status: crate::domain::automation::run::RunStatus::Succeeded,
                error: None,
                next_attempt_at: None,
                locked_at: None,
                locked_by: None,
                started_at: None,
                finished_at: None,
                created_at: now,
            })
            .await
        })
        .await
        .unwrap()
        .id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_matching_a_workflow_subscription_creates_exactly_one_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 1);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_stray_subscription_for_a_manual_only_workflow_produces_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_manual_only_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(
            runs_for(&pool, event_id).await,
            0,
            "a workflow whose graph holds only a manual trigger must not fire even with a stray subscription row"
        );
        assert!(
            is_dispatched(&pool, event_id).await,
            "still done with, not retried forever"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_matching_two_workflow_subscriptions_creates_a_run_for_each() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_a = seed_workflow(&pool, org_id).await;
        let workflow_b = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_a, "t1", &["quote.accepted"], true)
            .await;
        seed_workflow_subscription(&pool, org_id, workflow_b, "t1", &["quote.accepted"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], false)
            .await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert!(
            is_dispatched(&pool, event_id).await,
            "an event nobody wants is still done with, not re-read forever"
        );
    }

    async fn set_workflow_state(pool: &PgPool, workflow_id: Uuid, enabled: bool, archived: bool) {
        sqlx::query!(
            "UPDATE automation.workflow
             SET enabled = $2, archived_at = CASE WHEN $3 THEN now() END
             WHERE id = $1",
            workflow_id,
            enabled,
            archived,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_workflow_receives_no_run_but_its_event_is_still_done_with() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        set_workflow_state(&pool, workflow_id, false, false).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_archived_workflow_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        set_workflow_state(&pool, workflow_id, false, true).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn only_the_enabled_workflow_receives_a_run_when_two_share_a_trigger() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let enabled = seed_workflow(&pool, org_id).await;
        let disabled = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, enabled, "t1", &["quote.accepted"], true).await;
        seed_workflow_subscription(&pool, org_id, disabled, "t1", &["quote.accepted"], true).await;
        set_workflow_state(&pool, disabled, false, false).await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for_workflow(&pool, event_id, enabled).await, 1);
        assert_eq!(runs_for_workflow(&pool, event_id, disabled).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn re_enabling_a_workflow_lets_the_next_event_start_a_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        set_workflow_state(&pool, workflow_id, false, false).await;
        let skipped = seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;
        set_workflow_state(&pool, workflow_id, true, false).await;
        let delivered = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, skipped).await, 0);
        assert_eq!(runs_for(&pool, delivered).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_of_another_organization_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        let their_workflow = seed_workflow(&pool, theirs).await;
        seed_workflow_subscription(
            &pool,
            theirs,
            their_workflow,
            "t1",
            &["quote.accepted"],
            true,
        )
        .await;
        let event_id = seed_event(&pool, mine, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_pointing_at_another_organizations_workflow_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        let their_workflow = seed_workflow(&pool, theirs).await;
        seed_workflow_subscription(&pool, mine, their_workflow, "t1", &["quote.accepted"], true)
            .await;
        let event_id = seed_event(&pool, mine, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(runs_for(&pool, event_id).await, 0);
        assert_eq!(counter_of(&pool, mine).await, None);
        assert_eq!(counter_of(&pool, theirs).await, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_to_another_event_receives_no_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.declined"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow.id, "t1", &["quote.accepted"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow_a, "t1", &["b.done"], true).await;
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
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
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
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;

        let second = dispatch(&pool).await;

        assert_eq!(second.events, 0);
    }

    async fn numbers_of(pool: &PgPool, org_id: OrganizationId) -> Vec<i64> {
        sqlx::query_scalar!(
            r#"SELECT run_number AS "run_number!" FROM automation.run
               WHERE org_id = $1 ORDER BY run_number"#,
            org_id.0,
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn counter_of(pool: &PgPool, org_id: OrganizationId) -> Option<i64> {
        sqlx::query_scalar!(
            "SELECT last_value FROM automation.run_counter WHERE org_id = $1",
            org_id.0,
        )
        .fetch_optional(pool)
        .await
        .unwrap()
    }

    async fn number_of_run_for(pool: &PgPool, event_id: Uuid, workflow_id: Uuid) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT run_number AS "run_number!" FROM automation.run
               WHERE trigger_event_id = $1 AND workflow_id = $2"#,
            event_id,
            workflow_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn dispatched_runs_get_distinct_consecutive_numbers_in_event_order() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        dispatch(&pool).await;
        let org_id = seed_organization(&pool).await;
        let workflow_a = seed_workflow(&pool, org_id).await;
        let workflow_b = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_a, "t1", &["quote.accepted"], true)
            .await;
        seed_workflow_subscription(&pool, org_id, workflow_b, "t1", &["quote.accepted"], true)
            .await;
        let first = seed_event(&pool, org_id, Actor::system()).await;
        let second = seed_event(&pool, org_id, Actor::system()).await;
        let third = seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(numbers_of(&pool, org_id).await, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(counter_of(&pool, org_id).await, Some(6));
        let numbers_by_event = [
            number_of_run_for(&pool, first, workflow_a).await,
            number_of_run_for(&pool, first, workflow_b).await,
            number_of_run_for(&pool, second, workflow_a).await,
            number_of_run_for(&pool, second, workflow_b).await,
            number_of_run_for(&pool, third, workflow_a).await,
            number_of_run_for(&pool, third, workflow_b).await,
        ];
        assert!(
            numbers_by_event[0].max(numbers_by_event[1])
                < numbers_by_event[2].min(numbers_by_event[3]),
            "{numbers_by_event:?}"
        );
        assert!(
            numbers_by_event[2].max(numbers_by_event[3])
                < numbers_by_event[4].min(numbers_by_event[5]),
            "{numbers_by_event:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_batch_numbers_each_organization_on_its_own_counter() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        dispatch(&pool).await;
        let org_a = seed_organization(&pool).await;
        let org_b = seed_organization(&pool).await;
        let workflow_a = seed_workflow(&pool, org_a).await;
        let workflow_b = seed_workflow(&pool, org_b).await;
        seed_workflow_subscription(&pool, org_a, workflow_a, "t1", &["quote.accepted"], true).await;
        seed_workflow_subscription(&pool, org_b, workflow_b, "t1", &["quote.accepted"], true).await;
        seed_run(&pool, org_a, workflow_a).await;
        seed_event(&pool, org_a, Actor::system()).await;
        seed_event(&pool, org_b, Actor::system()).await;
        seed_event(&pool, org_a, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(numbers_of(&pool, org_a).await, vec![1, 2, 3]);
        assert_eq!(numbers_of(&pool, org_b).await, vec![1]);
        assert_eq!(counter_of(&pool, org_a).await, Some(3));
        assert_eq!(counter_of(&pool, org_b).await, Some(1));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_the_fan_out_burns_no_number() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        dispatch(&pool).await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        let event_id = seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;

        sqlx::query!(
            "UPDATE automation.event SET dispatched_at = NULL WHERE id = $1",
            event_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        dispatch(&pool).await;
        seed_event(&pool, org_id, Actor::system()).await;
        dispatch(&pool).await;

        assert_eq!(numbers_of(&pool, org_id).await, vec![1, 2]);
        assert_eq!(counter_of(&pool, org_id).await, Some(2));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_dispatchers_working_one_organization_at_once_never_share_a_number() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        dispatch(&pool).await;
        let org_id = seed_organization(&pool).await;
        let workflow_id = seed_workflow(&pool, org_id).await;
        seed_workflow_subscription(&pool, org_id, workflow_id, "t1", &["quote.accepted"], true)
            .await;
        for _ in 0..4 {
            seed_event(&pool, org_id, Actor::system()).await;
        }

        let one_event_at_a_time = |pool: PgPool| async move {
            for _ in 0..2 {
                with_tx(&pool, async |tx| {
                    let mut repo = PgEventDispatchRepository::new(&tx);
                    repo.dispatch_pending(1).await
                })
                .await
                .unwrap();
            }
        };
        tokio::join!(
            one_event_at_a_time(pool.clone()),
            one_event_at_a_time(pool.clone())
        );

        assert_eq!(numbers_of(&pool, org_id).await, vec![1, 2, 3, 4]);
        assert_eq!(counter_of(&pool, org_id).await, Some(4));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_nobody_wants_leaves_the_counter_untouched() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        dispatch(&pool).await;
        let org_id = seed_organization(&pool).await;
        seed_event(&pool, org_id, Actor::system()).await;

        dispatch(&pool).await;

        assert_eq!(counter_of(&pool, org_id).await, None);
    }

    #[test]
    fn the_default_outcome_creates_no_run() {
        assert_eq!(DispatchOutcome::default().runs, 0);
    }
}

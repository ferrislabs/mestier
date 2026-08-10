use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::repository;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::RunRepository,
        run::{DueRun, Run, RunSettlement, RunStatus, RunStep},
        workflow::Graph,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

/// The bootstrapping step every run is created with: not a real connector,
/// only a place to durably record the trigger payload so every real
/// connector's expressions can read `trigger.*` the same way whether the run
/// is fresh or resumed. Never looked up in the connector catalogue, never
/// placed by the editor — an internal convention of this module alone.
const TRIGGER_STEP_CONNECTOR_ID: &str = "$trigger";

#[repository(domain = Run, backend = Postgres)]
pub struct PgRunRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgRunRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct RunRow {
    id: Uuid,
    org_id: Uuid,
    workflow_id: Uuid,
    workflow_version_id: Uuid,
    trigger_event_id: Option<Uuid>,
    status: String,
    error: Option<String>,
    next_attempt_at: Option<DateTime<Utc>>,
    locked_at: Option<DateTime<Utc>>,
    locked_by: Option<String>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<RunRow> for Run {
    type Error = CoreError;

    fn try_from(row: RunRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            workflow_id: row.workflow_id,
            workflow_version_id: row.workflow_version_id,
            trigger_event_id: row.trigger_event_id,
            status: row.status.parse().map_err(CoreError::Internal)?,
            error: row.error,
            next_attempt_at: row.next_attempt_at,
            locked_at: row.locked_at,
            locked_by: row.locked_by,
            started_at: row.started_at,
            finished_at: row.finished_at,
            created_at: row.created_at,
        })
    }
}

struct RunStepRow {
    id: Uuid,
    run_id: Uuid,
    connector_id: String,
    iteration_path: String,
    attempts: i32,
    status: String,
    input: Option<Value>,
    output: Option<Value>,
    error: Option<String>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<RunStepRow> for RunStep {
    type Error = CoreError;

    fn try_from(row: RunStepRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            run_id: row.run_id,
            connector_id: row.connector_id,
            iteration_path: row.iteration_path,
            attempts: u32::try_from(row.attempts)
                .map_err(|_| CoreError::Internal("run step attempts went negative".to_owned()))?,
            status: row.status.parse().map_err(CoreError::Internal)?,
            input: row.input,
            output: row.output,
            error: row.error,
            started_at: row.started_at,
            finished_at: row.finished_at,
            created_at: row.created_at,
        })
    }
}

impl<'tx> RunRepository for PgRunRepository<'tx> {
    async fn create_run(&mut self, run: &Run, trigger_payload: Value) -> Result<Run, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query_as!(
            RunRow,
            r#"INSERT INTO automation.run
                   (id, org_id, workflow_id, workflow_version_id, trigger_event_id, status,
                    error, next_attempt_at, locked_at, locked_by, started_at, finished_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
               RETURNING id, org_id, workflow_id, workflow_version_id, trigger_event_id, status,
                         error, next_attempt_at, locked_at, locked_by, started_at, finished_at, created_at"#,
            run.id,
            run.org_id.0,
            run.workflow_id,
            run.workflow_version_id,
            run.trigger_event_id,
            run.status.as_str(),
            run.error,
            run.next_attempt_at,
            run.locked_at,
            run.locked_by,
            run.started_at,
            run.finished_at,
            run.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let now = Utc::now();
        sqlx::query!(
            r#"INSERT INTO automation.run_step
                   (id, run_id, connector_id, iteration_path, attempts, status, input, output,
                    error, started_at, finished_at, created_at)
               VALUES ($1, $2, $3, '', 0, 'succeeded', NULL, $4, NULL, $5, $5, $5)"#,
            generate_uuid_v7(),
            row.id,
            TRIGGER_STEP_CONNECTOR_ID,
            trigger_payload,
            now,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn claim_due(
        &mut self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> Result<Vec<DueRun>, CoreError> {
        let mut tx = self.tx.lock().await;

        // Same shape as `PgDeliveryRepository::claim_due`: a window function
        // for the per-organization quota cannot be combined with
        // `FOR UPDATE`, so the lock is taken in a second pass keyed by id,
        // and `SKIP LOCKED` means a row another worker already claimed is
        // simply skipped rather than blocked on.
        let rows = sqlx::query!(
            r#"
            WITH ranked AS (
                SELECT id,
                       row_number() OVER (PARTITION BY org_id ORDER BY next_attempt_at) AS rank
                FROM automation.run
                WHERE status = 'pending' AND next_attempt_at <= now()
            ),
            due AS (
                SELECT r.id
                FROM automation.run r
                JOIN ranked rk ON rk.id = r.id
                WHERE rk.rank <= $3
                  AND r.status = 'pending'
                  AND r.next_attempt_at <= now()
                ORDER BY r.next_attempt_at
                LIMIT $2
                FOR UPDATE OF r SKIP LOCKED
            )
            UPDATE automation.run r
            SET status = 'running',
                locked_at = now(),
                locked_by = $1,
                started_at = COALESCE(r.started_at, now())
            FROM due
            WHERE r.id = due.id
            RETURNING r.id, r.org_id, r.workflow_id, r.workflow_version_id, r.trigger_event_id
            "#,
            worker,
            batch,
            per_org,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| DueRun {
                id: row.id,
                org_id: OrganizationId(row.org_id),
                workflow_id: row.workflow_id,
                workflow_version_id: row.workflow_version_id,
                trigger_event_id: row.trigger_event_id,
            })
            .collect())
    }

    async fn steps_for_run(&mut self, run_id: Uuid) -> Result<Vec<RunStep>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query_as!(
            RunStepRow,
            r#"SELECT id, run_id, connector_id, iteration_path, attempts, status, input, output,
                      error, started_at, finished_at, created_at
               FROM automation.run_step
               WHERE run_id = $1"#,
            run_id,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(RunStep::try_from).collect()
    }

    async fn graph_for_version(
        &mut self,
        org_id: OrganizationId,
        workflow_version_id: Uuid,
    ) -> Result<Option<Graph>, CoreError> {
        let mut tx = self.tx.lock().await;

        let graph_json = sqlx::query_scalar!(
            r#"SELECT wv.graph
               FROM automation.workflow_version wv
               JOIN automation.workflow w ON w.id = wv.workflow_id
               WHERE w.org_id = $1 AND wv.id = $2"#,
            org_id.0,
            workflow_version_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        graph_json
            .map(|json| {
                serde_json::from_value(json).map_err(|e| {
                    CoreError::Internal(format!("invalid workflow graph in database: {e}"))
                })
            })
            .transpose()
    }

    async fn upsert_step(&mut self, step: &RunStep) -> Result<RunStep, CoreError> {
        let mut tx = self.tx.lock().await;

        // `id` is deliberately not part of the conflict target: a caller
        // never needs to look an existing row's id up before writing to it,
        // it can always pass a fresh one and let this fall back to updating
        // the row the unique constraint already identifies.
        let row = sqlx::query_as!(
            RunStepRow,
            r#"INSERT INTO automation.run_step
                   (id, run_id, connector_id, iteration_path, attempts, status, input, output,
                    error, started_at, finished_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
               ON CONFLICT (run_id, connector_id, iteration_path) DO UPDATE
               SET attempts = EXCLUDED.attempts,
                   status = EXCLUDED.status,
                   input = COALESCE(EXCLUDED.input, automation.run_step.input),
                   output = EXCLUDED.output,
                   error = EXCLUDED.error,
                   started_at = COALESCE(automation.run_step.started_at, EXCLUDED.started_at),
                   finished_at = EXCLUDED.finished_at
               RETURNING id, run_id, connector_id, iteration_path, attempts, status, input, output,
                         error, started_at, finished_at, created_at"#,
            step.id,
            step.run_id,
            step.connector_id,
            step.iteration_path,
            i32::try_from(step.attempts)
                .map_err(|_| CoreError::Internal("run step attempts is out of range".to_owned()))?,
            step.status.as_str(),
            step.input,
            step.output,
            step.error,
            step.started_at,
            step.finished_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn settle_run(
        &mut self,
        run_id: Uuid,
        settlement: RunSettlement,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        match settlement {
            RunSettlement::Succeeded => {
                sqlx::query!(
                    r#"UPDATE automation.run
                       SET status = $2, error = NULL, next_attempt_at = NULL,
                           locked_at = NULL, locked_by = NULL, finished_at = now()
                       WHERE id = $1"#,
                    run_id,
                    RunStatus::Succeeded.as_str(),
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
            RunSettlement::Failed { error } => {
                sqlx::query!(
                    r#"UPDATE automation.run
                       SET status = $2, error = $3, next_attempt_at = NULL,
                           locked_at = NULL, locked_by = NULL, finished_at = now()
                       WHERE id = $1"#,
                    run_id,
                    RunStatus::Failed.as_str(),
                    error,
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
            RunSettlement::Rescheduled { next_attempt_at } => {
                sqlx::query!(
                    r#"UPDATE automation.run
                       SET status = $2, error = NULL, next_attempt_at = $3,
                           locked_at = NULL, locked_by = NULL
                       WHERE id = $1"#,
                    run_id,
                    RunStatus::Pending.as_str(),
                    next_attempt_at,
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
        }

        Ok(())
    }

    async fn release_stale_claims(&mut self, older_than: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;

        // A run whose worker died in flight comes back `pending`, immediately
        // due — the same choice `PgDeliveryRepository::release_stale_claims`
        // makes: at-least-once already allows a step to be re-attempted, and
        // the engine never replays a step that already succeeded.
        let released = sqlx::query!(
            r#"UPDATE automation.run
               SET status = 'pending', next_attempt_at = now(), locked_at = NULL, locked_by = NULL
               WHERE status = 'running' AND locked_at < $1"#,
            older_than,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(released)
    }
}

/// `claim_due` and `release_stale_claims` scan `automation.run` with no
/// per-organization filter (only a per-organization *quota* once a row is
/// already a candidate) — the same shape as
/// `PgDeliveryRepository::claim_due`. A test that creates a `pending` run
/// and only later checks on it can otherwise have it claimed out from under
/// it by another test's concurrent claim. Mirrors `dispatcher::DISPATCH_LOCK`.
#[cfg(test)]
pub(crate) static RUN_CLAIM_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
mod tests {
    use chrono::Duration as ChronoDuration;
    use common::generate_uuid_v7;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::domain::automation::ports::WorkflowRepository;
    use crate::domain::automation::run::StepStatus;
    use crate::domain::automation::workflow::{Graph, PlacedConnector, Workflow};
    use crate::infrastructure::automation::postgres::PgWorkflowRepository;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run run-repository integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
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
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    /// A workflow with one saved version, so tests have a real
    /// `workflow_version_id` to pin a run to — the FK `automation.run`
    /// carries requires one.
    async fn seed_workflow_version(
        pool: &PgPool,
        org_id: OrganizationId,
        graph: Graph,
    ) -> (Uuid, Uuid) {
        with_tx(pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let now = Utc::now();
            let workflow = repo
                .insert(&Workflow {
                    id: generate_uuid_v7(),
                    org_id,
                    name: "Test workflow".to_string(),
                    description: None,
                    enabled: true,
                    current_version_id: None,
                    created_at: now,
                    updated_at: now,
                })
                .await?;
            let version = repo
                .insert_version(org_id, workflow.id, &graph, None)
                .await?;
            Ok::<_, CoreError>((workflow.id, version.id))
        })
        .await
        .unwrap()
    }

    fn simple_graph() -> Graph {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
            }],
            edges: Vec::new(),
        }
    }

    fn pending_run(org_id: OrganizationId, workflow_id: Uuid, workflow_version_id: Uuid) -> Run {
        let now = Utc::now();
        Run {
            id: generate_uuid_v7(),
            org_id,
            workflow_id,
            workflow_version_id,
            trigger_event_id: None,
            status: RunStatus::Pending,
            error: None,
            next_attempt_at: Some(now),
            locked_at: None,
            locked_by: None,
            started_at: None,
            finished_at: None,
            created_at: now,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_run_also_persists_its_trigger_step() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        let trigger_payload = json!({ "quote_id": "q-1" });

        let created = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, trigger_payload.clone()).await
        })
        .await
        .unwrap();

        assert_eq!(created, run);

        let steps = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run(created.id).await
        })
        .await
        .unwrap();

        assert_eq!(steps.len(), 1, "only the trigger step exists so far");
        assert_eq!(steps[0].connector_id, "$trigger");
        assert_eq!(steps[0].iteration_path, "");
        assert_eq!(steps[0].status, StepStatus::Succeeded);
        assert_eq!(steps[0].output, Some(trigger_payload));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_due_pending_run_is_claimed_and_marked_running() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "claim").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        let claimed = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-1", 100, 100).await
        })
        .await
        .unwrap();

        let mine = claimed
            .iter()
            .find(|r| r.id == run.id)
            .expect("our run was claimed");
        assert_eq!(mine.org_id, org_id);
        assert_eq!(mine.workflow_id, workflow_id);
        assert_eq!(mine.workflow_version_id, version_id);

        let status = sqlx::query_scalar!("SELECT status FROM automation.run WHERE id = $1", run.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "running");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_run_not_yet_due_is_left_alone() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "not-due").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let mut run = pending_run(org_id, workflow_id, version_id);
        run.next_attempt_at = Some(Utc::now() + ChronoDuration::hours(1));
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        let claimed = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-1", 100, 100).await
        })
        .await
        .unwrap();

        assert!(claimed.iter().all(|r| r.id != run.id));
    }

    /// One noisy organization must not take a whole batch — the same
    /// per-organization quota `PgDeliveryRepository::claim_due` enforces.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_organization_cannot_take_the_whole_batch() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let noisy_org = seed_organization(&pool, "noisy").await;
        let quiet_org = seed_organization(&pool, "quiet").await;
        let (noisy_workflow, noisy_version) =
            seed_workflow_version(&pool, noisy_org, simple_graph()).await;
        let (quiet_workflow, quiet_version) =
            seed_workflow_version(&pool, quiet_org, simple_graph()).await;

        for _ in 0..5 {
            let run = pending_run(noisy_org, noisy_workflow, noisy_version);
            with_tx(&pool, async |tx| {
                let mut repo = PgRunRepository::new(&tx);
                repo.create_run(&run, json!({})).await
            })
            .await
            .unwrap();
        }
        let quiet_run = pending_run(quiet_org, quiet_workflow, quiet_version);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&quiet_run, json!({})).await
        })
        .await
        .unwrap();

        let claimed = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-1", 100, 2).await
        })
        .await
        .unwrap();

        assert_eq!(claimed.iter().filter(|r| r.org_id == noisy_org).count(), 2);
        assert_eq!(claimed.iter().filter(|r| r.org_id == quiet_org).count(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn graph_for_version_returns_the_pinned_graph() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "graph").await;
        let graph = simple_graph();
        let (_, version_id) = seed_workflow_version(&pool, org_id, graph.clone()).await;

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.graph_for_version(org_id, version_id).await
        })
        .await
        .unwrap();

        assert_eq!(found, Some(graph));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn graph_for_version_from_another_organization_is_absent() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "graph-owner").await;
        let stranger_org = seed_organization(&pool, "graph-stranger").await;
        let (_, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.graph_for_version(stranger_org, version_id).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    fn in_flight_step(run_id: Uuid, connector_id: &str, iteration_path: &str) -> RunStep {
        let now = Utc::now();
        RunStep {
            id: generate_uuid_v7(),
            run_id,
            connector_id: connector_id.to_string(),
            iteration_path: iteration_path.to_string(),
            attempts: 1,
            status: StepStatus::InFlight,
            input: Some(json!({ "name": "brioche" })),
            output: None,
            error: None,
            started_at: Some(now),
            finished_at: None,
            created_at: now,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upserting_a_new_step_inserts_it() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "upsert-insert").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        let step = in_flight_step(run.id, "c2", "loop1[3]");
        let written = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.upsert_step(&step).await
        })
        .await
        .unwrap();

        assert_eq!(written.connector_id, "c2");
        assert_eq!(written.iteration_path, "loop1[3]");
        assert_eq!(written.status, StepStatus::InFlight);
        assert_eq!(written.attempts, 1);
    }

    /// The acceptance criterion the whole port rests on: retrying a step
    /// updates the one row the unique constraint already identifies, rather
    /// than writing a second one for the same connector and iteration.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upserting_the_same_connector_and_iteration_again_updates_the_same_row() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "upsert-update").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        let first_attempt = in_flight_step(run.id, "c2", "loop1[3]");
        let first_written = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.upsert_step(&first_attempt).await
        })
        .await
        .unwrap();

        // A retry: same connector, same iteration, a fresh row id and a
        // higher attempt count — as the engine would build it after a
        // failure.
        let mut retry = in_flight_step(run.id, "c2", "loop1[3]");
        retry.id = generate_uuid_v7();
        retry.attempts = 2;
        let retried = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.upsert_step(&retry).await
        })
        .await
        .unwrap();

        assert_eq!(
            retried.id, first_written.id,
            "the existing row is updated, not duplicated"
        );
        assert_eq!(retried.attempts, 2);

        let steps = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run(run.id).await
        })
        .await
        .unwrap();
        assert_eq!(
            steps.iter().filter(|s| s.connector_id == "c2").count(),
            1,
            "one row per (connector, iteration), never two"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn settling_a_run_as_succeeded_finishes_it() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settle-succeeded").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.settle_run(run.id, RunSettlement::Succeeded).await
        })
        .await
        .unwrap();

        let row = sqlx::query!(
            "SELECT status, finished_at FROM automation.run WHERE id = $1",
            run.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "succeeded");
        assert!(row.finished_at.is_some());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn settling_a_run_as_failed_records_the_error_and_stops_retrying() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settle-failed").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.settle_run(
                run.id,
                RunSettlement::Failed {
                    error: "the retry schedule ran out".to_string(),
                },
            )
            .await
        })
        .await
        .unwrap();

        let row = sqlx::query!(
            "SELECT status, error, next_attempt_at FROM automation.run WHERE id = $1",
            run.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "failed");
        assert_eq!(row.error.as_deref(), Some("the retry schedule ran out"));
        assert!(row.next_attempt_at.is_none());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn settling_a_run_as_rescheduled_unlocks_it_for_its_next_attempt() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settle-rescheduled").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-1", 100, 100).await
        })
        .await
        .unwrap();

        let next_attempt_at = Utc::now() + ChronoDuration::seconds(30);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.settle_run(run.id, RunSettlement::Rescheduled { next_attempt_at })
                .await
        })
        .await
        .unwrap();

        let row = sqlx::query!(
            "SELECT status, locked_at, locked_by FROM automation.run WHERE id = $1",
            run.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "pending");
        assert!(row.locked_at.is_none());
        assert!(row.locked_by.is_none());
    }

    /// A worker that dies mid-flight must not strand its claim: the run has
    /// to come back rather than sit `running` forever.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_lost_workers_claim_comes_back() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "stale-claim").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run, json!({})).await
        })
        .await
        .unwrap();
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-doomed", 100, 100).await
        })
        .await
        .unwrap();
        sqlx::query!(
            "UPDATE automation.run SET locked_at = now() - interval '1 hour' WHERE id = $1",
            run.id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let released = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.release_stale_claims(Utc::now() - ChronoDuration::minutes(5))
                .await
        })
        .await
        .unwrap();
        assert!(released >= 1);

        let claimable = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-2", 100, 100).await
        })
        .await
        .unwrap();
        assert!(
            claimable.iter().any(|r| r.id == run.id),
            "the released run is claimable again"
        );
    }
}

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
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
    trigger_payload: Option<Value>,
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
            trigger_payload: row.trigger_payload,
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
    async fn create_run(&mut self, run: &Run) -> Result<Run, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query_as!(
            RunRow,
            r#"INSERT INTO automation.run
                   (id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                    trigger_payload, status, error, next_attempt_at, locked_at, locked_by,
                    started_at, finished_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                         trigger_payload, status, error, next_attempt_at, locked_at, locked_by,
                         started_at, finished_at, created_at"#,
            run.id,
            run.org_id.0,
            run.workflow_id,
            run.workflow_version_id,
            run.trigger_event_id,
            run.trigger_payload,
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
            RETURNING r.id, r.org_id, r.workflow_id, r.workflow_version_id, r.trigger_event_id,
                      r.trigger_payload
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
                trigger_payload: row.trigger_payload,
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

    async fn steps_for_run_scoped(
        &mut self,
        org_id: OrganizationId,
        run_id: Uuid,
    ) -> Result<Vec<RunStep>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query_as!(
            RunStepRow,
            r#"SELECT rs.id, rs.run_id, rs.connector_id, rs.iteration_path, rs.attempts,
                      rs.status, rs.input, rs.output, rs.error, rs.started_at, rs.finished_at,
                      rs.created_at
               FROM automation.run_step rs
               JOIN automation.run r ON r.id = rs.run_id
               WHERE r.org_id = $1 AND rs.run_id = $2"#,
            org_id.0,
            run_id,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(RunStep::try_from).collect()
    }

    async fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        run_id: Uuid,
    ) -> Result<Option<Run>, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query_as!(
            RunRow,
            r#"SELECT id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                      trigger_payload, status, error, next_attempt_at, locked_at, locked_by,
                      started_at, finished_at, created_at
               FROM automation.run WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            run_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Run::try_from).transpose()
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<Run>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query_as!(
            RunRow,
            r#"SELECT id, org_id, workflow_id, workflow_version_id, trigger_event_id,
                      trigger_payload, status, error, next_attempt_at, locked_at, locked_by,
                      started_at, finished_at, created_at
               FROM automation.run
               WHERE org_id = $1
               ORDER BY created_at DESC"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Run::try_from).collect()
    }

    async fn delete_steps(
        &mut self,
        run_id: Uuid,
        connector_ids: &[String],
    ) -> Result<u64, CoreError> {
        if connector_ids.is_empty() {
            return Ok(0);
        }

        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            r#"DELETE FROM automation.run_step
               WHERE run_id = $1 AND connector_id = ANY($2::text[])"#,
            run_id,
            connector_ids,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(deleted)
    }

    async fn requeue_for_replay(&mut self, run_id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"UPDATE automation.run
               SET status = 'pending', error = NULL, next_attempt_at = now(),
                   locked_at = NULL, locked_by = NULL, finished_at = NULL
               WHERE id = $1"#,
            run_id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
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

    async fn purge_succeeded(
        &mut self,
        org_id: OrganizationId,
        older_than: DateTime<Utc>,
        batch: i64,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;

        // `status = 'succeeded'` excludes every failed, cancelled, pending or
        // running run unconditionally — see the port doc for why a failed
        // run must survive this pass. `run_step` follows through its
        // `ON DELETE CASCADE`. Same bounded-batch shape as
        // `PgEventLogRepository::purge_expired`.
        let deleted = sqlx::query!(
            r#"
            WITH victims AS (
                SELECT id FROM automation.run
                WHERE org_id = $1 AND status = 'succeeded' AND finished_at < $2
                ORDER BY finished_at
                LIMIT $3
            )
            DELETE FROM automation.run rn
            USING victims v
            WHERE rn.id = v.id
            "#,
            org_id.0,
            older_than,
            batch,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(deleted)
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
            trigger_payload: None,
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
    async fn creating_a_run_persists_its_trigger_payload_and_starts_with_no_step() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let trigger_payload = json!({ "quote_id": "q-1" });
        let run = Run {
            trigger_payload: Some(trigger_payload.clone()),
            ..pending_run(org_id, workflow_id, version_id)
        };

        let created = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await
        })
        .await
        .unwrap();

        assert_eq!(created, run);
        assert_eq!(created.trigger_payload, Some(trigger_payload));

        let steps = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run(created.id).await
        })
        .await
        .unwrap();

        assert!(
            steps.is_empty(),
            "a run has no step until the engine executes a real connector: {steps:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_run_with_no_trigger_payload_creates_with_none() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-no-payload").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);

        let created = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await
        })
        .await
        .unwrap();

        assert_eq!(created.trigger_payload, None);
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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
                repo.create_run(&run).await
            })
            .await
            .unwrap();
        }
        let quiet_run = pending_run(quiet_org, quiet_workflow, quiet_version);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&quiet_run).await
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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
            repo.create_run(&run).await
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

    fn finished_run(
        org_id: OrganizationId,
        workflow_id: Uuid,
        workflow_version_id: Uuid,
        status: RunStatus,
        finished_at: DateTime<Utc>,
    ) -> Run {
        Run {
            status,
            next_attempt_at: None,
            started_at: Some(finished_at),
            finished_at: Some(finished_at),
            ..pending_run(org_id, workflow_id, workflow_version_id)
        }
    }

    // --- find_by_id / list_by_organization ------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_by_id_returns_a_run_in_its_own_organization() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-owner").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(org_id, run.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, Some(run));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_by_id_from_another_organization_reads_back_absent() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-real-owner").await;
        let stranger_org = seed_organization(&pool, "find-stranger").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(stranger_org, run.id).await
        })
        .await
        .unwrap();

        assert_eq!(
            found, None,
            "a run id from another organization must read back as absent, not forbidden"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_by_organization_only_returns_this_organizations_runs_most_recent_first() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list-owner").await;
        let other_org = seed_organization(&pool, "list-other").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let (other_workflow, other_version) =
            seed_workflow_version(&pool, other_org, simple_graph()).await;
        let first = pending_run(org_id, workflow_id, version_id);
        let second = pending_run(org_id, workflow_id, version_id);
        let not_mine = pending_run(other_org, other_workflow, other_version);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&first).await?;
            repo.create_run(&second).await?;
            repo.create_run(&not_mine).await
        })
        .await
        .unwrap();

        let listed = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.list_by_organization(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|r| r.org_id == org_id));
    }

    // --- steps_for_run_scoped / delete_steps / requeue_for_replay ------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn steps_for_run_scoped_returns_nothing_for_another_organizations_run() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "steps-owner").await;
        let stranger_org = seed_organization(&pool, "steps-stranger").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await?;
            repo.upsert_step(&in_flight_step(run.id, "c1", "")).await
        })
        .await
        .unwrap();

        let mine = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run_scoped(org_id, run.id).await
        })
        .await
        .unwrap();
        assert_eq!(mine.len(), 1);

        let stranger_view = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run_scoped(stranger_org, run.id).await
        })
        .await
        .unwrap();
        assert!(
            stranger_view.is_empty(),
            "a run id scoped to another organization must expose no step"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn delete_steps_removes_every_iteration_of_the_named_connectors_only() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-steps").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = pending_run(org_id, workflow_id, version_id);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await?;
            repo.upsert_step(&in_flight_step(run.id, "loop_body", "loop1[0]"))
                .await?;
            repo.upsert_step(&in_flight_step(run.id, "loop_body", "loop1[1]"))
                .await?;
            repo.upsert_step(&in_flight_step(run.id, "unrelated", ""))
                .await
        })
        .await
        .unwrap();

        let deleted = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.delete_steps(run.id, &["loop_body".to_string()]).await
        })
        .await
        .unwrap();
        assert_eq!(deleted, 2, "both iterations of the named connector");

        let remaining = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.steps_for_run(run.id).await
        })
        .await
        .unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].connector_id, "unrelated");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn requeue_for_replay_puts_a_finished_run_back_in_the_claimable_queue() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "requeue").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let run = finished_run(
            org_id,
            workflow_id,
            version_id,
            RunStatus::Succeeded,
            Utc::now(),
        );
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.requeue_for_replay(run.id).await
        })
        .await
        .unwrap();

        let row = sqlx::query!(
            "SELECT status, finished_at, next_attempt_at FROM automation.run WHERE id = $1",
            run.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "pending");
        assert!(row.finished_at.is_none());
        assert!(row.next_attempt_at.is_some());

        let claimable = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.claim_due("worker-1", 100, 100).await
        })
        .await
        .unwrap();
        assert!(claimable.iter().any(|r| r.id == run.id));
    }

    // --- purge_succeeded --------------------------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_succeeded_deletes_succeeded_runs_older_than_the_cutoff_and_keeps_the_rest() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "purge-succeeded").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let cutoff = Utc::now() - ChronoDuration::days(30);
        let expired = finished_run(
            org_id,
            workflow_id,
            version_id,
            RunStatus::Succeeded,
            cutoff - ChronoDuration::days(1),
        );
        let fresh = finished_run(
            org_id,
            workflow_id,
            version_id,
            RunStatus::Succeeded,
            cutoff + ChronoDuration::days(1),
        );
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&expired).await?;
            repo.create_run(&fresh).await
        })
        .await
        .unwrap();

        let deleted = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, cutoff, 100).await
        })
        .await
        .unwrap();

        assert_eq!(deleted, 1);
        let remaining: Vec<Uuid> =
            sqlx::query_scalar!("SELECT id FROM automation.run WHERE org_id = $1", org_id.0,)
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(remaining, vec![fresh.id]);
    }

    /// The acceptance criterion the whole purge exists to uphold: a failed
    /// run is never a candidate, however old — it is what an operator reads
    /// to understand why an automation stopped working.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_succeeded_never_purges_a_failed_run() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "purge-keeps-failed").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let ancient = Utc::now() - ChronoDuration::days(3650);
        let failed = finished_run(org_id, workflow_id, version_id, RunStatus::Failed, ancient);
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&failed).await
        })
        .await
        .unwrap();

        let deleted = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, Utc::now(), 100).await
        })
        .await
        .unwrap();

        assert_eq!(deleted, 0);
        let still_there = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(org_id, failed.id).await
        })
        .await
        .unwrap();
        assert!(
            still_there.is_some(),
            "a failed run must survive the succeeded-only purge"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_succeeded_cascades_to_its_steps() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "purge-cascade").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let ancient = Utc::now() - ChronoDuration::days(3650);
        let run = finished_run(
            org_id,
            workflow_id,
            version_id,
            RunStatus::Succeeded,
            ancient,
        );
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&run).await?;
            repo.upsert_step(&in_flight_step(run.id, "c1", "")).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, Utc::now(), 100).await
        })
        .await
        .unwrap();

        let step_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.run_step WHERE run_id = $1",
            run.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            step_count,
            Some(0),
            "run_step must follow its run away via the existing CASCADE"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_succeeded_only_touches_the_given_organizations_runs() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "purge-scoped-owner").await;
        let other_org = seed_organization(&pool, "purge-scoped-other").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let (other_workflow, other_version) =
            seed_workflow_version(&pool, other_org, simple_graph()).await;
        let ancient = Utc::now() - ChronoDuration::days(3650);
        let mine = finished_run(
            org_id,
            workflow_id,
            version_id,
            RunStatus::Succeeded,
            ancient,
        );
        let not_mine = finished_run(
            other_org,
            other_workflow,
            other_version,
            RunStatus::Succeeded,
            ancient,
        );
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.create_run(&mine).await?;
            repo.create_run(&not_mine).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, Utc::now(), 100).await
        })
        .await
        .unwrap();

        let not_mine_still_there = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(other_org, not_mine.id).await
        })
        .await
        .unwrap();
        assert!(
            not_mine_still_there.is_some(),
            "another organization's succeeded run must survive a purge scoped to this one"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_succeeded_never_deletes_more_than_the_batch_in_one_call() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "purge-batch").await;
        let (workflow_id, version_id) = seed_workflow_version(&pool, org_id, simple_graph()).await;
        let ancient = Utc::now() - ChronoDuration::days(3650);
        let runs = [
            finished_run(
                org_id,
                workflow_id,
                version_id,
                RunStatus::Succeeded,
                ancient,
            ),
            finished_run(
                org_id,
                workflow_id,
                version_id,
                RunStatus::Succeeded,
                ancient,
            ),
            finished_run(
                org_id,
                workflow_id,
                version_id,
                RunStatus::Succeeded,
                ancient,
            ),
        ];
        with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            for run in &runs {
                repo.create_run(run).await?;
            }
            Ok::<_, CoreError>(())
        })
        .await
        .unwrap();

        let deleted_first_pass = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, Utc::now(), 2).await
        })
        .await
        .unwrap();
        assert_eq!(deleted_first_pass, 2, "bounded to the batch size");

        let deleted_second_pass = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.purge_succeeded(org_id, Utc::now(), 2).await
        })
        .await
        .unwrap();
        assert_eq!(
            deleted_second_pass, 1,
            "the next pass finishes what the first left behind"
        );
    }
}

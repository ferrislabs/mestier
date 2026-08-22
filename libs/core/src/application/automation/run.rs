use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        expression::ExpressionContext,
        ports::{AutomationSettingsRepository, RunRepository, WorkflowRepository},
        run::{
            ConnectorInput, ConnectorOutcome, DueRun, LoopStackEntry, Run, RunSettlement,
            RunStatus, RunStep, StepStatus, branch_targets, connectors_by_id, descendants_via,
            format_iteration_path, resolve_config, roots,
        },
        settings::with_jitter,
        workflow::{Branch, Graph, PlacedConnector},
    },
    infrastructure::automation::{connectors::ConnectorRegistry, worker::WorkerSchedule},
};

/// What one engine pass did, across every run it claimed — the run-engine
/// analogue of `application::automation::PassOutcome` for deliveries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnginePassOutcome {
    pub claimed: usize,
    pub succeeded: usize,
    /// Not finished, but made progress or is waiting on a retry — reclaimed
    /// on a later pass.
    pub rescheduled: usize,
    /// A step exhausted its retry schedule; the run is terminally failed.
    pub failed: usize,
}

/// One loop level a connector is currently nested inside, carried on the
/// walk stack so the innermost frame can build an
/// `ExpressionContext::loop_frame` and every frame can render its
/// `iteration_path`.
#[derive(Debug, Clone)]
struct LoopStep {
    loop_id: String,
    index: usize,
    item: Value,
}

/// One connector waiting to be visited, with the loop nesting that got the
/// walk there. A plain `Vec` used as a stack (push/pop from the end) gives a
/// depth-first order: a loop's whole per-item subtree finishes before the
/// next item starts, and the loop's `After` continuation — pushed *before*
/// any item frame — only pops once every item's subtree is exhausted.
#[derive(Debug, Clone)]
struct Frame {
    connector_id: String,
    path: Vec<LoopStep>,
}

impl Frame {
    fn iteration_path(&self) -> String {
        let entries: Vec<LoopStackEntry> = self
            .path
            .iter()
            .map(|step| (step.loop_id.clone(), step.index))
            .collect();
        format_iteration_path(&entries)
    }
}

/// What one slice of one run did, before it is settled.
enum SliceResult {
    Completed,
    BudgetExhausted,
    StepFailed {
        error: String,
        dead: bool,
        next_attempt_at: Option<DateTime<Utc>>,
    },
}

/// Rebuilds the [`ConnectorOutcome`] a settled step's persisted `output`
/// represents, without recomputing anything — the resumption discipline the
/// whole module rests on. The mapping is the mirror image of what
/// `process_run` writes for each kind: `flow.condition` derives `taken` from
/// `matched`, `flow.loop` reads `items` back, anything else is a plain
/// `Produced`.
fn reconstruct_outcome(kind: &str, output: &Value) -> ConnectorOutcome {
    match kind {
        "flow.condition" => {
            let matched = output
                .get("matched")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            ConnectorOutcome::Branch {
                taken: if matched { Branch::Then } else { Branch::Else },
                output: output.clone(),
            }
        }
        "flow.loop" => {
            let items = output
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            ConnectorOutcome::Iterate { items }
        }
        _ => ConnectorOutcome::Produced(output.clone()),
    }
}

impl MestierUseCase {
    /// Starts a run: pins the workflow's current version — a later edit
    /// moves `Workflow::current_version_id`, never this run's copy of it —
    /// and persists `trigger_payload` on the run itself.
    #[transactional(run, workflow)]
    pub async fn start_run(
        &self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        trigger_payload: Value,
    ) -> Result<Uuid, CoreError> {
        let mut workflows = workflow_repository;
        let mut runs = run_repository;

        let workflow = workflows
            .find_by_id(org_id, workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let workflow_version_id = workflow.current_version_id.ok_or_else(|| {
            CoreError::Conflict("the workflow has no saved version to run".to_string())
        })?;

        let now = Utc::now();
        let run = Run {
            id: generate_uuid_v7(),
            org_id,
            workflow_id,
            workflow_version_id,
            trigger_event_id: None,
            trigger_payload: Some(trigger_payload),
            status: RunStatus::Pending,
            error: None,
            next_attempt_at: Some(now),
            locked_at: None,
            locked_by: None,
            started_at: None,
            finished_at: None,
            created_at: now,
        };

        let created = runs.create_run(&run).await?;
        Ok(created.id)
    }

    #[transactional(run)]
    pub async fn find_run(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
    ) -> Result<Option<Run>, CoreError> {
        let mut runs = run_repository;
        runs.find_by_id(org_id, run_id).await
    }

    #[transactional(run)]
    pub async fn list_runs(&self, org_id: OrganizationId) -> Result<Vec<Run>, CoreError> {
        let mut runs = run_repository;
        runs.list_by_organization(org_id).await
    }

    /// A run's steps — `input`, `output`, `error`, `attempts` and
    /// `iteration_path` on each — for the run inspector. Scoped by `org_id`
    /// itself (`RunRepository::steps_for_run_scoped`), not merely gated by a
    /// prior call to [`Self::find_run`]: a defense-in-depth match for the
    /// discipline every other automation repository already keeps, that a
    /// cross-organization lookup reads back absent rather than trusting the
    /// caller to have checked.
    #[transactional(run)]
    pub async fn list_run_steps(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
    ) -> Result<Vec<RunStep>, CoreError> {
        let mut runs = run_repository;
        runs.steps_for_run_scoped(org_id, run_id).await
    }

    /// Replays a run starting at `connector_id`: everything from that step
    /// onward (`descendants_via`, branch-blind, the same reachability the
    /// engine's own `advance` uses) loses its recorded steps and is executed
    /// again on the next worker pass; everything upstream keeps its settled
    /// row and is read back, never re-executed — the same resumption
    /// discipline `process_run` already relies on, just seeded by a human
    /// choosing where to restart instead of a crash.
    ///
    /// Refused while the run is still `pending` or `running`: replaying a
    /// run the worker might be mid-slice on would race its own writes.
    #[transactional(run)]
    pub async fn replay_run_from_step(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
        connector_id: String,
    ) -> Result<Run, CoreError> {
        let mut runs = run_repository;

        let run = runs
            .find_by_id(org_id, run_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if matches!(run.status, RunStatus::Pending | RunStatus::Running) {
            return Err(CoreError::Conflict(
                "a run still in progress cannot be replayed".to_string(),
            ));
        }

        let graph = runs
            .graph_for_version(org_id, run.workflow_version_id)
            .await?
            .ok_or_else(|| {
                CoreError::Internal(
                    "a run points at a workflow version whose graph cannot be found".to_string(),
                )
            })?;
        let known_ids: std::collections::HashSet<&str> =
            graph.connectors.iter().map(|c| c.id.as_str()).collect();
        if !known_ids.contains(connector_id.as_str()) {
            return Err(CoreError::Conflict(format!(
                "unknown connector `{connector_id}` in this run's workflow version"
            )));
        }

        let to_reset: Vec<String> = descendants_via(&graph, &[connector_id.as_str()])
            .into_iter()
            .map(str::to_owned)
            .collect();
        runs.delete_steps(run_id, &to_reset).await?;
        runs.requeue_for_replay(run_id).await?;

        runs.find_by_id(org_id, run_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[transactional(run)]
    async fn claim_runs(
        &self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> Result<Vec<DueRun>, CoreError> {
        let mut repo = run_repository;
        repo.claim_due(worker, batch, per_org).await
    }

    #[transactional(run)]
    async fn load_run_state(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
        workflow_version_id: Uuid,
    ) -> Result<(Graph, Vec<RunStep>), CoreError> {
        let mut repo = run_repository;
        let graph = repo
            .graph_for_version(org_id, workflow_version_id)
            .await?
            .ok_or_else(|| {
                CoreError::Internal(
                    "a run points at a workflow version whose graph cannot be found".to_string(),
                )
            })?;
        let steps = repo.steps_for_run(run_id).await?;
        Ok((graph, steps))
    }

    #[transactional(run)]
    async fn write_step(&self, step: &RunStep) -> Result<RunStep, CoreError> {
        let mut repo = run_repository;
        repo.upsert_step(step).await
    }

    #[transactional(run)]
    async fn write_steps(&self, steps: &[RunStep]) -> Result<Vec<RunStep>, CoreError> {
        let mut repo = run_repository;
        let mut written = Vec::with_capacity(steps.len());
        for step in steps {
            written.push(repo.upsert_step(step).await?);
        }
        Ok(written)
    }

    #[transactional(run)]
    async fn settle_run_outcome(
        &self,
        run_id: Uuid,
        settlement: RunSettlement,
    ) -> Result<(), CoreError> {
        let mut repo = run_repository;
        repo.settle_run(run_id, settlement).await
    }

    /// Reuses `AutomationSettingsRepository::settings_for`: the retry
    /// schedule is an organization-wide automation setting, not specific to
    /// this engine, so nothing here needs its own copy of that query.
    #[transactional(automation_settings)]
    async fn retry_schedule_for(
        &self,
        org_id: OrganizationId,
    ) -> Result<crate::domain::automation::settings::AutomationSettings, CoreError> {
        let mut repo = automation_settings_repository;
        repo.settings_for(org_id).await
    }

    /// Brings back runs a worker claimed and never settled.
    #[transactional(run)]
    pub async fn recover_lost_runs(&self, older_than: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut repo = run_repository;
        repo.release_stale_claims(older_than).await
    }

    /// Runs one engine pass: claim what is due, work each run for at most
    /// one slice, settle it. Mirrors `run_delivery_pass`'s shape — claiming
    /// and every write are their own short transaction; connector execution
    /// (a use case call today, the network in #202) happens outside all of
    /// them.
    pub async fn run_engine_pass(
        &self,
        connectors: &ConnectorRegistry,
        worker: &str,
        schedule: WorkerSchedule,
    ) -> Result<EnginePassOutcome, CoreError> {
        let claimed = self
            .claim_runs(worker, schedule.batch, schedule.per_org)
            .await?;
        let mut outcome = EnginePassOutcome {
            claimed: claimed.len(),
            ..EnginePassOutcome::default()
        };

        for due in &claimed {
            match self.process_run(due, connectors, &schedule).await {
                Ok(SliceResult::Completed) => outcome.succeeded += 1,
                Ok(SliceResult::BudgetExhausted) => outcome.rescheduled += 1,
                Ok(SliceResult::StepFailed { dead: true, .. }) => outcome.failed += 1,
                Ok(SliceResult::StepFailed { dead: false, .. }) => outcome.rescheduled += 1,
                Err(error) => {
                    tracing::error!(%error, run_id = %due.id, "processing a run failed");
                }
            }
        }

        Ok(outcome)
    }

    /// Works one claimed run for a bounded slice — `schedule.max_steps_per_slice`
    /// connectors or `schedule.max_slice_duration`, whichever comes first —
    /// then settles it: finished, terminally failed, or left `pending` for a
    /// later pass to pick back up.
    ///
    /// Every pass re-walks the graph from its roots, using the run's
    /// persisted steps to skip anything already settled. That is what makes
    /// resuming free of special cases: a fresh run and a resumed one run the
    /// exact same code, and a step that already succeeded costs a hash-map
    /// lookup, never a re-execution.
    async fn process_run(
        &self,
        due: &DueRun,
        connectors: &ConnectorRegistry,
        schedule: &WorkerSchedule,
    ) -> Result<SliceResult, CoreError> {
        let (graph, steps) = self
            .load_run_state(due.org_id, due.id, due.workflow_version_id)
            .await?;

        let mut index: HashMap<(String, String), RunStep> = steps
            .into_iter()
            .map(|s| ((s.connector_id.clone(), s.iteration_path.clone()), s))
            .collect();

        let placed_by_id: HashMap<&str, &PlacedConnector> = connectors_by_id(&graph);
        let mut connector_outputs: BTreeMap<String, Value> = BTreeMap::new();

        let mut stack: Vec<Frame> = roots(&graph)
            .into_iter()
            .rev()
            .map(|id| Frame {
                connector_id: id.to_string(),
                path: Vec::new(),
            })
            .collect();

        let start = Instant::now();
        let mut steps_executed = 0usize;
        // Set the moment the walk must stop early; `None` once it drains the
        // whole stack means the run is done. Settling happens once, after
        // the loop, so every exit path — including the two `break`s below —
        // goes through the same place instead of each carrying its own call.
        let mut early_result: Option<SliceResult> = None;

        while let Some(frame) = stack.pop() {
            let iteration_path = frame.iteration_path();
            let key = (frame.connector_id.clone(), iteration_path.clone());

            // Only a *settled* step (succeeded, skipped, or permanently
            // dead) carries a trustworthy outcome to read back — see
            // `StepStatus::is_settled`. `Pending`/`InFlight`/`Failed` all
            // fall through to a fresh attempt below: `InFlight` is exactly a
            // worker that died mid-step, and there is no exactly-once here.
            if let Some(existing) = index.get(&key).cloned()
                && existing.status.is_settled()
            {
                match existing.status {
                    StepStatus::Succeeded => {
                        let output = existing.output.clone().unwrap_or(Value::Null);
                        connector_outputs.insert(frame.connector_id.clone(), output.clone());
                        let kind = placed_by_id
                            .get(frame.connector_id.as_str())
                            .map(|p| p.kind.as_str())
                            .unwrap_or_default();
                        let reconstructed = reconstruct_outcome(kind, &output);
                        self.advance(
                            due.id,
                            &graph,
                            &frame,
                            &reconstructed,
                            &mut stack,
                            &mut index,
                        )
                        .await?;
                        continue;
                    }
                    StepStatus::Skipped => continue,
                    StepStatus::Dead => {
                        early_result = Some(SliceResult::StepFailed {
                            error: existing.error.clone().unwrap_or_default(),
                            dead: true,
                            next_attempt_at: None,
                        });
                        break;
                    }
                    StepStatus::Pending | StepStatus::InFlight | StepStatus::Failed => {
                        unreachable!("is_settled() already excludes these statuses")
                    }
                }
            }

            if steps_executed >= schedule.max_steps_per_slice
                || start.elapsed() >= schedule.max_slice_duration
            {
                early_result = Some(SliceResult::BudgetExhausted);
                break;
            }

            let attempts_before = index.get(&key).map(|s| s.attempts).unwrap_or(0);
            let Some(placed) = placed_by_id.get(frame.connector_id.as_str()).copied() else {
                early_result = Some(SliceResult::StepFailed {
                    error: format!(
                        "the graph no longer has a connector `{}`",
                        frame.connector_id
                    ),
                    dead: true,
                    next_attempt_at: None,
                });
                break;
            };

            let loop_frame =
                frame
                    .path
                    .last()
                    .map(|step| crate::domain::automation::expression::LoopFrame {
                        item: &step.item,
                        index: step.index,
                    });
            let now = Utc::now();
            let ctx = ExpressionContext {
                trigger: due.trigger_payload.as_ref(),
                connectors: &connector_outputs,
                loop_frame,
                now,
            };
            let resolved = resolve_config(&placed.config, &ctx);

            let in_flight = RunStep {
                id: generate_uuid_v7(),
                run_id: due.id,
                connector_id: frame.connector_id.clone(),
                iteration_path: iteration_path.clone(),
                attempts: attempts_before + 1,
                status: StepStatus::InFlight,
                input: resolved
                    .as_ref()
                    .ok()
                    .map(|config| Value::Object(config.clone())),
                output: None,
                error: None,
                started_at: Some(now),
                finished_at: None,
                created_at: now,
            };
            let written = self.write_step(&in_flight).await?;
            steps_executed += 1;

            let outcome = match resolved {
                Err(error) => ConnectorOutcome::Failed {
                    error: error.to_string(),
                },
                Ok(config) => {
                    let input = ConnectorInput {
                        org_id: due.org_id,
                        run_id: due.id,
                        config: &config,
                        credential_id: placed.credential_id,
                    };
                    match connectors
                        .execute(&placed.kind, placed.version, input)
                        .await
                    {
                        Ok(outcome) => outcome,
                        Err(unknown) => ConnectorOutcome::Failed {
                            error: unknown.to_string(),
                        },
                    }
                }
            };

            match &outcome {
                ConnectorOutcome::Failed { error } => {
                    let settings = self.retry_schedule_for(due.org_id).await?;
                    let next_attempt_at = settings
                        .backoff_after(attempts_before)
                        .map(|interval| with_jitter(interval, written.id.as_u128() as u64))
                        .and_then(|interval| chrono::Duration::from_std(interval).ok())
                        .map(|interval| Utc::now() + interval);
                    let dead = next_attempt_at.is_none();

                    let settled = RunStep {
                        status: if dead {
                            StepStatus::Dead
                        } else {
                            StepStatus::Failed
                        },
                        error: Some(error.clone()),
                        finished_at: Some(Utc::now()),
                        ..written
                    };
                    self.write_step(&settled).await?;

                    early_result = Some(SliceResult::StepFailed {
                        error: error.clone(),
                        dead,
                        next_attempt_at,
                    });
                    break;
                }
                _ => {
                    let output = match &outcome {
                        ConnectorOutcome::Produced(output) => output.clone(),
                        ConnectorOutcome::Branch { output, .. } => output.clone(),
                        ConnectorOutcome::Iterate { items } => json!({ "items": items }),
                        ConnectorOutcome::Failed { .. } => unreachable!("handled above"),
                    };
                    let settled = RunStep {
                        status: StepStatus::Succeeded,
                        output: Some(output.clone()),
                        finished_at: Some(Utc::now()),
                        ..written
                    };
                    let settled = self.write_step(&settled).await?;
                    connector_outputs.insert(frame.connector_id.clone(), output);
                    index.insert(key, settled);

                    self.advance(due.id, &graph, &frame, &outcome, &mut stack, &mut index)
                        .await?;
                }
            }
        }

        let result = early_result.unwrap_or(SliceResult::Completed);
        let settlement = match &result {
            SliceResult::Completed => RunSettlement::Succeeded,
            SliceResult::BudgetExhausted => RunSettlement::Rescheduled {
                // Immediately due: a run that only ran out of budget, not
                // out of luck, is picked back up on the very next pass.
                next_attempt_at: Utc::now(),
            },
            SliceResult::StepFailed {
                dead: true, error, ..
            } => RunSettlement::Failed {
                error: error.clone(),
            },
            SliceResult::StepFailed {
                dead: false,
                next_attempt_at,
                ..
            } => RunSettlement::Rescheduled {
                next_attempt_at: next_attempt_at.unwrap_or_else(Utc::now),
            },
        };
        self.settle_run_outcome(due.id, settlement).await?;

        Ok(result)
    }

    /// Pushes what comes after a connector that just produced `outcome` —
    /// whether it was executed this instant or its outcome was reconstructed
    /// from a persisted, already-`succeeded` step. One shared path so a
    /// fresh run and a resumed one push exactly the same frames.
    async fn advance(
        &self,
        run_id: Uuid,
        graph: &Graph,
        frame: &Frame,
        outcome: &ConnectorOutcome,
        stack: &mut Vec<Frame>,
        index: &mut HashMap<(String, String), RunStep>,
    ) -> Result<(), CoreError> {
        match outcome {
            ConnectorOutcome::Produced(_) => {
                for target in branch_targets(graph, &frame.connector_id, None) {
                    stack.push(Frame {
                        connector_id: target.to_string(),
                        path: frame.path.clone(),
                    });
                }
            }
            ConnectorOutcome::Branch { taken, .. } => {
                let untaken = if *taken == Branch::Then {
                    Branch::Else
                } else {
                    Branch::Then
                };
                let taken_targets = branch_targets(graph, &frame.connector_id, Some(*taken));
                let untaken_targets = branch_targets(graph, &frame.connector_id, Some(untaken));
                let taken_reach = descendants_via(graph, &taken_targets);
                let untaken_reach = descendants_via(graph, &untaken_targets);

                let iteration_path = frame.iteration_path();
                let to_skip: Vec<&str> = untaken_reach
                    .difference(&taken_reach)
                    .copied()
                    .filter(|id| !index.contains_key(&(id.to_string(), iteration_path.clone())))
                    .collect();

                if !to_skip.is_empty() {
                    let now = Utc::now();
                    let skip_steps: Vec<RunStep> = to_skip
                        .iter()
                        .map(|id| RunStep {
                            id: generate_uuid_v7(),
                            run_id,
                            connector_id: id.to_string(),
                            iteration_path: iteration_path.clone(),
                            attempts: 0,
                            status: StepStatus::Skipped,
                            input: None,
                            output: None,
                            error: None,
                            started_at: Some(now),
                            finished_at: Some(now),
                            created_at: now,
                        })
                        .collect();
                    let written = self.write_steps(&skip_steps).await?;
                    for step in written {
                        index.insert(
                            (step.connector_id.clone(), step.iteration_path.clone()),
                            step,
                        );
                    }
                }

                for target in taken_targets {
                    stack.push(Frame {
                        connector_id: target.to_string(),
                        path: frame.path.clone(),
                    });
                }
            }
            ConnectorOutcome::Iterate { items } => {
                let each_targets = branch_targets(graph, &frame.connector_id, Some(Branch::Each));
                let after_targets = branch_targets(graph, &frame.connector_id, Some(Branch::After));

                // Pushed first, so it sits at the bottom of the stack:
                // it only pops once every item's subtree (pushed after,
                // therefore above it) has fully drained.
                for after in after_targets {
                    stack.push(Frame {
                        connector_id: after.to_string(),
                        path: frame.path.clone(),
                    });
                }
                for (idx, item) in items.iter().enumerate().rev() {
                    for target in &each_targets {
                        let mut path = frame.path.clone();
                        path.push(LoopStep {
                            loop_id: frame.connector_id.clone(),
                            index: idx,
                            item: item.clone(),
                        });
                        stack.push(Frame {
                            connector_id: (*target).to_string(),
                            path,
                        });
                    }
                }
            }
            ConnectorOutcome::Failed { .. } => unreachable!("a failed step never advances"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use common::generate_uuid_v7;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, Edge, SaveWorkflowVersionCommand,
    };
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
    use tokio::sync::OnceCell;

    use crate::application::test_support::scratch_pool;
    use crate::infrastructure::realtime::EventHub;

    /// A database of this suite's own, not the shared development one.
    ///
    /// Two reasons, both observed rather than anticipated.
    ///
    /// These tests drive the worker themselves, but `run_engine_pass` claims
    /// every *due* run in the database. Any other worker pointed at the same one
    /// — a dev API left running, most often — races them for their own runs and
    /// executes them with its own build, leaving a run `pending` and the
    /// assertion flapping: ten failures one minute, twenty passes the next, with
    /// nothing changed in between. A suite that passes depending on whether
    /// somebody's server happens to be up cannot gate a merge.
    ///
    /// And this suite was the largest single source of stray fixtures: it seeds
    /// an organization per test and never deleted one. Most of the 5 128
    /// abandoned organizations found in the dev database came from here, which in
    /// turn is what made the run queue crowded enough to break other things.
    ///
    /// Its own database settles both at once — nothing else claims its runs, and
    /// nothing it leaves behind outlives the run.
    static SCRATCH_URL: OnceCell<String> = OnceCell::const_new();

    async fn make_pool() -> PgPool {
        scratch_pool("mestier_automation_test_", &SCRATCH_URL).await
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

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    /// Generous enough that no acceptance test other than the budget one is
    /// ever bounded by it.
    fn generous_schedule() -> WorkerSchedule {
        WorkerSchedule {
            interval: Duration::from_secs(5),
            batch: 100,
            per_org: 100,
            claim_timeout: Duration::from_secs(300),
            max_steps_per_slice: 10_000,
            max_slice_duration: Duration::from_secs(120),
        }
    }

    fn schedule_with_step_budget(max_steps: usize) -> WorkerSchedule {
        WorkerSchedule {
            max_steps_per_slice: max_steps,
            ..generous_schedule()
        }
    }

    async fn start_workflow(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        graph: Graph,
    ) -> Uuid {
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph,
                created_by: None,
            })
            .await
            .unwrap();
        workflow.id
    }

    fn condition(id: &str, predicate: &str) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(predicate));
        PlacedConnector {
            id: id.to_string(),
            kind: "flow.condition".to_string(),
            version: 1,
            credential_id: None,
            config,
        }
    }

    fn flow_loop(id: &str, items: Value) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), items);
        PlacedConnector {
            id: id.to_string(),
            kind: "flow.loop".to_string(),
            version: 1,
            credential_id: None,
            config,
        }
    }

    fn customer_create(id: &str, name: &str) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("name".to_string(), json!(name));
        PlacedConnector {
            id: id.to_string(),
            kind: "mestier.customer.create".to_string(),
            version: 1,
            credential_id: None,
            config,
        }
    }

    fn edge(from: &str, to: &str, branch: Option<Branch>) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            branch,
        }
    }

    #[derive(Debug)]
    struct StepRow {
        connector_id: String,
        iteration_path: String,
        status: String,
        attempts: i32,
        error: Option<String>,
    }

    async fn run_steps(pool: &PgPool, run_id: Uuid) -> Vec<StepRow> {
        sqlx::query_as!(
            StepRow,
            r#"SELECT connector_id, iteration_path, status, attempts, error
               FROM automation.run_step WHERE run_id = $1 ORDER BY created_at"#,
            run_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn run_status(pool: &PgPool, run_id: Uuid) -> String {
        sqlx::query_scalar!("SELECT status FROM automation.run WHERE id = $1", run_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn customer_count(pool: &PgPool, org_id: OrganizationId) -> i64 {
        sqlx::query_scalar!("SELECT COUNT(*) FROM customers WHERE org_id = $1", org_id.0,)
            .fetch_one(pool)
            .await
            .unwrap()
            .unwrap_or(0)
    }

    async fn make_run_due_now(pool: &PgPool, run_id: Uuid) {
        sqlx::query!(
            "UPDATE automation.run SET next_attempt_at = now() WHERE id = $1",
            run_id,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    /// Overrides the organization's retry schedule with a single, short
    /// entry so a test can watch a step die without waiting on the default
    /// six-attempt schedule. `next_attempt_at` is still forced to `now()`
    /// between passes rather than actually slept on.
    async fn seed_short_retry_schedule(pool: &PgPool, org_id: OrganizationId) {
        sqlx::query!(
            r#"INSERT INTO automation.settings (org_id, retry_schedule_seconds)
               VALUES ($1, ARRAY[1]::bigint[])"#,
            org_id.0,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_linear_run_of_four_connectors_completes_with_one_step_each() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "linear").await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![
                condition("c1", "{{ true }}"),
                condition("c2", "{{ true }}"),
                condition("c3", "{{ true }}"),
                condition("c4", "{{ true }}"),
            ],
            edges: vec![
                edge("c1", "c2", Some(Branch::Then)),
                edge("c2", "c3", Some(Branch::Then)),
                edge("c3", "c4", Some(Branch::Then)),
            ],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        // Asserted on our own run, never on the pass's aggregate counters:
        // `run_engine_pass` claims every due run system-wide (the same
        // shape as the delivery pass), so a shared dev database can carry
        // other runs alongside ours.
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");

        let steps = run_steps(&pool, run_id).await;
        assert_eq!(
            steps.len(),
            4,
            "one run_step row per connector, literally: {steps:?}"
        );
        assert!(steps.iter().all(|s| s.status == "succeeded"), "{steps:?}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_flow_loop_over_twelve_elements_produces_twelve_distinct_iteration_paths() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "loop12").await;
        let usecase = use_case(pool.clone());

        let items: Vec<Value> = (0..12).map(|i| json!({ "index": i })).collect();
        let graph = Graph {
            connectors: vec![
                flow_loop("loop1", json!("{{ trigger.items }}")),
                condition("body", "{{ true }}"),
            ],
            edges: vec![edge("loop1", "body", Some(Branch::Each))],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "items": items }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        let bodies: Vec<&StepRow> = steps.iter().filter(|s| s.connector_id == "body").collect();
        assert_eq!(bodies.len(), 12, "{steps:?}");
        let paths: std::collections::BTreeSet<&str> =
            bodies.iter().map(|s| s.iteration_path.as_str()).collect();
        assert_eq!(
            paths.len(),
            12,
            "every iteration_path is distinct: {steps:?}"
        );
        for i in 0..12 {
            assert!(
                paths.contains(format!("loop1[{i}]").as_str()),
                "missing loop1[{i}] in {paths:?}"
            );
        }
    }

    /// The acceptance criterion the whole ticket rests on: a step that keeps
    /// failing is retried in place, and resuming never re-creates the work
    /// already done before it.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn failing_partway_through_a_loop_then_resuming_does_not_reexecute_earlier_elements() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "resume-500").await;
        let usecase = use_case(pool.clone());

        const TOTAL: usize = 500;
        const POISON_INDEX: usize = 299;
        let items: Vec<Value> = (0..TOTAL)
            .map(|i| {
                if i == POISON_INDEX {
                    json!({ "name": "" })
                } else {
                    json!({ "name": format!("Customer {i}") })
                }
            })
            .collect();

        let graph = Graph {
            connectors: vec![
                flow_loop("loop1", json!("{{ trigger.items }}")),
                customer_create("body", "{{ loop.item.name }}"),
            ],
            edges: vec![edge("loop1", "body", Some(Branch::Each))],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "items": items }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());

        // First pass: everything up to (but not including) the poisoned
        // element succeeds; that element fails and halts the run.
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "pending");
        assert_eq!(
            customer_count(&pool, org_id).await,
            POISON_INDEX as i64,
            "only the elements before the poisoned one were created"
        );
        let steps = run_steps(&pool, run_id).await;
        let poisoned = steps
            .iter()
            .find(|s| {
                s.connector_id == "body" && s.iteration_path == format!("loop1[{POISON_INDEX}]")
            })
            .expect("the poisoned iteration has its own row");
        assert_eq!(poisoned.status, "failed");
        assert_eq!(poisoned.attempts, 1);

        // Resume: force the retry due now rather than waiting on the
        // organization's real backoff.
        make_run_due_now(&pool, run_id).await;
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(
            customer_count(&pool, org_id).await,
            POISON_INDEX as i64,
            "resuming must not recreate the elements before the poisoned one"
        );
        let steps_after = run_steps(&pool, run_id).await;
        let poisoned_after = steps_after
            .iter()
            .find(|s| {
                s.connector_id == "body" && s.iteration_path == format!("loop1[{POISON_INDEX}]")
            })
            .unwrap();
        assert_eq!(
            poisoned_after.attempts, 2,
            "the poisoned iteration itself was retried"
        );
        assert_eq!(
            steps_after
                .iter()
                .filter(|s| s.connector_id == "body" && s.status == "succeeded")
                .count(),
            POISON_INDEX,
            "no earlier element gained a second row or a second attempt"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn flow_condition_takes_then_and_records_the_else_branch_as_skipped() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "condition").await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![
                condition("c1", "{{ true }}"),
                condition("then_branch", "{{ true }}"),
                condition("else_branch", "{{ true }}"),
            ],
            edges: vec![
                edge("c1", "then_branch", Some(Branch::Then)),
                edge("c1", "else_branch", Some(Branch::Else)),
            ],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        let then_step = steps
            .iter()
            .find(|s| s.connector_id == "then_branch")
            .unwrap();
        assert_eq!(then_step.status, "succeeded");
        let else_step = steps
            .iter()
            .find(|s| s.connector_id == "else_branch")
            .expect("the skipped branch still has its own row, not an absent one");
        assert_eq!(else_step.status, "skipped");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_failed_step_retries_on_the_organizations_schedule_then_dies_once_exhausted() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "retry-die").await;
        seed_short_retry_schedule(&pool, org_id).await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![customer_create("body", "")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());

        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();
        assert_eq!(run_status(&pool, run_id).await, "pending");
        let first = run_steps(&pool, run_id).await;
        let body = first.iter().find(|s| s.connector_id == "body").unwrap();
        assert_eq!(body.status, "failed");
        assert_eq!(body.attempts, 1);

        make_run_due_now(&pool, run_id).await;
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "failed");
        let second = run_steps(&pool, run_id).await;
        let body_after = second.iter().find(|s| s.connector_id == "body").unwrap();
        assert_eq!(body_after.status, "dead");
        assert_eq!(body_after.attempts, 2);
        assert!(body_after.error.is_some());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_run_that_exhausts_its_slice_budget_returns_and_finishes_on_the_next_pass() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "budget").await;
        let usecase = use_case(pool.clone());

        let ids = ["c1", "c2", "c3", "c4", "c5"];
        let connectors: Vec<PlacedConnector> =
            ids.iter().map(|id| condition(id, "{{ true }}")).collect();
        let edges: Vec<Edge> = ids
            .windows(2)
            .map(|pair| edge(pair[0], pair[1], Some(Branch::Then)))
            .collect();
        let workflow_id = start_workflow(&usecase, org_id, Graph { connectors, edges }).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());

        // Asserted on our own run throughout, never on the pass's aggregate
        // counters — see the note on the linear-run test.
        usecase
            .run_engine_pass(&connectors, "worker-1", schedule_with_step_budget(3))
            .await
            .unwrap();
        assert_eq!(
            run_status(&pool, run_id).await,
            "pending",
            "the slice ran out before finishing"
        );
        let after_first = run_steps(&pool, run_id).await;
        assert_eq!(
            after_first
                .iter()
                .filter(|s| s.status == "succeeded")
                .count(),
            3,
            "{after_first:?}"
        );

        make_run_due_now(&pool, run_id).await;
        usecase
            .run_engine_pass(&connectors, "worker-1", schedule_with_step_budget(3))
            .await
            .unwrap();
        assert_eq!(
            run_status(&pool, run_id).await,
            "succeeded",
            "the run finished on the next pass"
        );
        let after_second = run_steps(&pool, run_id).await;
        assert_eq!(
            after_second
                .iter()
                .filter(|s| s.status == "succeeded")
                .count(),
            5,
            "no step ran twice: {after_second:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_worker_killed_mid_run_leaves_it_recoverable_by_releasing_stale_claims() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "stale").await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        // Claimed, but never processed — exactly what a worker dying between
        // the claim and its first step leaves behind.
        let claimed = usecase.claim_runs("worker-doomed", 100, 100).await.unwrap();
        assert!(claimed.iter().any(|r| r.id == run_id));
        assert_eq!(run_status(&pool, run_id).await, "running");

        sqlx::query!(
            "UPDATE automation.run SET locked_at = now() - interval '1 hour' WHERE id = $1",
            run_id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let released = usecase
            .recover_lost_runs(Utc::now() - chrono::Duration::minutes(5))
            .await
            .unwrap();
        assert!(released >= 1);
        assert_eq!(run_status(&pool, run_id).await, "pending");

        // Recoverable in practice: a normal pass now finishes it. Asserted
        // on our own run, never on the pass's aggregate counters — see the
        // note on the linear-run test.
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-2", generous_schedule())
            .await
            .unwrap();
        assert_eq!(run_status(&pool, run_id).await, "succeeded");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn editing_a_workflow_mid_run_does_not_change_what_the_run_executes() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "pinned-version").await;
        let usecase = use_case(pool.clone());

        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Pinned".to_string(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: Graph {
                    connectors: vec![condition("only_v1", "{{ true }}")],
                    edges: vec![],
                },
                created_by: None,
            })
            .await
            .unwrap();

        let run_id = usecase
            .start_run(org_id, workflow.id, json!({}))
            .await
            .unwrap();

        // Edited after the run was started, before it was ever worked.
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: Graph {
                    connectors: vec![condition("only_v2", "{{ true }}")],
                    edges: vec![],
                },
                created_by: None,
            })
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        assert!(
            steps.iter().any(|s| s.connector_id == "only_v1"),
            "the run executed the version it was pinned to: {steps:?}"
        );
        assert!(
            steps.iter().all(|s| s.connector_id != "only_v2"),
            "the run must never see the edit made after it started: {steps:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn nested_loops_produce_distinct_iteration_paths() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "nested-loops").await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![
                flow_loop("outer", json!(["a", "b"])),
                flow_loop("inner", json!(["x", "y", "z"])),
                condition("leaf", "{{ true }}"),
            ],
            edges: vec![
                edge("outer", "inner", Some(Branch::Each)),
                edge("inner", "leaf", Some(Branch::Each)),
            ],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;

        let inner_rows: Vec<&StepRow> =
            steps.iter().filter(|s| s.connector_id == "inner").collect();
        assert_eq!(inner_rows.len(), 2, "{steps:?}");
        let inner_paths: std::collections::BTreeSet<&str> = inner_rows
            .iter()
            .map(|s| s.iteration_path.as_str())
            .collect();
        assert_eq!(
            inner_paths,
            std::collections::BTreeSet::from(["outer[0]", "outer[1]"])
        );

        let leaf_rows: Vec<&StepRow> = steps.iter().filter(|s| s.connector_id == "leaf").collect();
        assert_eq!(leaf_rows.len(), 6, "{steps:?}");
        let leaf_paths: std::collections::BTreeSet<&str> = leaf_rows
            .iter()
            .map(|s| s.iteration_path.as_str())
            .collect();
        assert_eq!(
            leaf_paths.len(),
            6,
            "every nested iteration_path is distinct"
        );
        for outer in 0..2 {
            for inner in 0..3 {
                let path = format!("outer[{outer}].inner[{inner}]");
                assert!(
                    leaf_paths.contains(path.as_str()),
                    "missing {path} in {leaf_paths:?}"
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn starting_a_run_for_a_workflow_with_no_saved_version_is_a_conflict() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "no-version").await;
        let usecase = use_case(pool.clone());
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Never saved".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let error = usecase
            .start_run(org_id, workflow.id, json!({}))
            .await
            .expect_err("a workflow with no version cannot be run");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn starting_a_run_for_an_unknown_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "no-workflow").await;
        let usecase = use_case(pool.clone());

        let error = usecase
            .start_run(org_id, generate_uuid_v7(), json!({}))
            .await
            .expect_err("there is no workflow to run");

        assert!(matches!(error, CoreError::NotFound));
    }

    // --- find_run / list_runs / list_run_steps --------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn finding_a_run_in_its_own_organization_returns_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-run").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "x": 1 }))
            .await
            .unwrap();

        let found = usecase.find_run(org_id, run_id).await.unwrap();

        let found = found.expect("the run belongs to this organization");
        assert_eq!(found.id, run_id);
        assert_eq!(found.workflow_id, workflow_id);
        assert_eq!(found.trigger_payload, Some(json!({ "x": 1 })));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn finding_a_run_from_another_organization_reads_back_absent() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-run-owner").await;
        let stranger_org = seed_organization(&pool, "find-run-stranger").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let found = usecase.find_run(stranger_org, run_id).await.unwrap();

        assert_eq!(
            found, None,
            "a run id from another organization must read back as absent, not forbidden"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_runs_only_returns_the_requesting_organizations_runs() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list-runs-owner").await;
        let other_org = seed_organization(&pool, "list-runs-other").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph.clone()).await;
        let other_workflow_id = start_workflow(&usecase, other_org, graph).await;
        usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        usecase
            .start_run(other_org, other_workflow_id, json!({}))
            .await
            .unwrap();

        let listed = usecase.list_runs(org_id).await.unwrap();

        assert_eq!(listed.len(), 1);
        assert!(listed.iter().all(|r| r.org_id == org_id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_run_steps_exposes_each_steps_resolved_input_and_output() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list-steps").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![customer_create("c1", "Inspected Customer")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].connector_id, "c1");
        assert_eq!(steps[0].status, StepStatus::Succeeded);
        assert_eq!(steps[0].attempts, 1);
        let input = steps[0].input.as_ref().expect("a resolved input is kept");
        assert_eq!(input["name"], json!("Inspected Customer"));
        assert!(steps[0].output.is_some());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_run_steps_for_another_organization_is_empty() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list-steps-owner").await;
        let stranger_org = seed_organization(&pool, "list-steps-stranger").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        let steps = usecase.list_run_steps(stranger_org, run_id).await.unwrap();

        assert!(steps.is_empty());
    }

    // --- replay_run_from_step --------------------------------------------

    /// The acceptance criterion the whole endpoint exists for: replaying
    /// from a step re-executes that step and everything downstream of it,
    /// and nothing upstream — proven by an observable side effect
    /// (`mestier.customer.create`), not merely by step-row bookkeeping.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_from_a_step_only_reexecutes_that_step_and_its_descendants() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay").await;
        let usecase = use_case(pool.clone());

        let graph = Graph {
            connectors: vec![
                customer_create("c1", "A"),
                customer_create("c2", "B"),
                customer_create("c3", "C"),
            ],
            edges: vec![edge("c1", "c2", None), edge("c2", "c3", None)],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();
        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        assert_eq!(customer_count(&pool, org_id).await, 3);

        let c1_step_id_before: Uuid = sqlx::query_scalar!(
            "SELECT id FROM automation.run_step WHERE run_id = $1 AND connector_id = 'c1'",
            run_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let replayed = usecase
            .replay_run_from_step(org_id, run_id, "c2".to_string())
            .await
            .unwrap();
        assert_eq!(replayed.status, RunStatus::Pending);

        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();
        assert_eq!(run_status(&pool, run_id).await, "succeeded");

        assert_eq!(
            customer_count(&pool, org_id).await,
            5,
            "c2 and c3 ran again (2 more customers); c1 did not (still 3 + 2)"
        );

        let c1_step_id_after: Uuid = sqlx::query_scalar!(
            "SELECT id FROM automation.run_step WHERE run_id = $1 AND connector_id = 'c1'",
            run_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            c1_step_id_after, c1_step_id_before,
            "the step upstream of the replay target must be untouched, not re-created"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_an_unknown_connector_is_refused_and_names_it() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay-unknown").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        let error = usecase
            .replay_run_from_step(org_id, run_id, "not-a-real-connector".to_string())
            .await
            .expect_err("the graph has no such connector");

        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(format!("{error}").contains("not-a-real-connector"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_a_run_still_pending_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay-pending").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let error = usecase
            .replay_run_from_step(org_id, run_id, "c1".to_string())
            .await
            .expect_err("a run still pending has never been worked yet");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_a_run_from_another_organization_is_not_found() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay-owner").await;
        let stranger_org = seed_organization(&pool, "replay-stranger").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        let error = usecase
            .replay_run_from_step(stranger_org, run_id, "c1".to_string())
            .await
            .expect_err("a stranger must not replay another organization's run");

        assert!(matches!(error, CoreError::NotFound));
    }
}

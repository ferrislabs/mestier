use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        collection::{COLLECTION_OPERATION_KIND, NESTED_EXPRESSION_FIELDS},
        connector::{ExecutionMode, connector_catalogue},
        expression::{ExpressionContext, LoopFrame},
        ports::{AutomationSettingsRepository, RunRepository, WorkflowRepository},
        run::{
            ConnectorInput, ConnectorOutcome, DueRun, LoopStackEntry, MERGE_KIND, Next, NodeIndex,
            Run, RunOverview, RunSettlement, RunStatus, RunStep, StepItems, StepStatus, as_item,
            connectors_by_id, descendants_via, flatten, format_iteration_path, gather_lists,
            merge_blockers, next_after, push_result, referenced_names, resolve_config,
            resolve_config_nested, roots, trigger_item,
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
pub(super) struct LoopStep {
    pub(super) loop_id: String,
    pub(super) index: usize,
    pub(super) item: Arc<Value>,
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
    fn scope_paths(&self) -> Vec<String> {
        let entries: Vec<LoopStackEntry> = self
            .path
            .iter()
            .map(|step| (step.loop_id.clone(), step.index))
            .collect();
        (0..=entries.len())
            .rev()
            .map(|depth| format_iteration_path(&entries[..depth]))
            .collect()
    }

    fn iteration_path(&self) -> String {
        let entries: Vec<LoopStackEntry> = self
            .path
            .iter()
            .map(|step| (step.loop_id.clone(), step.index))
            .collect();
        format_iteration_path(&entries)
    }

    fn loop_item(&self, loop_id: &str) -> Option<Value> {
        self.path
            .iter()
            .rev()
            .find(|step| step.loop_id == loop_id)
            .map(|step| step.item.as_ref().clone())
    }
}

/// What one slice of one run did, before it is settled.
enum SliceResult {
    Completed,
    Parked {
        resume_at: DateTime<Utc>,
    },
    BudgetExhausted,
    StepFailed {
        error: String,
        dead: bool,
        next_attempt_at: Option<DateTime<Utc>>,
    },
}

fn succeeded_output_in_scope(
    index: &HashMap<(String, String), RunStep>,
    connector_id: &str,
    scope_paths: &[String],
) -> Option<Value> {
    let step = scope_paths
        .iter()
        .find_map(|path| index.get(&(connector_id.to_owned(), path.clone())))?;
    (step.status == StepStatus::Succeeded).then(|| step.output.clone().unwrap_or(Value::Null))
}

fn succeeded_items_in_scope(
    index: &HashMap<(String, String), RunStep>,
    connector_id: &str,
    scope_paths: &[String],
) -> Option<StepItems> {
    succeeded_output_in_scope(index, connector_id, scope_paths)
        .map(|output| StepItems::from_json(&output))
}

fn input_lists(
    graph: &Graph,
    index: &HashMap<(String, String), RunStep>,
    frame: &Frame,
    trigger: &Value,
) -> Vec<Vec<Value>> {
    let scope_paths = frame.scope_paths();
    gather_lists(
        graph,
        &frame.connector_id,
        |from, branch| {
            if branch == Some(Branch::Each)
                && let Some(item) = frame.loop_item(from)
            {
                return Some(vec![as_item(item)]);
            }
            succeeded_items_in_scope(index, from, &scope_paths).map(|items| items.sent(branch))
        },
        trigger,
    )
}

enum InputGate {
    Ready,
    Parked,
    Deferred,
}

fn input_gate(
    graph: &Graph,
    index: &HashMap<(String, String), RunStep>,
    frame: &Frame,
) -> InputGate {
    let blockers = merge_blockers(graph, &frame.connector_id);
    if blockers.len() < 2 {
        return InputGate::Ready;
    }
    let scope_paths = frame.scope_paths();
    let mut gate = InputGate::Ready;
    for blocker in blockers {
        let step = scope_paths
            .iter()
            .find_map(|path| index.get(&(blocker.to_owned(), path.clone())));
        match step.map(|step| step.status) {
            Some(StepStatus::Waiting) => return InputGate::Parked,
            Some(status) if status.is_settled() => {}
            _ => gate = InputGate::Deferred,
        }
    }
    gate
}

fn release_deferred(
    deferred: &mut Vec<Frame>,
    index: &HashMap<(String, String), RunStep>,
    forced: &mut HashSet<(String, String)>,
) -> Option<Frame> {
    while !deferred.is_empty() {
        let frame = deferred.remove(0);
        let key = (frame.connector_id.clone(), frame.iteration_path());
        if index.get(&key).is_some_and(|step| step.status.is_settled()) {
            continue;
        }
        forced.insert(key);
        return Some(frame);
    }
    None
}

fn earliest(current: Option<DateTime<Utc>>, candidate: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Some(current.map_or(candidate, |current| current.min(candidate)))
}

fn pass_through_step(
    run_id: Uuid,
    connector_id: &str,
    iteration_path: &str,
    input: &[Value],
    output: &StepItems,
) -> RunStep {
    let now = Utc::now();
    RunStep {
        id: generate_uuid_v7(),
        run_id,
        connector_id: connector_id.to_owned(),
        iteration_path: iteration_path.to_owned(),
        attempts: 0,
        status: StepStatus::Succeeded,
        input: Some(Value::Array(input.to_vec())),
        output: Some(output.to_json()),
        error: None,
        resume_at: None,
        started_at: Some(now),
        finished_at: Some(now),
        created_at: now,
    }
}

enum NodeResult {
    Done(StepItems),
    Wait {
        resume_at: DateTime<Utc>,
        items: StepItems,
    },
    Failed(String),
}

struct NodeRun<'a> {
    due: &'a DueRun,
    placed: &'a PlacedConnector,
    mode: ExecutionMode,
    branches: &'a [Branch],
    input: &'a [Value],
    lists: &'a [Vec<Value>],
    view: &'a BTreeMap<String, Vec<Value>>,
    loop_step: Option<&'a LoopStep>,
}

async fn run_over_items(connectors: &ConnectorRegistry, run: &NodeRun<'_>) -> NodeResult {
    let positions: Vec<usize> = match run.mode {
        ExecutionMode::PerItem => (0..run.input.len()).collect(),
        ExecutionMode::Once => vec![0],
    };
    let mut produced: Vec<Value> = Vec::new();
    let mut then_items: Vec<Value> = Vec::new();
    let mut else_items: Vec<Value> = Vec::new();
    let mut iterated: Vec<Value> = Vec::new();

    for position in positions {
        let ctx = ExpressionContext {
            trigger: run.due.trigger_payload.as_ref(),
            nodes: run.view,
            position,
            loop_frame: run.loop_step.map(|step| LoopFrame {
                item: &step.item,
                index: step.index,
            }),
            now: Utc::now(),
        };
        let resolved = if run.placed.kind == COLLECTION_OPERATION_KIND {
            resolve_config_nested(&run.placed.config, &ctx, &NESTED_EXPRESSION_FIELDS)
        } else {
            resolve_config(&run.placed.config, &ctx)
        };
        let mut config = match resolved {
            Ok(config) => config,
            Err(error) => return NodeResult::Failed(error.to_string()),
        };
        if run.placed.kind == MERGE_KIND {
            config.insert(
                "inputs".to_owned(),
                Value::Array(run.lists.iter().cloned().map(Value::Array).collect()),
            );
        }

        let input = ConnectorInput {
            org_id: run.due.org_id,
            run_id: run.due.id,
            config: &config,
            credential_id: run.placed.credential_id,
            operation: run.placed.operation.as_ref(),
        };
        let outcome = match connectors
            .execute(&run.placed.kind, run.placed.version, input)
            .await
        {
            Ok(outcome) => outcome,
            Err(unknown) => return NodeResult::Failed(unknown.to_string()),
        };

        match outcome {
            ConnectorOutcome::Produced(value) => push_result(&mut produced, value),
            ConnectorOutcome::Branch { taken, .. } => {
                if let Some(item) = run.input.get(position) {
                    if taken == Branch::Then {
                        then_items.push(item.clone());
                    } else {
                        else_items.push(item.clone());
                    }
                }
            }
            ConnectorOutcome::Iterate { items } => iterated = items,
            ConnectorOutcome::Wait { resume_at, .. } => {
                return NodeResult::Wait {
                    resume_at,
                    items: StepItems::Main(run.input.to_vec()),
                };
            }
            ConnectorOutcome::Failed { error } => return NodeResult::Failed(error),
        }
    }

    NodeResult::Done(if run.branches.contains(&Branch::Then) {
        StepItems::Branched(vec![(Branch::Then, then_items), (Branch::Else, else_items)])
    } else if run.branches.contains(&Branch::Each) {
        StepItems::Branched(vec![
            (Branch::Each, iterated.clone()),
            (Branch::After, iterated),
        ])
    } else {
        StepItems::Main(produced)
    })
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
        workflow.ensure_not_archived()?;
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
            trigger_id: None,
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
    pub async fn find_run_overview(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
    ) -> Result<Option<RunOverview>, CoreError> {
        let mut runs = run_repository;
        runs.overview_by_id(org_id, run_id).await
    }

    #[transactional(run)]
    pub async fn list_run_overviews(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<RunOverview>, CoreError> {
        let mut runs = run_repository;
        runs.overviews_by_organization(org_id).await
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
    #[transactional(run, workflow)]
    pub async fn replay_run_from_step(
        &self,
        org_id: OrganizationId,
        run_id: Uuid,
        connector_id: String,
    ) -> Result<RunOverview, CoreError> {
        let mut runs = run_repository;
        let mut workflows = workflow_repository;

        let run = runs
            .find_by_id(org_id, run_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if matches!(run.status, RunStatus::Pending | RunStatus::Running) {
            return Err(CoreError::Conflict(
                "a run still in progress cannot be replayed".to_string(),
            ));
        }
        workflows
            .find_by_id(org_id, run.workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?
            .ensure_not_archived()?;

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

        runs.overview_by_id(org_id, run_id)
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
                Ok(SliceResult::BudgetExhausted | SliceResult::Parked { .. }) => {
                    outcome.rescheduled += 1
                }
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
        let catalogue = connector_catalogue();
        let node_index = NodeIndex::new(&graph);
        let trigger = trigger_item(due.trigger_payload.as_ref());

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
        let mut parked_until: Option<DateTime<Utc>> = None;
        let mut deferred: Vec<Frame> = Vec::new();
        let mut forced: HashSet<(String, String)> = HashSet::new();
        let mut early_result: Option<SliceResult> = None;

        loop {
            let frame = match stack.pop() {
                Some(frame) => frame,
                None => match parked_until
                    .is_none()
                    .then(|| release_deferred(&mut deferred, &index, &mut forced))
                    .flatten()
                {
                    Some(frame) => frame,
                    None => break,
                },
            };
            let iteration_path = frame.iteration_path();
            let key = (frame.connector_id.clone(), iteration_path.clone());

            if let Some(existing) = index.get(&key).cloned()
                && existing.status.is_settled()
            {
                match existing.status {
                    StepStatus::Succeeded => {
                        let output =
                            StepItems::from_json(existing.output.as_ref().unwrap_or(&Value::Null));
                        self.advance(due.id, &graph, &frame, &output, &mut stack, &mut index)
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
                    StepStatus::Pending
                    | StepStatus::InFlight
                    | StepStatus::Failed
                    | StepStatus::Waiting => {
                        unreachable!("is_settled() already excludes these statuses")
                    }
                }
            }

            if let Some(existing) = index.get(&key)
                && existing.status == StepStatus::Waiting
                && let Some(resume_at) = existing.resume_at
                && resume_at > Utc::now()
            {
                parked_until = earliest(parked_until, resume_at);
                continue;
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

            if let Some(existing) = index.get(&key).cloned()
                && existing.status == StepStatus::Waiting
            {
                let output = StepItems::from_json(existing.output.as_ref().unwrap_or(&Value::Null));
                let resumed = self
                    .write_step(&RunStep {
                        status: StepStatus::Succeeded,
                        resume_at: None,
                        finished_at: Some(Utc::now()),
                        ..existing
                    })
                    .await?;
                steps_executed += 1;
                index.insert(key, resumed);
                self.advance(due.id, &graph, &frame, &output, &mut stack, &mut index)
                    .await?;
                continue;
            }

            if !forced.contains(&key) {
                match input_gate(&graph, &index, &frame) {
                    InputGate::Ready => {}
                    InputGate::Parked => continue,
                    InputGate::Deferred => {
                        deferred.push(frame);
                        continue;
                    }
                }
            }

            let descriptor = catalogue.get(&placed.kind, placed.version);
            let branches = descriptor.map_or(&[][..], |descriptor| descriptor.branches);
            let mode = descriptor.map_or(ExecutionMode::PerItem, |descriptor| descriptor.execution);
            let lists = input_lists(&graph, &index, &frame, &trigger);
            let input = flatten(&lists);

            if placed.disabled {
                let output = StepItems::passthrough(branches, input.clone());
                let settled = self
                    .write_step(&pass_through_step(
                        due.id,
                        &frame.connector_id,
                        &iteration_path,
                        &input,
                        &output,
                    ))
                    .await?;
                steps_executed += 1;
                index.insert(key, settled);

                self.advance(due.id, &graph, &frame, &output, &mut stack, &mut index)
                    .await?;
                continue;
            }

            let scope_paths = frame.scope_paths();
            let names = referenced_names(
                &placed.config,
                if placed.kind == COLLECTION_OPERATION_KIND {
                    &NESTED_EXPRESSION_FIELDS
                } else {
                    &[]
                },
            );
            let view = node_index.view(
                &names,
                &frame.connector_id,
                &trigger,
                |id| frame.loop_item(id),
                |id| succeeded_items_in_scope(&index, id, &scope_paths),
            );

            let now = Utc::now();
            let in_flight = RunStep {
                id: generate_uuid_v7(),
                run_id: due.id,
                connector_id: frame.connector_id.clone(),
                iteration_path: iteration_path.clone(),
                attempts: attempts_before + 1,
                status: StepStatus::InFlight,
                input: Some(Value::Array(input.clone())),
                output: None,
                error: None,
                resume_at: None,
                started_at: Some(now),
                finished_at: None,
                created_at: now,
            };
            let written = self.write_step(&in_flight).await?;
            steps_executed += 1;

            let result = run_over_items(
                connectors,
                &NodeRun {
                    due,
                    placed,
                    mode,
                    branches,
                    input: &input,
                    lists: &lists,
                    view: &view,
                    loop_step: frame.path.last(),
                },
            )
            .await;

            match result {
                NodeResult::Failed(error) => {
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
                        error,
                        dead,
                        next_attempt_at,
                    });
                    break;
                }
                NodeResult::Wait { resume_at, items } => {
                    let parked = RunStep {
                        status: StepStatus::Waiting,
                        output: Some(items.to_json()),
                        resume_at: Some(resume_at),
                        ..written
                    };
                    let parked = self.write_step(&parked).await?;
                    index.insert(key, parked);
                    parked_until = earliest(parked_until, resume_at);
                }
                NodeResult::Done(output) => {
                    let settled = RunStep {
                        status: StepStatus::Succeeded,
                        output: Some(output.to_json()),
                        finished_at: Some(Utc::now()),
                        ..written
                    };
                    let settled = self.write_step(&settled).await?;
                    index.insert(key, settled);

                    self.advance(due.id, &graph, &frame, &output, &mut stack, &mut index)
                        .await?;
                }
            }
        }

        let result = early_result.unwrap_or(match parked_until {
            Some(resume_at) => SliceResult::Parked { resume_at },
            None => SliceResult::Completed,
        });
        let settlement = match &result {
            SliceResult::Completed => RunSettlement::Succeeded,
            SliceResult::Parked { resume_at } => RunSettlement::Parked {
                resume_at: *resume_at,
            },
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
        output: &StepItems,
        stack: &mut Vec<Frame>,
        index: &mut HashMap<(String, String), RunStep>,
    ) -> Result<(), CoreError> {
        match next_after(graph, &frame.connector_id, output) {
            Next::Follow { targets, skipped } => {
                let iteration_path = frame.iteration_path();
                let to_skip: Vec<&str> = skipped
                    .into_iter()
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
                            resume_at: None,
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

                for target in targets {
                    stack.push(Frame {
                        connector_id: target.to_string(),
                        path: frame.path.clone(),
                    });
                }
            }
            Next::Iterate { after, each, items } => {
                for after in after {
                    stack.push(Frame {
                        connector_id: after.to_string(),
                        path: frame.path.clone(),
                    });
                }
                for (idx, item) in items.iter().enumerate().rev() {
                    let shared = Arc::new(item.clone());
                    for target in &each {
                        let mut path = frame.path.clone();
                        path.push(LoopStep {
                            loop_id: frame.connector_id.clone(),
                            index: idx,
                            item: Arc::clone(&shared),
                        });
                        stack.push(Frame {
                            connector_id: (*target).to_string(),
                            path,
                        });
                    }
                }
            }
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
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, Edge, SaveWorkflowVersionCommand,
    };
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
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
    async fn make_pool() -> PgPool {
        automation_pool().await
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

    use crate::domain::automation::workflow::{PlacedTrigger, TriggerKind};

    fn triggered(mut graph: Graph) -> Graph {
        if !graph.triggers.is_empty() {
            return graph;
        }
        let targeted: std::collections::HashSet<&str> =
            graph.edges.iter().map(|e| e.to.as_str()).collect();
        let entry_points: Vec<String> = graph
            .connectors
            .iter()
            .map(|c| c.id.clone())
            .filter(|id| !targeted.contains(id.as_str()))
            .collect();
        if entry_points.is_empty() {
            return graph;
        }
        graph.triggers.push(PlacedTrigger {
            id: "t1".to_string(),
            kind: TriggerKind::Manual,
            name: None,
            description: None,
        });
        for id in entry_points {
            graph.edges.push(Edge {
                from: "t1".to_string(),
                to: id,
                branch: None,
            });
        }
        graph
    }

    async fn start_workflow(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        graph: Graph,
    ) -> Uuid {
        let graph = triggered(graph);
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
                layout: None,
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
        }
    }

    fn flow_config(id: &str, variables: Value) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), variables);
        PlacedConnector {
            id: id.to_string(),
            kind: "flow.config".to_string(),
            version: 1,
            credential_id: None,
            config,
            name: None,
            description: None,
            disabled: false,
            operation: None,
        }
    }

    fn http_get(id: &str, url: &str) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("method".to_string(), json!("GET"));
        config.insert("url".to_string(), json!(url));
        PlacedConnector {
            id: id.to_string(),
            kind: "http.request".to_string(),
            version: 1,
            credential_id: None,
            config,
            name: None,
            description: None,
            disabled: false,
            operation: None,
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

    fn disabled(mut connector: PlacedConnector) -> PlacedConnector {
        connector.disabled = true;
        connector
    }

    fn named(mut connector: PlacedConnector, name: &str) -> PlacedConnector {
        connector.name = Some(name.to_owned());
        connector
    }

    async fn step_output(pool: &PgPool, run_id: Uuid, connector_id: &str) -> Option<Value> {
        sqlx::query_scalar!(
            r#"SELECT output FROM automation.run_step
               WHERE run_id = $1 AND connector_id = $2"#,
            run_id,
            connector_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn step_output_at(
        pool: &PgPool,
        run_id: Uuid,
        connector_id: &str,
        iteration_path: &str,
    ) -> Option<Value> {
        sqlx::query_scalar!(
            r#"SELECT output FROM automation.run_step
               WHERE run_id = $1 AND connector_id = $2 AND iteration_path = $3"#,
            run_id,
            connector_id,
            iteration_path,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn status_of<'a>(steps: &'a [StepRow], connector_id: &str) -> Option<&'a str> {
        steps
            .iter()
            .find(|step| step.connector_id == connector_id)
            .map(|step| step.status.as_str())
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
            triggers: Vec::new(),
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
    async fn a_disabled_connector_is_recorded_succeeded_without_ever_being_called() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-not-called").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![disabled(customer_create("ghost", "Ghost customer"))],
            edges: vec![],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let payload = json!({ "quote": 7 });
        let run_id = usecase
            .start_run(org_id, workflow_id, payload.clone())
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        assert_eq!(customer_count(&pool, org_id).await, 0);
        let steps = run_steps(&pool, run_id).await;
        assert_eq!(steps.len(), 1, "{steps:?}");
        assert_eq!(steps[0].status, "succeeded");
        assert_eq!(steps[0].attempts, 0);
        assert_eq!(steps[0].error, None);
        assert_eq!(
            step_output(&pool, run_id, "ghost").await,
            Some(json!([payload]))
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_connector_cannot_fail_even_with_an_unreachable_url_or_a_broken_expression()
    {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-no-fail").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                disabled(http_get("unreachable", "http://127.0.0.1:1/never")),
                disabled(http_get("broken", "{{ trigger.missing.path }}")),
            ],
            edges: vec![edge("unreachable", "broken", None)],
            triggers: Vec::new(),
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
        assert!(steps.iter().all(|s| s.status == "succeeded"), "{steps:?}");
        assert_eq!(steps.len(), 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_connector_hands_its_predecessors_output_to_what_follows() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-passes").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                flow_config("source", json!({ "x": 1 })),
                disabled(http_get("off", "http://127.0.0.1:1/never")),
                flow_config("after", json!({ "y": 2 })),
            ],
            edges: vec![edge("source", "off", None), edge("off", "after", None)],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "trigger": true }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        assert_eq!(status_of(&steps, "after"), Some("succeeded"), "{steps:?}");
        assert_eq!(
            step_output(&pool, run_id, "off").await,
            Some(json!([{ "x": 1 }]))
        );
        assert_eq!(
            step_output(&pool, run_id, "after").await,
            Some(json!([{ "y": 2 }]))
        );
    }

    fn merge_after_a_branch_inside_a_loop() -> Graph {
        Graph {
            connectors: vec![
                flow_loop("loop1", json!("{{ trigger.items }}")),
                condition("cond", "{{ loop.item }}"),
                flow_config("a", json!({ "branch": "then" })),
                flow_config("b", json!({ "branch": "else" })),
                disabled(http_get("merge", "http://127.0.0.1:1/never")),
            ],
            edges: vec![
                edge("loop1", "cond", Some(Branch::Each)),
                edge("cond", "a", Some(Branch::Then)),
                edge("cond", "b", Some(Branch::Else)),
                edge("a", "merge", None),
                edge("b", "merge", None),
            ],
            triggers: Vec::new(),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_merge_after_a_branch_in_a_loop_passes_the_branch_taken_in_each_iteration() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-merge-loop").await;
        let usecase = use_case(pool.clone());
        let workflow_id =
            start_workflow(&usecase, org_id, merge_after_a_branch_inside_a_loop()).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "items": [true, false, true] }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        let steps = run_steps(&pool, run_id).await;
        assert_eq!(run_status(&pool, run_id).await, "succeeded", "{steps:?}");
        for (iteration, branch) in ["then", "else", "then"].into_iter().enumerate() {
            assert_eq!(
                step_output_at(&pool, run_id, "merge", &format!("loop1[{iteration}]")).await,
                Some(json!([{ "branch": branch }])),
                "iteration {iteration} must carry the branch it took, not a leftover"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_connector_after_a_connector_outside_the_loop_still_reads_it() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-outer-source").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                flow_config("before", json!({ "scope": "outside" })),
                flow_loop("loop1", json!("{{ trigger.items }}")),
                disabled(http_get("inside", "http://127.0.0.1:1/never")),
            ],
            edges: vec![
                edge("before", "loop1", None),
                edge("before", "inside", None),
                edge("loop1", "inside", Some(Branch::Each)),
            ],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "items": [1, 2] }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        for iteration in 0..2 {
            assert_eq!(
                step_output_at(&pool, run_id, "inside", &format!("loop1[{iteration}]")).await,
                Some(json!([{ "scope": "outside" }, { "valeur": iteration + 1 }])),
                "the node receives what the connector before the loop sends, then the loop's item"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_condition_takes_its_then_branch_without_evaluating_its_predicate() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-condition").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                disabled(condition("c1", "{{ false }}")),
                condition("then_side", "{{ true }}"),
                condition("else_side", "{{ true }}"),
            ],
            edges: vec![
                edge("c1", "then_side", Some(Branch::Then)),
                edge("c1", "else_side", Some(Branch::Else)),
            ],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let payload = json!({ "go": "on" });
        let run_id = usecase
            .start_run(org_id, workflow_id, payload.clone())
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        assert_eq!(status_of(&steps, "c1"), Some("succeeded"));
        assert_eq!(status_of(&steps, "then_side"), Some("succeeded"));
        assert_eq!(status_of(&steps, "else_side"), Some("skipped"));
        assert_eq!(
            step_output(&pool, run_id, "c1").await,
            Some(json!({ "Then": [payload], "Else": [] }))
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_loop_iterates_nothing_and_carries_on_after_it() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-loop").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                disabled(flow_loop("loop1", json!("{{ trigger.items }}"))),
                condition("body", "{{ true }}"),
                condition("after", "{{ true }}"),
            ],
            edges: vec![
                edge("loop1", "body", Some(Branch::Each)),
                edge("loop1", "after", Some(Branch::After)),
            ],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({ "items": [1, 2, 3] }))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        assert_eq!(status_of(&steps, "loop1"), Some("succeeded"));
        assert_eq!(status_of(&steps, "body"), None, "{steps:?}");
        assert_eq!(status_of(&steps, "after"), Some("succeeded"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn resuming_after_a_disabled_condition_keeps_following_its_main_branch() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "disabled-resume").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                condition("c0", "{{ false }}"),
                condition("unused", "{{ true }}"),
                disabled(condition("c1", "{{ true }}")),
                condition("then_side", "{{ true }}"),
                condition("else_side", "{{ true }}"),
            ],
            edges: vec![
                edge("c0", "unused", Some(Branch::Then)),
                edge("c0", "c1", Some(Branch::Else)),
                edge("c1", "then_side", Some(Branch::Then)),
                edge("c1", "else_side", Some(Branch::Else)),
            ],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", schedule_with_step_budget(2))
            .await
            .unwrap();
        assert_eq!(run_status(&pool, run_id).await, "pending");
        let first_pass = run_steps(&pool, run_id).await;
        assert_eq!(
            status_of(&first_pass, "c1"),
            Some("succeeded"),
            "{first_pass:?}"
        );
        assert_eq!(status_of(&first_pass, "then_side"), None, "{first_pass:?}");

        make_run_due_now(&pool, run_id).await;
        usecase
            .run_engine_pass(&connectors, "worker-2", generous_schedule())
            .await
            .unwrap();

        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        let steps = run_steps(&pool, run_id).await;
        assert_eq!(
            status_of(&steps, "then_side"),
            Some("succeeded"),
            "{steps:?}"
        );
        assert_eq!(status_of(&steps, "else_side"), Some("skipped"), "{steps:?}");
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
        let workflow_id = start_workflow(
            &usecase,
            org_id,
            Graph {
                connectors,
                edges,
                triggers: Vec::new(),
            },
        )
        .await;
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
            triggers: Vec::new(),
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
                graph: triggered(Graph {
                    connectors: vec![condition("only_v1", "{{ true }}")],
                    edges: vec![],
                    triggers: Vec::new(),
                }),
                layout: None,
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
                graph: triggered(Graph {
                    connectors: vec![condition("only_v2", "{{ true }}")],
                    edges: vec![],
                    triggers: Vec::new(),
                }),
                layout: None,
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
    async fn a_run_keeps_resolving_the_graph_it_pinned_after_the_workflow_is_edited_twice() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "graph-pinned").await;
        let usecase = use_case(pool.clone());

        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Edited twice".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let pinned_graph = Graph {
            connectors: vec![condition("only_v1", "{{ true }}")],
            edges: vec![],
            triggers: Vec::new(),
        };
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: triggered(pinned_graph.clone()),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        let run_id = usecase
            .start_run(org_id, workflow.id, json!({}))
            .await
            .unwrap();

        for label in ["only_v2", "only_v3"] {
            usecase
                .save_workflow_version(SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph: triggered(Graph {
                        connectors: vec![condition(label, "{{ true }}")],
                        edges: vec![],
                        triggers: Vec::new(),
                    }),
                    layout: None,
                    created_by: None,
                })
                .await
                .unwrap();
        }

        let run = usecase.find_run(org_id, run_id).await.unwrap().unwrap();
        let version = usecase
            .find_workflow_version_by_id(org_id, run.workflow_version_id)
            .await
            .unwrap()
            .expect("the run's pinned version still exists");

        assert_eq!(version.graph, triggered(pinned_graph));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_run_whose_pinned_version_has_been_deleted_still_reads() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "graph-deleted-version").await;
        let usecase = use_case(pool.clone());

        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Doomed version".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let version = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: triggered(Graph {
                    connectors: vec![condition("c1", "{{ true }}")],
                    edges: vec![],
                    triggers: Vec::new(),
                }),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();
        let run_id = usecase
            .start_run(org_id, workflow.id, json!({}))
            .await
            .unwrap();

        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("SET session_replication_role = replica")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("DELETE FROM automation.workflow_version WHERE id = $1")
            .bind(version.id)
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("SET session_replication_role = DEFAULT")
            .execute(&mut *conn)
            .await
            .unwrap();
        drop(conn);

        let run = usecase
            .find_run(org_id, run_id)
            .await
            .unwrap()
            .expect("the run itself is still readable");
        assert_eq!(run.workflow_version_id, version.id);

        let found_version = usecase
            .find_workflow_version_by_id(org_id, run.workflow_version_id)
            .await
            .unwrap();
        assert_eq!(
            found_version, None,
            "the version is gone, and that must read back as absent, not as an error"
        );

        let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
        assert!(steps.is_empty(), "a run that never executed has no steps");
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
            triggers: Vec::new(),
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
    async fn starting_a_run_for_a_manual_workflow_succeeds() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "manual-run").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;

        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let found = usecase.find_run(org_id, run_id).await.unwrap();
        assert!(found.is_some());
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

    fn lifecycle_update(
        org_id: OrganizationId,
        id: Uuid,
    ) -> crate::domain::automation::workflow::UpdateWorkflowCommand {
        crate::domain::automation::workflow::UpdateWorkflowCommand {
            org_id,
            id,
            name: None,
            description: None,
            enabled: None,
            folder_id: None,
            tags: None,
            archived: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn starting_a_run_for_an_archived_workflow_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "start-archived").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        usecase
            .update_workflow(crate::domain::automation::workflow::UpdateWorkflowCommand {
                archived: Some(true),
                ..lifecycle_update(org_id, workflow_id)
            })
            .await
            .unwrap();

        let error = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .expect_err("an archived workflow cannot be started");

        assert!(matches!(&error, CoreError::Conflict(message)
            if crate::domain::automation::workflow::WorkflowRefusal::from_conflict(message)
                == Some(crate::domain::automation::workflow::WorkflowRefusal::Archived)));
        assert!(usecase.list_runs(org_id).await.unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_workflow_can_still_be_started_manually() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "start-disabled").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        usecase
            .update_workflow(crate::domain::automation::workflow::UpdateWorkflowCommand {
                enabled: Some(false),
                ..lifecycle_update(org_id, workflow_id)
            })
            .await
            .unwrap();

        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        assert!(usecase.find_run(org_id, run_id).await.unwrap().is_some());
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
        let input = steps[0].input.as_ref().expect("the input items are kept");
        assert_eq!(input, &json!([{}]));
        let output = steps[0].output.as_ref().expect("the output items are kept");
        assert_eq!(output[0]["name"], json!("Inspected Customer"));
        assert_eq!(output.as_array().map(Vec::len), Some(1));
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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
        assert_eq!(replayed.run.status, RunStatus::Pending);
        assert_eq!(replayed.run.id, run_id);
        assert_eq!(replayed.number, 1, "a replay keeps the run's number");
        assert_eq!(replayed.steps_total, 3);
        assert_eq!(
            replayed.steps_done, 1,
            "the replayed step and its descendants were reset in the same transaction"
        );

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
            triggers: Vec::new(),
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
    async fn replaying_a_run_of_an_archived_workflow_is_refused_and_keeps_every_step() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay-archived").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![customer_create("c1", "A"), customer_create("c2", "B")],
            edges: vec![edge("c1", "c2", None)],
            triggers: Vec::new(),
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
        usecase
            .update_workflow(crate::domain::automation::workflow::UpdateWorkflowCommand {
                archived: Some(true),
                ..lifecycle_update(org_id, workflow_id)
            })
            .await
            .unwrap();

        let error = usecase
            .replay_run_from_step(org_id, run_id, "c1".to_string())
            .await
            .expect_err("an archived workflow cannot be replayed");

        assert!(matches!(&error, CoreError::Conflict(message)
            if crate::domain::automation::workflow::WorkflowRefusal::from_conflict(message)
                == Some(crate::domain::automation::workflow::WorkflowRefusal::Archived)));
        assert_eq!(run_status(&pool, run_id).await, "succeeded");
        assert_eq!(run_steps(&pool, run_id).await.len(), 2);
        assert_eq!(customer_count(&pool, org_id).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_a_run_of_a_merely_disabled_workflow_is_allowed() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "replay-disabled").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![condition("c1", "{{ true }}")],
            edges: vec![],
            triggers: Vec::new(),
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
        usecase
            .update_workflow(crate::domain::automation::workflow::UpdateWorkflowCommand {
                enabled: Some(false),
                ..lifecycle_update(org_id, workflow_id)
            })
            .await
            .unwrap();

        let replayed = usecase
            .replay_run_from_step(org_id, run_id, "c1".to_string())
            .await
            .unwrap();

        assert_eq!(replayed.run.status, RunStatus::Pending);
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
            triggers: Vec::new(),
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
            triggers: Vec::new(),
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

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_downstream_connector_receives_the_url_resolved_from_an_upstream_configs_output() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "config-http").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                named(
                    flow_config(
                        "c2",
                        json!({ "firstname": "John", "lastname": "Doe", "id": "1" }),
                    ),
                    "Réglages",
                ),
                customer_create(
                    "c1",
                    "{{ $('Réglages').item.json.firstname }} {{ $('Réglages').item.json.lastname }} #{{ $('Réglages').item.json.id }}",
                ),
            ],
            edges: vec![edge("c2", "c1", None)],
            triggers: Vec::new(),
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
        let customer_step = steps
            .iter()
            .find(|s| s.connector_id == "c1")
            .expect("the customer step has its own row");
        let output = customer_step
            .output
            .as_ref()
            .expect("an output is recorded for a step that ran");
        assert_eq!(
            output[0]["name"],
            json!("John Doe #1"),
            "the connector must receive the values resolved from flow.config's item"
        );
        assert_eq!(
            customer_step.input,
            Some(json!([{ "firstname": "John", "lastname": "Doe", "id": "1" }]))
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_resumed_run_still_resolves_an_earlier_configs_output_for_a_later_step() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "config-http-resume").await;
        let usecase = use_case(pool.clone());
        let graph = Graph {
            connectors: vec![
                named(flow_config("c2", json!({ "id": "2" })), "Réglages"),
                customer_create("c1", "todo {{ $('Réglages').item.json.id }}"),
            ],
            edges: vec![edge("c2", "c1", None)],
            triggers: Vec::new(),
        };
        let workflow_id = start_workflow(&usecase, org_id, graph).await;
        let run_id = usecase
            .start_run(org_id, workflow_id, json!({}))
            .await
            .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", schedule_with_step_budget(1))
            .await
            .unwrap();

        assert_eq!(
            run_status(&pool, run_id).await,
            "pending",
            "only the budgeted first step ran"
        );
        let after_first = run_steps(&pool, run_id).await;
        let c2_first = after_first
            .iter()
            .find(|s| s.connector_id == "c2")
            .expect("c2 ran in the first slice");
        assert_eq!(c2_first.status, "succeeded");
        assert!(
            after_first.iter().all(|s| s.connector_id != "c1"),
            "c1 must not have started yet: {after_first:?}"
        );

        make_run_due_now(&pool, run_id).await;
        usecase
            .run_engine_pass(&connectors, "worker-1", schedule_with_step_budget(1))
            .await
            .unwrap();

        let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
        let customer_step = steps
            .iter()
            .find(|s| s.connector_id == "c1")
            .expect("the customer step ran on the resumed pass");
        let output = customer_step
            .output
            .as_ref()
            .expect("an output is recorded for a step that ran");
        assert_eq!(
            output[0]["name"],
            json!("todo 2"),
            "the resumed pass must resolve c2's items the same way the first pass would"
        );
    }

    mod collection_operation_runs {
        use serde_json::Map;

        use super::*;
        use crate::domain::automation::collection::{
            CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod,
            ImportCollectionCommand, Operation, Parameter, ParameterLocation,
            ReplaceCollectionCommand,
        };
        use crate::domain::automation::workflow::OperationRef;
        use crate::infrastructure::automation::connectors::test_support::{
            StubResponse, TestServer,
        };

        fn draft(base_url: &str, path: &str) -> CollectionDraft {
            CollectionDraft::new(CollectionDraftInput {
                title: "Addresses".to_owned(),
                version: "1.0.0".to_owned(),
                description: String::new(),
                base_url: base_url.to_owned(),
                source: CollectionSource::Url {
                    url: "https://api.example.com/openapi.json".to_owned(),
                },
                security: Vec::new(),
                operations: vec![Operation {
                    id: "getAddress".to_owned(),
                    method: HttpMethod::Get,
                    path: path.to_owned(),
                    summary: String::new(),
                    tag: String::new(),
                    parameters: vec![Parameter {
                        name: "id".to_owned(),
                        location: ParameterLocation::Path,
                        required: true,
                        description: String::new(),
                        schema: json!({ "type": "string" }),
                    }],
                    request_body: None,
                    response: None,
                    response_status: None,
                }],
                schemas: Map::new(),
            })
            .unwrap()
        }

        fn operation_node(collection_id: Uuid) -> PlacedConnector {
            PlacedConnector {
                id: "c1".to_string(),
                kind: "collection.operation".to_string(),
                version: 1,
                credential_id: None,
                config: json!({ "path": { "id": "7" } })
                    .as_object()
                    .cloned()
                    .unwrap(),
                name: None,
                description: None,
                disabled: false,
                operation: Some(OperationRef {
                    collection_id,
                    operation_id: "getAddress".to_owned(),
                }),
            }
        }

        fn registry_reaching(usecase: &MestierUseCase) -> ConnectorRegistry {
            ConnectorRegistry::new(usecase.clone()).with_collection_client(
                usecase.clone(),
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .unwrap(),
            )
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn the_engine_runs_the_stored_operation_and_follows_an_update_with_no_new_version() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "engine-collection").await;
            let usecase = use_case(pool.clone());
            let server =
                TestServer::respond_always(StubResponse::json(200, &json!({ "street": "x" })))
                    .await;
            let stored = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft(&server.url, "/addresses/{id}"),
                })
                .await
                .unwrap();
            let graph = Graph {
                connectors: vec![operation_node(stored.id)],
                edges: Vec::new(),
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            let connectors = registry_reaching(&usecase);

            let first = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, first).await;
            usecase
                .run_engine_pass(&connectors, "worker-1", generous_schedule())
                .await
                .unwrap();
            usecase
                .replace_collection(ReplaceCollectionCommand {
                    org_id,
                    id: stored.id,
                    draft: draft(&server.url, "/v2/places/{id}"),
                })
                .await
                .unwrap();
            let second = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, second).await;
            usecase
                .run_engine_pass(&connectors, "worker-1", generous_schedule())
                .await
                .unwrap();

            assert_eq!(run_status(&pool, first).await, "succeeded");
            assert_eq!(run_status(&pool, second).await, "succeeded");
            let output = step_output(&pool, first, "c1").await.unwrap();
            assert_eq!(output[0]["status"], 200);
            assert_eq!(output[0]["body"], json!({ "street": "x" }));
            let paths: Vec<String> = server.requests().into_iter().map(|r| r.path).collect();
            assert_eq!(paths, ["/addresses/7", "/v2/places/7"]);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn expressions_nested_in_the_node_are_resolved_for_the_call() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "engine-collection-nested").await;
            let usecase = use_case(pool.clone());
            let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
            let stored = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft(&server.url, "/addresses/{id}"),
                })
                .await
                .unwrap();
            let mut node = operation_node(stored.id);
            node.config = json!({ "path": { "id": "{{ trigger.id }}" } })
                .as_object()
                .cloned()
                .unwrap();
            let graph = Graph {
                connectors: vec![node],
                edges: Vec::new(),
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({ "id": "from-the-trigger" }))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &registry_reaching(&usecase),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(server.last_request().path, "/addresses/from-the-trigger");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn another_connector_keeps_its_nested_text_literal() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "engine-http-literal").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![PlacedConnector {
                    id: "c1".to_string(),
                    kind: "flow.config".to_string(),
                    version: 1,
                    credential_id: None,
                    config: json!({ "variables": { "name": "{{ trigger.id }}" } })
                        .as_object()
                        .cloned()
                        .unwrap(),
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                }],
                edges: Vec::new(),
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({ "id": "42" }))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();

            let output = step_output(&pool, run_id, "c1").await.unwrap();
            assert_eq!(output, json!([{ "name": "{{ trigger.id }}" }]));
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_run_whose_collection_was_deleted_fails_its_step_naming_the_collection() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "engine-collection-gone").await;
            let usecase = use_case(pool.clone());
            let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
            let stored = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft(&server.url, "/addresses/{id}"),
                })
                .await
                .unwrap();
            let graph = Graph {
                connectors: vec![operation_node(stored.id)],
                edges: Vec::new(),
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            usecase.delete_collection(org_id, stored.id).await.unwrap();
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &registry_reaching(&usecase),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(steps.len(), 1);
            assert_eq!(steps[0].status, "failed");
            let error = steps[0].error.clone().unwrap();
            assert!(error.contains(&stored.id.to_string()), "{error}");
            assert!(server.requests().is_empty());
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_base_url_on_a_refused_address_fails_the_run_at_run_time_with_a_named_error() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "engine-collection-private").await;
            let usecase = use_case(pool.clone());
            let stored = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft("http://10.255.255.1:9", "/addresses/{id}"),
                })
                .await
                .unwrap();
            let graph = Graph {
                connectors: vec![operation_node(stored.id)],
                edges: Vec::new(),
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(steps[0].status, "failed");
            let error = steps[0].error.clone().unwrap();
            assert!(
                error.contains("10.255.255.1") && error.contains("private"),
                "{error}"
            );
        }
    }

    mod parked_runs {
        use chrono::Duration as ChronoDuration;

        use super::*;

        fn wait(id: &str, duration: i64, unit: &str) -> PlacedConnector {
            let mut config = serde_json::Map::new();
            config.insert("duration".to_string(), json!(duration));
            config.insert("unit".to_string(), json!(unit));
            PlacedConnector {
                id: id.to_string(),
                kind: "flow.wait".to_string(),
                version: 1,
                credential_id: None,
                config,
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }
        }

        fn merge(id: &str, mode: &str, field: Option<&str>) -> PlacedConnector {
            let mut config = serde_json::Map::new();
            config.insert("mode".to_string(), json!(mode));
            if let Some(field) = field {
                config.insert("field".to_string(), json!(field));
            }
            PlacedConnector {
                id: id.to_string(),
                kind: "flow.merge".to_string(),
                version: 1,
                credential_id: None,
                config,
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }
        }

        struct StepState {
            status: String,
            resume_at: Option<DateTime<Utc>>,
            attempts: i32,
        }

        async fn step_state(pool: &PgPool, run_id: Uuid, connector_id: &str) -> Option<StepState> {
            sqlx::query!(
                r#"SELECT status, resume_at, attempts FROM automation.run_step
                   WHERE run_id = $1 AND connector_id = $2"#,
                run_id,
                connector_id,
            )
            .fetch_optional(pool)
            .await
            .unwrap()
            .map(|row| StepState {
                status: row.status,
                resume_at: row.resume_at,
                attempts: row.attempts,
            })
        }

        struct RunState {
            status: String,
            locked_by: Option<String>,
            locked_at: Option<DateTime<Utc>>,
            next_attempt_at: Option<DateTime<Utc>>,
        }

        async fn run_state(pool: &PgPool, run_id: Uuid) -> RunState {
            let row = sqlx::query!(
                "SELECT status, locked_by, locked_at, next_attempt_at FROM automation.run WHERE id = $1",
                run_id,
            )
            .fetch_one(pool)
            .await
            .unwrap();
            RunState {
                status: row.status,
                locked_by: row.locked_by,
                locked_at: row.locked_at,
                next_attempt_at: row.next_attempt_at,
            }
        }

        async fn resume_time_has_come(pool: &PgPool, run_id: Uuid, connector_id: &str) {
            sqlx::query!(
                "UPDATE automation.run_step SET resume_at = now() - interval '1 second'
                 WHERE run_id = $1 AND connector_id = $2 AND status = 'waiting'",
                run_id,
                connector_id,
            )
            .execute(pool)
            .await
            .unwrap();
            make_run_due_now(pool, run_id).await;
        }

        async fn pass(usecase: &MestierUseCase) {
            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();
        }

        async fn start(usecase: &MestierUseCase, org_id: OrganizationId, graph: Graph) -> Uuid {
            let workflow_id = start_workflow(usecase, org_id, graph).await;
            usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap()
        }

        fn wait_graph() -> Graph {
            Graph {
                connectors: vec![
                    flow_config("c1", json!({ "before": true })),
                    wait("w1", 2, "days"),
                    flow_config("c3", json!({ "after": true })),
                ],
                edges: vec![edge("c1", "w1", None), edge("w1", "c3", None)],
                triggers: Vec::new(),
            }
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_wait_parks_its_step_holds_no_worker_and_the_run_continues_after_it() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "wait-park").await;
            let usecase = use_case(pool.clone());
            let run_id = start(&usecase, org_id, wait_graph()).await;

            pass(&usecase).await;

            let parked = step_state(&pool, run_id, "w1").await.unwrap();
            assert_eq!(parked.status, "waiting");
            let resume_at = parked.resume_at.expect("a parked step has a resume time");
            let expected = Utc::now() + ChronoDuration::days(2);
            assert!(
                (resume_at - expected).num_seconds().abs() < 300,
                "{resume_at}"
            );
            assert_eq!(parked.attempts, 1);
            assert_eq!(
                step_state(&pool, run_id, "c1").await.unwrap().status,
                "succeeded"
            );
            assert!(step_state(&pool, run_id, "c3").await.is_none());
            let output = step_output(&pool, run_id, "w1").await.unwrap();
            assert_eq!(output, json!([{ "before": true }]));

            let run = run_state(&pool, run_id).await;
            assert_eq!(run.status, "running");
            assert_eq!(run.locked_by, None);
            assert_eq!(run.locked_at, None);
            assert_eq!(run.next_attempt_at, Some(resume_at));

            pass(&usecase).await;
            assert_eq!(
                step_state(&pool, run_id, "w1").await.unwrap().status,
                "waiting",
                "a pass before the resume time leaves the step parked"
            );

            resume_time_has_come(&pool, run_id, "w1").await;
            pass(&usecase).await;

            let resumed = step_state(&pool, run_id, "w1").await.unwrap();
            assert_eq!(resumed.status, "succeeded");
            assert_eq!(resumed.resume_at, None);
            assert_eq!(
                resumed.attempts, 1,
                "resuming never executes the wait again"
            );
            assert_eq!(
                step_output(&pool, run_id, "w1").await.unwrap(),
                json!([{ "before": true }])
            );
            assert_eq!(
                step_state(&pool, run_id, "c3").await.unwrap().status,
                "succeeded"
            );
            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(run_steps(&pool, run_id).await.len(), 3);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_run_claimed_before_its_parked_step_is_due_goes_back_to_sleep_until_that_step() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "wait-early").await;
            let usecase = use_case(pool.clone());
            let run_id = start(&usecase, org_id, wait_graph()).await;
            pass(&usecase).await;
            let resume_at = step_state(&pool, run_id, "w1").await.unwrap().resume_at;

            make_run_due_now(&pool, run_id).await;
            pass(&usecase).await;

            assert_eq!(
                step_state(&pool, run_id, "w1").await.unwrap().status,
                "waiting"
            );
            assert!(step_state(&pool, run_id, "c3").await.is_none());
            let run = run_state(&pool, run_id).await;
            assert_eq!(run.status, "running");
            assert_eq!(run.locked_by, None);
            assert_eq!(run.next_attempt_at, resume_at);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn another_branch_runs_on_while_a_step_is_parked() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "wait-branch").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("a", json!({ "a": 1 })),
                    wait("w", 1, "hours"),
                    flow_config("x", json!({ "x": 1 })),
                    flow_config("b", json!({ "b": 1 })),
                    flow_config("y", json!({ "y": 1 })),
                ],
                edges: vec![
                    edge("a", "w", None),
                    edge("w", "x", None),
                    edge("b", "y", None),
                ],
                triggers: Vec::new(),
            };
            let run_id = start(&usecase, org_id, graph).await;

            pass(&usecase).await;

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "w"), Some("waiting"));
            assert_eq!(status_of(&steps, "y"), Some("succeeded"));
            assert_eq!(status_of(&steps, "x"), None);
            assert_eq!(run_status(&pool, run_id).await, "running");

            resume_time_has_come(&pool, run_id, "w").await;
            pass(&usecase).await;

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "x"), Some("succeeded"));
            assert_eq!(run_status(&pool, run_id).await, "succeeded");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_worker_lost_after_parking_leaves_the_step_parked_and_it_resumes_once() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "wait-lost-worker").await;
            let usecase = use_case(pool.clone());
            let run_id = start(&usecase, org_id, wait_graph()).await;
            pass(&usecase).await;
            let before = step_state(&pool, run_id, "w1").await.unwrap();

            sqlx::query!(
                "UPDATE automation.run
                 SET locked_by = 'worker-doomed', locked_at = now() - interval '1 hour'
                 WHERE id = $1",
                run_id,
            )
            .execute(&pool)
            .await
            .unwrap();
            let released = usecase
                .recover_lost_runs(Utc::now() - ChronoDuration::minutes(5))
                .await
                .unwrap();
            assert!(released >= 1);
            assert_eq!(run_status(&pool, run_id).await, "pending");
            pass(&usecase).await;

            let after = step_state(&pool, run_id, "w1").await.unwrap();
            assert_eq!(after.status, "waiting");
            assert_eq!(after.resume_at, before.resume_at);
            assert_eq!(after.attempts, 1);
            assert_eq!(run_state(&pool, run_id).await.status, "running");

            resume_time_has_come(&pool, run_id, "w1").await;
            pass(&usecase).await;
            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(step_state(&pool, run_id, "w1").await.unwrap().attempts, 1);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_run_in_progress_is_not_replayed_and_a_finished_one_parks_again_from_the_wait() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "wait-replay").await;
            let usecase = use_case(pool.clone());
            let run_id = start(&usecase, org_id, wait_graph()).await;
            pass(&usecase).await;

            let refused = usecase
                .replay_run_from_step(org_id, run_id, "w1".to_string())
                .await
                .unwrap_err();
            assert!(matches!(refused, CoreError::Conflict(_)), "{refused:?}");
            assert_eq!(
                step_state(&pool, run_id, "w1").await.unwrap().status,
                "waiting"
            );

            resume_time_has_come(&pool, run_id, "w1").await;
            pass(&usecase).await;
            assert_eq!(run_status(&pool, run_id).await, "succeeded");

            usecase
                .replay_run_from_step(org_id, run_id, "w1".to_string())
                .await
                .unwrap();
            assert!(step_state(&pool, run_id, "w1").await.is_none());
            assert!(step_state(&pool, run_id, "c3").await.is_none());
            pass(&usecase).await;

            let parked = step_state(&pool, run_id, "w1").await.unwrap();
            assert_eq!(parked.status, "waiting");
            assert!(parked.resume_at.unwrap() > Utc::now() + ChronoDuration::days(1));
            assert_eq!(step_state(&pool, run_id, "c1").await.unwrap().attempts, 1);
            assert_eq!(run_state(&pool, run_id).await.status, "running");
        }

        fn two_sources(mode: &str, field: Option<&str>) -> Graph {
            Graph {
                connectors: vec![
                    flow_config("a", json!({ "id": 1, "name": "A" })),
                    flow_config("b", json!({ "id": 1, "total": 10 })),
                    merge("m", mode, field),
                ],
                edges: vec![edge("a", "m", None), edge("b", "m", None)],
                triggers: Vec::new(),
            }
        }

        async fn merged_items(
            usecase: &MestierUseCase,
            pool: &PgPool,
            org_id: OrganizationId,
            graph: Graph,
        ) -> Value {
            let run_id = start(usecase, org_id, graph).await;
            pass(usecase).await;
            assert_eq!(run_status(pool, run_id).await, "succeeded");
            step_output(pool, run_id, "m").await.unwrap()
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_merge_appends_combines_by_position_and_combines_by_field_its_incoming_branches()
        {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "merge-modes").await;
            let usecase = use_case(pool.clone());

            let appended = merged_items(&usecase, &pool, org_id, two_sources("append", None)).await;
            let by_position = merged_items(
                &usecase,
                &pool,
                org_id,
                two_sources("combine_by_position", None),
            )
            .await;
            let by_field = merged_items(
                &usecase,
                &pool,
                org_id,
                two_sources("combine_by_field", Some("id")),
            )
            .await;

            assert_eq!(
                appended,
                json!([{ "id": 1, "name": "A" }, { "id": 1, "total": 10 }])
            );
            assert_eq!(by_position, json!([{ "id": 1, "name": "A", "total": 10 }]));
            assert_eq!(by_field, json!([{ "id": 1, "name": "A", "total": 10 }]));
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_merge_ignores_a_skipped_predecessor() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "merge-skipped").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    condition("cond", "{{ true }}"),
                    flow_config("a", json!({ "side": "then" })),
                    flow_config("b", json!({ "side": "else" })),
                    merge("m", "append", None),
                ],
                edges: vec![
                    edge("cond", "a", Some(Branch::Then)),
                    edge("cond", "b", Some(Branch::Else)),
                    edge("a", "m", None),
                    edge("b", "m", None),
                ],
                triggers: Vec::new(),
            };

            let items = merged_items(&usecase, &pool, org_id, graph).await;

            assert_eq!(items, json!([{ "side": "then" }]));
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_merge_does_not_run_early_while_a_predecessor_sits_behind_a_parked_step() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "merge-behind-parked").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("a", json!({ "from": "a" })),
                    wait("w", 1, "hours"),
                    flow_config("b", json!({ "from": "b" })),
                    merge("m", "append", None),
                ],
                edges: vec![
                    edge("a", "m", None),
                    edge("w", "b", None),
                    edge("b", "m", None),
                ],
                triggers: Vec::new(),
            };
            let run_id = start(&usecase, org_id, graph).await;

            pass(&usecase).await;

            assert_eq!(
                step_state(&pool, run_id, "a").await.unwrap().status,
                "succeeded"
            );
            assert!(step_state(&pool, run_id, "m").await.is_none());
            assert_eq!(run_status(&pool, run_id).await, "running");

            resume_time_has_come(&pool, run_id, "w").await;
            pass(&usecase).await;

            let items = step_output(&pool, run_id, "m").await.unwrap();
            assert_eq!(items, json!([{ "from": "a" }, { "from": "b" }]));
            assert_eq!(run_status(&pool, run_id).await, "succeeded");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_merge_waits_for_a_parked_predecessor_then_merges_in_edge_order() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "merge-parked").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("a", json!({ "from": "a" })),
                    wait("w", 1, "minutes"),
                    flow_config("b", json!({ "from": "b" })),
                    merge("m", "append", None),
                ],
                edges: vec![
                    edge("a", "w", None),
                    edge("b", "m", None),
                    edge("w", "m", None),
                ],
                triggers: Vec::new(),
            };
            let run_id = start(&usecase, org_id, graph).await;

            pass(&usecase).await;

            assert_eq!(
                step_state(&pool, run_id, "b").await.unwrap().status,
                "succeeded"
            );
            assert_eq!(
                step_state(&pool, run_id, "w").await.unwrap().status,
                "waiting"
            );
            assert!(step_state(&pool, run_id, "m").await.is_none());
            assert_eq!(run_status(&pool, run_id).await, "running");

            resume_time_has_come(&pool, run_id, "w").await;
            pass(&usecase).await;

            let items = step_output(&pool, run_id, "m").await.unwrap();
            assert_eq!(items, json!([{ "from": "b" }, { "from": "a" }]));
            assert_eq!(run_status(&pool, run_id).await, "succeeded");
        }
    }

    mod items_runs {
        use super::*;
        use crate::domain::automation::workflow::{PlacedTrigger, TriggerKind};

        fn merge_append(id: &str) -> PlacedConnector {
            let mut config = serde_json::Map::new();
            config.insert("mode".to_string(), json!("append"));
            PlacedConnector {
                id: id.to_string(),
                kind: "flow.merge".to_string(),
                version: 1,
                credential_id: None,
                config,
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }
        }

        fn quote_search(id: &str) -> PlacedConnector {
            PlacedConnector {
                id: id.to_string(),
                kind: "mestier.quote.search".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }
        }

        fn three_sources() -> (Vec<PlacedConnector>, Vec<Edge>) {
            (
                vec![
                    named(flow_config("a", json!({ "n": 1 })), "A"),
                    named(flow_config("b", json!({ "n": 5 })), "B"),
                    named(flow_config("c", json!({ "n": 9 })), "C"),
                    named(merge_append("m"), "Fusion"),
                ],
                vec![
                    edge("a", "m", None),
                    edge("b", "m", None),
                    edge("c", "m", None),
                ],
            )
        }

        async fn run_once(
            usecase: &MestierUseCase,
            pool: &PgPool,
            org_id: OrganizationId,
            graph: Graph,
            payload: Value,
        ) -> Uuid {
            let workflow_id = start_workflow(usecase, org_id, graph).await;
            let run_id = usecase
                .start_run(org_id, workflow_id, payload)
                .await
                .unwrap();
            make_run_due_now(pool, run_id).await;
            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();
            run_id
        }

        fn names(output: &Value) -> Vec<String> {
            output
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["name"].as_str().unwrap().to_owned())
                .collect()
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_node_fed_three_items_runs_three_times_and_produces_three_items() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "three-items").await;
            let usecase = use_case(pool.clone());
            let (mut connectors, mut edges) = three_sources();
            connectors.push(named(
                customer_create("create", "Client {{ $('Fusion').item.json.n }}"),
                "Création",
            ));
            edges.push(edge("m", "create", None));
            let graph = Graph {
                connectors,
                edges,
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(customer_count(&pool, org_id).await, 3);
            let merged = step_output(&pool, run_id, "m").await.unwrap();
            assert_eq!(merged, json!([{ "n": 1 }, { "n": 5 }, { "n": 9 }]));
            let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
            let created = steps.iter().find(|s| s.connector_id == "create").unwrap();
            assert_eq!(created.input, Some(merged));
            assert_eq!(
                names(created.output.as_ref().unwrap()),
                ["Client 1", "Client 5", "Client 9"]
            );
            assert_eq!(created.attempts, 1);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_condition_sends_each_item_to_the_branch_its_predicate_selects() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "split").await;
            let usecase = use_case(pool.clone());
            let (mut connectors, mut edges) = three_sources();
            connectors.extend([
                named(
                    condition("cond", "{{ $('Fusion').item.json.n > 4 }}"),
                    "Seuil",
                ),
                customer_create("big", "Gros {{ $('Seuil').item.json.n }}"),
                customer_create("small", "Petit {{ $('Seuil').item.json.n }}"),
            ]);
            edges.extend([
                edge("m", "cond", None),
                edge("cond", "big", Some(Branch::Then)),
                edge("cond", "small", Some(Branch::Else)),
            ]);
            let graph = Graph {
                connectors,
                edges,
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(
                step_output(&pool, run_id, "cond").await.unwrap(),
                json!({ "Then": [{ "n": 5 }, { "n": 9 }], "Else": [{ "n": 1 }] })
            );
            assert_eq!(
                names(&step_output(&pool, run_id, "big").await.unwrap()),
                ["Gros 5", "Gros 9"]
            );
            assert_eq!(
                names(&step_output(&pool, run_id, "small").await.unwrap()),
                ["Petit 1"]
            );
            assert_eq!(customer_count(&pool, org_id).await, 3);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_branch_that_gets_no_item_is_skipped_and_the_other_one_runs() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "split-empty").await;
            let usecase = use_case(pool.clone());
            let (mut connectors, mut edges) = three_sources();
            connectors.extend([
                named(
                    condition("cond", "{{ $('Fusion').item.json.n > 0 }}"),
                    "Seuil",
                ),
                customer_create("big", "Gros {{ $('Seuil').item.json.n }}"),
                customer_create("small", "Petit"),
            ]);
            edges.extend([
                edge("m", "cond", None),
                edge("cond", "big", Some(Branch::Then)),
                edge("cond", "small", Some(Branch::Else)),
            ]);
            let graph = Graph {
                connectors,
                edges,
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "big"), Some("succeeded"), "{steps:?}");
            assert_eq!(status_of(&steps, "small"), Some("skipped"), "{steps:?}");
            assert_eq!(customer_count(&pool, org_id).await, 3);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn with_no_item_a_per_item_node_does_not_run_and_a_once_node_does() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "zero-items").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    named(quote_search("search"), "Recherche"),
                    customer_create("create", "Jamais"),
                    flow_config("config", json!({ "ran": true })),
                ],
                edges: vec![
                    edge("search", "create", None),
                    edge("search", "config", None),
                ],
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(step_output(&pool, run_id, "search").await, Some(json!([])));
            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "create"), Some("succeeded"));
            assert_eq!(step_output(&pool, run_id, "create").await, Some(json!([])));
            assert_eq!(customer_count(&pool, org_id).await, 0);
            assert_eq!(
                step_output(&pool, run_id, "config").await,
                Some(json!([{ "ran": true }]))
            );
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_reference_to_a_node_that_produced_no_item_fails_the_step_naming_the_node() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "no-item").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    named(quote_search("search"), "Recherche"),
                    flow_config("config", json!("{{ $('Recherche').item.json }}")),
                ],
                edges: vec![edge("search", "config", None)],
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "config"), Some("failed"), "{steps:?}");
            let error = steps
                .iter()
                .find(|s| s.connector_id == "config")
                .and_then(|s| s.error.clone())
                .unwrap();
            assert!(error.contains("Recherche"), "{error}");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_stored_workflow_with_the_old_syntax_fails_clearly_when_run() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "legacy").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("source", json!({ "id": "1" })),
                    flow_config("target", json!("PLACEHOLDER")),
                ],
                edges: vec![edge("source", "target", None)],
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            sqlx::query(
                "UPDATE automation.workflow_version
                 SET graph = replace(graph::text, 'PLACEHOLDER', '{{ connectors.source.output.id }}')::jsonb
                 WHERE workflow_id = $1",
            )
            .bind(workflow_id)
            .execute(&pool)
            .await
            .unwrap();
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    generous_schedule(),
                )
                .await
                .unwrap();

            let steps = run_steps(&pool, run_id).await;
            assert_eq!(status_of(&steps, "target"), Some("failed"), "{steps:?}");
            let error = steps
                .iter()
                .find(|s| s.connector_id == "target")
                .and_then(|s| s.error.clone())
                .unwrap();
            assert!(
                error.contains("cette expression utilise l'ancienne syntaxe"),
                "{error}"
            );
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_trigger_is_reached_by_its_name_and_produces_one_item() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "trigger-name").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("first", json!("{{ $('Départ').item.json }}")),
                    flow_config("second", json!("{{ $('Départ').first().json }}")),
                ],
                edges: vec![edge("t1", "first", None), edge("first", "second", None)],
                triggers: vec![PlacedTrigger {
                    id: "t1".to_string(),
                    kind: TriggerKind::Manual,
                    name: Some("Départ".to_string()),
                    description: None,
                }],
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({ "quote": 7 })).await;

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(
                step_output(&pool, run_id, "first").await,
                Some(json!([{ "quote": 7 }]))
            );
            assert_eq!(
                step_output(&pool, run_id, "second").await,
                Some(json!([{ "quote": 7 }]))
            );
            let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
            let first = steps.iter().find(|s| s.connector_id == "first").unwrap();
            assert_eq!(first.input, Some(json!([{ "quote": 7 }])));
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_loop_iterates_the_items_of_a_node_and_each_iteration_reads_its_own_item() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "loop-items").await;
            let usecase = use_case(pool.clone());
            let (mut connectors, mut edges) = three_sources();
            connectors.extend([
                named(flow_loop("l", json!("{{ $('Fusion').all() }}")), "Boucle"),
                customer_create("body", "Tour {{ $('Boucle').item.json.n }}"),
                flow_config("done", json!("{{ $('Boucle').last().json }}")),
            ]);
            edges.extend([
                edge("m", "l", None),
                edge("l", "body", Some(Branch::Each)),
                edge("l", "done", Some(Branch::After)),
            ]);
            let graph = Graph {
                connectors,
                edges,
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            assert_eq!(
                step_output(&pool, run_id, "l").await.unwrap(),
                json!({
                    "Each": [{ "n": 1 }, { "n": 5 }, { "n": 9 }],
                    "After": [{ "n": 1 }, { "n": 5 }, { "n": 9 }],
                })
            );
            for (iteration, expected) in ["Tour 1", "Tour 5", "Tour 9"].into_iter().enumerate() {
                let output =
                    step_output_at(&pool, run_id, "body", &format!("l[{iteration}]")).await;
                assert_eq!(names(&output.unwrap()), [expected]);
            }
            assert_eq!(
                step_output(&pool, run_id, "done").await,
                Some(json!([{ "n": 9 }]))
            );
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_node_with_two_predecessors_runs_once_with_the_items_of_both() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "two-preds").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    named(flow_config("a", json!({ "from": "a" })), "A"),
                    named(flow_config("b", json!({ "from": "b" })), "B"),
                    customer_create(
                        "join",
                        "{{ $('A').item.json.from }}{{ $('B').item.json.from }}",
                    ),
                ],
                edges: vec![edge("a", "join", None), edge("b", "join", None)],
                triggers: Vec::new(),
            };

            let run_id = run_once(&usecase, &pool, org_id, graph, json!({})).await;

            let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
            let join: Vec<_> = steps.iter().filter(|s| s.connector_id == "join").collect();
            assert_eq!(join.len(), 1);
            assert_eq!(
                join[0].input,
                Some(json!([{ "from": "a" }, { "from": "b" }]))
            );
            assert_eq!(names(join[0].output.as_ref().unwrap()), ["ab", "ab"]);
            assert_eq!(customer_count(&pool, org_id).await, 2);
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_stored_value_that_is_not_a_list_still_advances_a_resumed_run() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_pool().await;
            let org_id = seed_organization(&pool, "legacy-output").await;
            let usecase = use_case(pool.clone());
            let graph = Graph {
                connectors: vec![
                    flow_config("first", json!({ "x": 1 })),
                    named(flow_config("second", json!({ "y": 2 })), "Second"),
                ],
                edges: vec![edge("first", "second", None)],
                triggers: Vec::new(),
            };
            let workflow_id = start_workflow(&usecase, org_id, graph).await;
            let run_id = usecase
                .start_run(org_id, workflow_id, json!({}))
                .await
                .unwrap();
            make_run_due_now(&pool, run_id).await;
            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-1",
                    schedule_with_step_budget(1),
                )
                .await
                .unwrap();
            sqlx::query(
                "UPDATE automation.run_step SET output = '{\"x\": 1}'::jsonb
                 WHERE run_id = $1 AND connector_id = 'first'",
            )
            .bind(run_id)
            .execute(&pool)
            .await
            .unwrap();
            make_run_due_now(&pool, run_id).await;

            usecase
                .run_engine_pass(
                    &ConnectorRegistry::new(usecase.clone()),
                    "worker-2",
                    generous_schedule(),
                )
                .await
                .unwrap();

            assert_eq!(run_status(&pool, run_id).await, "succeeded");
            let steps = usecase.list_run_steps(org_id, run_id).await.unwrap();
            let second = steps.iter().find(|s| s.connector_id == "second").unwrap();
            assert_eq!(second.input, Some(json!([{ "x": 1 }])));
        }
    }
}

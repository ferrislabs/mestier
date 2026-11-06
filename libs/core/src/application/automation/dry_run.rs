use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use common::{CoreError, OrganizationId};
use events::EventCatalogue;
use mestier_macros::transactional;
use serde_json::{Value, json};
use uuid::Uuid;

use super::run::LoopStep;
use crate::{
    application::MestierUseCase,
    domain::automation::{
        catalogue::event_catalogue,
        collection::{COLLECTION_OPERATION_KIND, Collection, NESTED_EXPRESSION_FIELDS},
        connector::{ConnectorCatalogue, connector_catalogue},
        credential::Credential,
        expression::{EvalBudget, ExpressionContext, LoopFrame, json_size},
        ports::{CollectionRepository, CredentialRepository},
        run::{
            ConnectorOutcome, DryRunDispatch, DryRunError, DryRunInput, DryRunLimit, DryRunStep,
            MAX_DRY_RUN_BYTES, MAX_DRY_RUN_DURATION, MAX_DRY_RUN_EDGES, MAX_DRY_RUN_FIELD_BYTES,
            MAX_DRY_RUN_LOOP_ITEMS, MAX_DRY_RUN_NODE_BYTES, MAX_DRY_RUN_NODES,
            MAX_DRY_RUN_PAYLOAD_BYTES, MAX_DRY_RUN_PENDING, MAX_DRY_RUN_STEPS, Next,
            StoredOperation, disabled_outcome, format_iteration_path, next_after,
            pass_through_output, resolve_config_nested_within, resolve_config_within,
        },
        workflow::{Branch, Graph, GraphError, PlacedConnector, TriggerKind, validate_graph},
    },
    infrastructure::automation::connectors::ConnectorDryRun,
};

pub(crate) enum Payloads<'a> {
    Given(&'a Value),
    Sampled(&'a [Value]),
}

impl Payloads<'_> {
    fn of(&self, trigger: Option<usize>) -> Option<&Value> {
        match self {
            Self::Given(payload) => Some(payload),
            Self::Sampled(per_trigger) => trigger.and_then(|index| per_trigger.get(index)),
        }
    }
}

enum Settled {
    Succeeded(usize),
    Skipped,
    Failed,
}

enum Flow {
    Continue,
    Failed(usize),
}

#[derive(Clone, Copy)]
struct Pending {
    node: usize,
    trigger: Option<usize>,
}

struct Scope {
    base: Vec<LoopStep>,
    path: Vec<LoopStep>,
    text: String,
    pending: Vec<Pending>,
    known: HashSet<usize>,
    settled: HashMap<usize, Settled>,
    remaining: VecDeque<(usize, Arc<Value>)>,
    each: Vec<usize>,
    loop_id: String,
    lineage: Option<usize>,
}

impl Scope {
    fn root(entries: &[Pending]) -> Self {
        Self {
            base: Vec::new(),
            path: Vec::new(),
            text: String::new(),
            pending: entries.to_vec(),
            known: entries.iter().map(|entry| entry.node).collect(),
            settled: HashMap::new(),
            remaining: VecDeque::new(),
            each: Vec::new(),
            loop_id: String::new(),
            lineage: None,
        }
    }

    fn start_iteration(&mut self) {
        let Some((index, item)) = self.remaining.pop_front() else {
            return;
        };
        self.path = self.base.clone();
        self.path.push(LoopStep {
            loop_id: self.loop_id.clone(),
            index,
            item,
        });
        let entries: Vec<(String, usize)> = self
            .path
            .iter()
            .map(|step| (step.loop_id.clone(), step.index))
            .collect();
        self.text = format_iteration_path(&entries);
        self.pending = self
            .each
            .iter()
            .map(|&node| Pending {
                node,
                trigger: self.lineage,
            })
            .collect();
        self.known = self.each.iter().copied().collect();
        self.settled.clear();
    }
}

struct Recorder {
    steps: Vec<DryRunStep>,
    spent: usize,
}

impl Recorder {
    fn push(&mut self, step: DryRunStep) -> Result<usize, DryRunError> {
        if self.steps.len() >= MAX_DRY_RUN_STEPS {
            return Err(DryRunError::TooManySteps {
                limit: MAX_DRY_RUN_STEPS,
            });
        }
        self.spent = self.spent.saturating_add(step.weight());
        if self.spent > MAX_DRY_RUN_BYTES {
            return Err(DryRunError::TooLarge {
                limit: MAX_DRY_RUN_BYTES,
            });
        }
        self.steps.push(step);
        Ok(self.steps.len() - 1)
    }
}

struct Shape<'g> {
    index: HashMap<&'g str, usize>,
    placed: Vec<&'g PlacedConnector>,
    succ: Vec<Vec<usize>>,
    preds: Vec<Vec<usize>>,
    loops_of: Vec<Vec<usize>>,
    blockers: Vec<Vec<usize>>,
}

fn reach_in_order(succ: &[Vec<usize>], starts: &[usize]) -> Vec<usize> {
    let mut seen: HashSet<usize> = HashSet::new();
    let mut order: Vec<usize> = Vec::new();
    let mut frontier: VecDeque<usize> = VecDeque::new();
    for &start in starts {
        if seen.insert(start) {
            order.push(start);
            frontier.push_back(start);
        }
    }
    while let Some(node) = frontier.pop_front() {
        for &next in &succ[node] {
            if seen.insert(next) {
                order.push(next);
                frontier.push_back(next);
            }
        }
    }
    order
}

impl<'g> Shape<'g> {
    fn new(graph: &'g Graph) -> Self {
        let mut index: HashMap<&str, usize> = HashMap::new();
        let mut placed: Vec<&PlacedConnector> = Vec::new();
        for connector in &graph.connectors {
            if !index.contains_key(connector.id.as_str()) {
                index.insert(connector.id.as_str(), placed.len());
                placed.push(connector);
            }
        }
        let mut succ: Vec<Vec<usize>> = vec![Vec::new(); placed.len()];
        let mut preds: Vec<Vec<usize>> = vec![Vec::new(); placed.len()];
        for edge in &graph.edges {
            let (Some(&from), Some(&to)) =
                (index.get(edge.from.as_str()), index.get(edge.to.as_str()))
            else {
                continue;
            };
            succ[from].push(to);
            if !preds[to].contains(&from) {
                preds[to].push(from);
            }
        }
        let mut loops_of: Vec<Vec<usize>> = vec![Vec::new(); placed.len()];
        for (looping, id) in placed.iter().map(|placed| placed.id.as_str()).enumerate() {
            let body: Vec<usize> = graph
                .edges
                .iter()
                .filter(|edge| edge.from == id && edge.branch == Some(Branch::Each))
                .filter_map(|edge| index.get(edge.to.as_str()).copied())
                .collect();
            if body.is_empty() {
                continue;
            }
            for member in reach_in_order(&succ, &body) {
                loops_of[member].push(looping);
            }
        }
        Self {
            blockers: vec![Vec::new(); placed.len()],
            index,
            placed,
            succ,
            preds,
            loops_of,
        }
    }

    fn limit_to(&mut self, entries: &[Pending]) {
        let starts: Vec<usize> = entries.iter().map(|entry| entry.node).collect();
        let reachable: HashSet<usize> = reach_in_order(&self.succ, &starts).into_iter().collect();
        self.blockers = (0..self.placed.len())
            .map(|node| {
                self.preds[node]
                    .iter()
                    .copied()
                    .filter(|pred| {
                        *pred != node
                            && reachable.contains(pred)
                            && self.loops_of[*pred]
                                .iter()
                                .all(|looping| self.loops_of[node].contains(looping))
                    })
                    .collect()
            })
            .collect();
    }
}

fn entries(
    graph: &Graph,
    shape: &Shape<'_>,
    trigger_id: Option<&str>,
) -> Result<Vec<Pending>, DryRunError> {
    let wanted: Vec<usize> = match trigger_id {
        Some(id) => {
            let position = graph
                .triggers
                .iter()
                .position(|trigger| trigger.id == id)
                .ok_or_else(|| DryRunError::UnknownTrigger {
                    trigger_id: id.to_owned(),
                })?;
            vec![position]
        }
        None => (0..graph.triggers.len()).collect(),
    };
    let mut seen: HashSet<usize> = HashSet::new();
    let mut found: Vec<Pending> = Vec::new();
    for trigger in wanted {
        let from = graph.triggers[trigger].id.as_str();
        for edge in graph.edges.iter().filter(|edge| edge.from == from) {
            let Some(&node) = shape.index.get(edge.to.as_str()) else {
                return Err(DryRunError::InvalidGraph(vec![GraphError::DanglingEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                }]));
            };
            if seen.insert(node) {
                found.push(Pending {
                    node,
                    trigger: Some(trigger),
                });
            }
        }
    }
    if trigger_id.is_none() {
        for node in 0..shape.placed.len() {
            if shape.preds[node].is_empty() && seen.insert(node) {
                found.push(Pending {
                    node,
                    trigger: None,
                });
            }
        }
    }
    Ok(found)
}

fn stored_operation<'a>(
    placed: &PlacedConnector,
    collections: &'a [Collection],
) -> Option<StoredOperation<'a>> {
    let reference = placed.operation.as_ref()?;
    let collection = collections
        .iter()
        .find(|collection| collection.id == reference.collection_id)?;
    let operation = collection.find_operation(&reference.operation_id)?;
    Some(StoredOperation {
        collection,
        operation,
    })
}

fn stored_output(outcome: &ConnectorOutcome) -> Result<Value, &str> {
    match outcome {
        ConnectorOutcome::Produced(output) | ConnectorOutcome::Branch { output, .. } => {
            Ok(output.clone())
        }
        ConnectorOutcome::Iterate { items } => Ok(json!({ "items": items })),
        ConnectorOutcome::Failed { error } => Err(error),
    }
}

struct Walk<'a, D> {
    dispatch: &'a D,
    graph: &'a Graph,
    catalogue: &'a ConnectorCatalogue,
    collections: &'a [Collection],
    payloads: Payloads<'a>,
    deadline: Instant,
    shape: Shape<'a>,
    recorder: Recorder,
    outputs: BTreeMap<String, Value>,
    scopes: Vec<Scope>,
}

impl<D: DryRunDispatch> Walk<'_, D> {
    fn run(mut self, entries: &[Pending]) -> Result<Vec<DryRunStep>, DryRunError> {
        self.scopes.push(Scope::root(entries));
        while let Some(depth) = self.scopes.len().checked_sub(1) {
            if self.scopes[depth].pending.is_empty() {
                if self.scopes[depth].remaining.is_empty() {
                    self.scopes.pop();
                } else {
                    self.scopes[depth].start_iteration();
                    self.ensure_room()?;
                }
                continue;
            }
            if Instant::now() >= self.deadline {
                return Err(DryRunError::TimedOut);
            }
            let position = self.pick(depth);
            let pending = self.scopes[depth].pending.remove(position);
            if let Flow::Failed(node) = self.visit(pending)? {
                self.sweep(node)?;
                break;
            }
        }
        Ok(self.recorder.steps)
    }

    fn ready(&self, node: usize) -> bool {
        self.shape.blockers[node].iter().all(|pred| {
            self.scopes
                .iter()
                .any(|scope| scope.settled.contains_key(pred))
        })
    }

    fn pick(&self, depth: usize) -> usize {
        self.scopes[depth]
            .pending
            .iter()
            .position(|pending| self.ready(pending.node))
            .unwrap_or(0)
    }

    fn ensure_room(&self) -> Result<(), DryRunError> {
        let queued: usize = self.scopes.iter().map(|scope| scope.pending.len()).sum();
        if queued > MAX_DRY_RUN_PENDING {
            return Err(DryRunError::TooManySteps {
                limit: MAX_DRY_RUN_PENDING,
            });
        }
        Ok(())
    }

    fn succeeded_output(&self, source: &str) -> Option<Value> {
        let node = *self.shape.index.get(source)?;
        let found = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.settled.get(&node))?;
        match found {
            Settled::Succeeded(position) => Some(
                self.recorder.steps[*position]
                    .output
                    .clone()
                    .unwrap_or(Value::Null),
            ),
            Settled::Skipped | Settled::Failed => None,
        }
    }

    fn settle(
        &mut self,
        node: usize,
        settled: Settled,
        step: DryRunStep,
    ) -> Result<usize, DryRunError> {
        let position = self.recorder.push(step)?;
        let depth = self.scopes.len() - 1;
        let settled = match settled {
            Settled::Succeeded(_) => Settled::Succeeded(position),
            other => other,
        };
        self.scopes[depth].settled.insert(node, settled);
        Ok(position)
    }

    fn fail(
        &mut self,
        node: usize,
        input: Option<Value>,
        error: String,
    ) -> Result<Flow, DryRunError> {
        let text = self.scopes[self.scopes.len() - 1].text.clone();
        let id = self.shape.placed[node].id.as_str();
        self.settle(
            node,
            Settled::Failed,
            DryRunStep::failed(id, &text, input, error),
        )?;
        Ok(Flow::Failed(node))
    }

    fn visit(&mut self, pending: Pending) -> Result<Flow, DryRunError> {
        let node = pending.node;
        let placed = self.shape.placed[node];
        let depth = self.scopes.len() - 1;
        let text = self.scopes[depth].text.clone();

        if placed.disabled {
            let output = pass_through_output(
                self.graph,
                &placed.id,
                |source| self.succeeded_output(source),
                self.payloads.of(pending.trigger),
            );
            let outcome = disabled_outcome(&placed.kind, output.clone());
            self.outputs
                .insert(placed.id.clone(), json!({ "output": output.clone() }));
            self.settle(
                node,
                Settled::Succeeded(0),
                DryRunStep::succeeded(&placed.id, &text, None, output),
            )?;
            self.advance(node, &outcome, pending.trigger)?;
            return Ok(Flow::Continue);
        }

        let resolved = {
            let scope = &self.scopes[depth];
            let loop_frame = scope.path.last().map(|step| LoopFrame {
                item: &step.item,
                index: step.index,
            });
            let ctx = ExpressionContext {
                trigger: self.payloads.of(pending.trigger),
                connectors: &self.outputs,
                loop_frame,
                now: Utc::now(),
            };
            let mut budget = EvalBudget::new(MAX_DRY_RUN_NODE_BYTES);
            if placed.kind == COLLECTION_OPERATION_KIND {
                resolve_config_nested_within(
                    &placed.config,
                    &ctx,
                    &NESTED_EXPRESSION_FIELDS,
                    &mut budget,
                )
            } else {
                resolve_config_within(&placed.config, &ctx, &mut budget)
            }
        };
        let config = match resolved {
            Ok(config) => config,
            Err(error) => return self.fail(node, None, error.to_string()),
        };

        let Some(descriptor) = self.catalogue.get(&placed.kind, placed.version) else {
            let message = format!(
                "no connector `{}` version {} in the catalogue",
                placed.kind, placed.version
            );
            return self.fail(node, Some(Value::Object(config)), message);
        };
        let answered = self.dispatch.dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: stored_operation(placed, self.collections),
            },
        );
        let Some(outcome) = answered else {
            let message = format!(
                "no connector implementation for `{}` version {}",
                placed.kind, placed.version
            );
            return self.fail(node, Some(Value::Object(config)), message);
        };

        match stored_output(&outcome) {
            Err(error) => {
                let error = error.to_owned();
                self.fail(node, Some(Value::Object(config)), error)
            }
            Ok(output) => {
                self.outputs
                    .insert(placed.id.clone(), json!({ "output": output.clone() }));
                self.settle(
                    node,
                    Settled::Succeeded(0),
                    DryRunStep::succeeded(&placed.id, &text, Some(Value::Object(config)), output),
                )?;
                self.advance(node, &outcome, pending.trigger)?;
                Ok(Flow::Continue)
            }
        }
    }

    fn index_of(&self, from: &str, to: &str) -> Result<usize, DryRunError> {
        self.shape.index.get(to).copied().ok_or_else(|| {
            DryRunError::InvalidGraph(vec![GraphError::DanglingEdge {
                from: from.to_owned(),
                to: to.to_owned(),
            }])
        })
    }

    fn discover(
        &mut self,
        from: &str,
        targets: &[&str],
        trigger: Option<usize>,
    ) -> Result<(), DryRunError> {
        let depth = self.scopes.len() - 1;
        for target in targets {
            let node = self.index_of(from, target)?;
            if self.scopes[depth].known.insert(node) {
                self.scopes[depth].pending.push(Pending { node, trigger });
                self.ensure_room()?;
            }
        }
        Ok(())
    }

    fn skip(&mut self, nodes: impl IntoIterator<Item = usize>) -> Result<(), DryRunError> {
        let depth = self.scopes.len() - 1;
        for node in nodes {
            if self.scopes[depth].settled.contains_key(&node) {
                continue;
            }
            self.scopes[depth]
                .pending
                .retain(|pending| pending.node != node);
            self.scopes[depth].known.insert(node);
            let text = self.scopes[depth].text.clone();
            let id = self.shape.placed[node].id.as_str();
            self.settle(node, Settled::Skipped, DryRunStep::skipped(id, &text))?;
        }
        Ok(())
    }

    fn advance(
        &mut self,
        node: usize,
        outcome: &ConnectorOutcome,
        trigger: Option<usize>,
    ) -> Result<(), DryRunError> {
        let from = self.shape.placed[node].id.as_str();
        match next_after(self.graph, from, outcome) {
            Next::Follow { targets, skipped } => {
                let mut untaken: Vec<usize> = skipped
                    .iter()
                    .filter_map(|id| self.shape.index.get(id).copied())
                    .collect();
                untaken.sort_unstable();
                self.skip(untaken)?;
                self.discover(from, &targets, trigger)
            }
            Next::Iterate { after, each, items } => {
                self.discover(from, &after, trigger)?;
                self.open_loop(from, &each, &after, items, trigger)
            }
        }
    }

    fn open_loop(
        &mut self,
        from: &str,
        each: &[&str],
        after: &[&str],
        items: &[Value],
        trigger: Option<usize>,
    ) -> Result<(), DryRunError> {
        let mut body: Vec<usize> = Vec::new();
        for target in each {
            let node = self.index_of(from, target)?;
            if !body.contains(&node) {
                body.push(node);
            }
        }
        if body.is_empty() {
            return Ok(());
        }
        if items.is_empty() {
            let mut kept: Vec<usize> = Vec::new();
            for target in after {
                kept.push(self.index_of(from, target)?);
            }
            let rejoined: HashSet<usize> = reach_in_order(&self.shape.succ, &kept)
                .into_iter()
                .collect();
            let unvisited = reach_in_order(&self.shape.succ, &body)
                .into_iter()
                .filter(|node| !rejoined.contains(node))
                .collect::<Vec<usize>>();
            return self.skip(unvisited);
        }

        let remaining: VecDeque<(usize, Arc<Value>)> = items
            .iter()
            .take(MAX_DRY_RUN_LOOP_ITEMS)
            .enumerate()
            .map(|(index, item)| (index, Arc::new(item.clone())))
            .collect();
        let base = self.scopes[self.scopes.len() - 1].path.clone();
        let mut scope = Scope::root(&[]);
        scope.base = base;
        scope.remaining = remaining;
        scope.each = body;
        scope.loop_id = from.to_owned();
        scope.lineage = trigger;
        scope.start_iteration();
        self.scopes.push(scope);
        self.ensure_room()
    }

    fn sweep(&mut self, failed: usize) -> Result<(), DryRunError> {
        let mut swept: HashSet<usize> = HashSet::new();
        for depth in (0..self.scopes.len()).rev() {
            let mut starts: Vec<usize> = self.scopes[depth]
                .pending
                .iter()
                .map(|pending| pending.node)
                .collect();
            if depth == self.scopes.len() - 1 {
                starts.extend(self.shape.succ[failed].iter().copied());
            }
            for node in reach_in_order(&self.shape.succ, &starts) {
                if self.scopes[depth].settled.contains_key(&node) || !swept.insert(node) {
                    continue;
                }
                let text = self.scopes[depth].text.clone();
                let id = self.shape.placed[node].id.as_str();
                self.recorder.push(DryRunStep::skipped(id, &text))?;
                self.scopes[depth].settled.insert(node, Settled::Skipped);
            }
        }
        Ok(())
    }
}

pub(crate) fn walk<D: DryRunDispatch>(
    dispatch: &D,
    graph: &Graph,
    catalogue: &ConnectorCatalogue,
    collections: &[Collection],
    trigger_id: Option<&str>,
    payloads: Payloads<'_>,
    deadline: Instant,
) -> Result<Vec<DryRunStep>, DryRunError> {
    let mut shape = Shape::new(graph);
    let starts = entries(graph, &shape, trigger_id)?;
    shape.limit_to(&starts);
    Walk {
        dispatch,
        graph,
        catalogue,
        collections,
        payloads,
        deadline,
        shape,
        recorder: Recorder {
            steps: Vec::new(),
            spent: 0,
        },
        outputs: BTreeMap::new(),
        scopes: Vec::new(),
    }
    .run(&starts)
}

fn ensure_within_limits(graph: &Graph, payload: Option<&Value>) -> Result<(), DryRunError> {
    if graph.connectors.len() + graph.triggers.len() > MAX_DRY_RUN_NODES {
        return Err(DryRunError::Oversized(DryRunLimit::Nodes));
    }
    if graph.edges.len() > MAX_DRY_RUN_EDGES {
        return Err(DryRunError::Oversized(DryRunLimit::Edges));
    }
    let oversized_field = graph
        .connectors
        .iter()
        .flat_map(|connector| connector.config.values())
        .any(|field| json_size(field) > MAX_DRY_RUN_FIELD_BYTES);
    if oversized_field {
        return Err(DryRunError::Oversized(DryRunLimit::Field));
    }
    if payload.is_some_and(|payload| json_size(payload) > MAX_DRY_RUN_PAYLOAD_BYTES) {
        return Err(DryRunError::Oversized(DryRunLimit::Payload));
    }
    Ok(())
}

fn sample_payloads(graph: &Graph, events: &EventCatalogue) -> Vec<Value> {
    graph
        .triggers
        .iter()
        .map(|trigger| match &trigger.kind {
            TriggerKind::Events(names) => names
                .first()
                .and_then(|name| events.descriptors().find(|event| event.name == name))
                .map(|event| event.payload_example.clone())
                .unwrap_or_else(|| json!({})),
            TriggerKind::Manual => json!({}),
        })
        .collect()
}

impl MestierUseCase {
    #[transactional(credential, collection)]
    async fn dry_run_reference_data(
        &self,
        org_id: OrganizationId,
        collection_ids: &[Uuid],
    ) -> Result<(Vec<Credential>, Vec<Collection>), CoreError> {
        let mut credentials = credential_repository;
        let mut collections = collection_repository;
        Ok((
            credentials.list_by_organization(org_id).await?,
            collections.find_many(org_id, collection_ids).await?,
        ))
    }

    pub async fn dry_run_graph(
        &self,
        org_id: OrganizationId,
        graph: &Graph,
        trigger_id: Option<&str>,
        trigger_payload: Option<&Value>,
    ) -> Result<Vec<DryRunStep>, DryRunError> {
        ensure_within_limits(graph, trigger_payload)?;
        let (credentials, collections) = self
            .dry_run_reference_data(org_id, &graph.referenced_collection_ids())
            .await?;
        let catalogue = connector_catalogue();
        let events = event_catalogue();
        validate_graph(graph, &catalogue, &credentials, &events, &collections)
            .map_err(DryRunError::InvalidGraph)?;

        let sampled = sample_payloads(graph, &events);
        let graph = graph.clone();
        let given = trigger_payload.cloned();
        let trigger_id = trigger_id.map(str::to_owned);
        let deadline = Instant::now() + MAX_DRY_RUN_DURATION;
        let walked = tokio::task::spawn_blocking(move || {
            let payloads = match &given {
                Some(payload) => Payloads::Given(payload),
                None => Payloads::Sampled(&sampled),
            };
            walk(
                &ConnectorDryRun,
                &graph,
                &catalogue,
                &collections,
                trigger_id.as_deref(),
                payloads,
                deadline,
            )
        })
        .await;
        match walked {
            Ok(result) => result,
            Err(error) => Err(DryRunError::Core(CoreError::Internal(format!(
                "the dry run did not complete: {error}"
            )))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use serde_json::Map;

    use super::*;
    use crate::domain::automation::{
        collection::{
            CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod, Operation,
            Parameter, ParameterLocation,
        },
        connector::{ConnectorDescriptor, connector_catalogue},
        run::{
            Connector, ConnectorInput, ConnectorOutcome, DryRunInput, DryRunStatus,
            MAX_DRY_RUN_BYTES, MAX_DRY_RUN_LOOP_ITEMS, MAX_DRY_RUN_STEPS,
        },
        workflow::{Branch, Edge, OperationRef, PlacedConnector, PlacedTrigger, TriggerKind},
    };
    use crate::infrastructure::automation::connectors::ConnectorDryRun;

    struct SpyConnector;

    impl Connector for SpyConnector {
        async fn execute(&self, _input: ConnectorInput<'_>) -> ConnectorOutcome {
            panic!("a dry run must never call execute")
        }
    }

    struct SpyDispatch {
        reached: RefCell<Vec<String>>,
    }

    impl SpyDispatch {
        fn new() -> Self {
            Self {
                reached: RefCell::new(Vec::new()),
            }
        }
    }

    impl DryRunDispatch for SpyDispatch {
        fn dry_run(
            &self,
            descriptor: &ConnectorDescriptor,
            input: &DryRunInput<'_>,
        ) -> Option<ConnectorOutcome> {
            self.reached.borrow_mut().push(descriptor.kind.to_owned());
            Some(SpyConnector::dry_run(descriptor, input))
        }
    }

    struct RecordingDispatch {
        reached: RefCell<Vec<String>>,
    }

    impl RecordingDispatch {
        fn new() -> Self {
            Self {
                reached: RefCell::new(Vec::new()),
            }
        }
    }

    impl DryRunDispatch for RecordingDispatch {
        fn dry_run(
            &self,
            descriptor: &ConnectorDescriptor,
            input: &DryRunInput<'_>,
        ) -> Option<ConnectorOutcome> {
            self.reached.borrow_mut().push(descriptor.kind.to_owned());
            ConnectorDryRun.dry_run(descriptor, input)
        }
    }

    fn node(id: &str, kind: &str, config: Value) -> PlacedConnector {
        PlacedConnector {
            id: id.to_owned(),
            kind: kind.to_owned(),
            version: 1,
            credential_id: None,
            config: config.as_object().cloned().expect("a config is an object"),
            name: None,
            description: None,
            disabled: false,
            operation: None,
        }
    }

    fn config_node(id: &str, variables: Value) -> PlacedConnector {
        node(id, "flow.config", json!({ "variables": variables }))
    }

    fn http_node(id: &str, url: &str) -> PlacedConnector {
        node(id, "http.request", json!({ "method": "GET", "url": url }))
    }

    fn customer_node(id: &str, name: &str) -> PlacedConnector {
        node(id, "mestier.customer.create", json!({ "name": name }))
    }

    fn condition_node(id: &str, predicate: &str) -> PlacedConnector {
        node(id, "flow.condition", json!({ "predicate": predicate }))
    }

    fn loop_node(id: &str, items: &str) -> PlacedConnector {
        node(id, "flow.loop", json!({ "items": items }))
    }

    fn disabled(mut connector: PlacedConnector) -> PlacedConnector {
        connector.disabled = true;
        connector
    }

    fn edge(from: &str, to: &str, branch: Option<Branch>) -> Edge {
        Edge {
            from: from.to_owned(),
            to: to.to_owned(),
            branch,
        }
    }

    fn trigger(id: &str) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_owned(),
            kind: TriggerKind::Manual,
            name: None,
            description: None,
        }
    }

    fn graph(connectors: Vec<PlacedConnector>, edges: Vec<Edge>) -> Graph {
        Graph {
            connectors,
            edges,
            triggers: vec![trigger("t1")],
        }
    }

    fn far_deadline() -> Instant {
        Instant::now() + std::time::Duration::from_secs(60)
    }

    fn walk_with<D: DryRunDispatch>(
        dispatch: &D,
        graph: &Graph,
        collections: &[Collection],
        trigger_id: Option<&str>,
        payload: &Value,
    ) -> Result<Vec<DryRunStep>, DryRunError> {
        walk(
            dispatch,
            graph,
            &connector_catalogue(),
            collections,
            trigger_id,
            Payloads::Given(payload),
            far_deadline(),
        )
    }

    fn walk_real(graph: &Graph, payload: &Value) -> Vec<DryRunStep> {
        walk_with(&ConnectorDryRun, graph, &[], None, payload).expect("the walk completes")
    }

    fn ids(steps: &[DryRunStep]) -> Vec<&str> {
        steps
            .iter()
            .map(|step| step.connector_id.as_str())
            .collect()
    }

    fn first<'a>(steps: &'a [DryRunStep], id: &str) -> &'a DryRunStep {
        steps
            .iter()
            .find(|step| step.connector_id == id)
            .unwrap_or_else(|| panic!("no step for `{id}` in {steps:?}"))
    }

    fn sample_of(kind: &str) -> Value {
        connector_catalogue()
            .get(kind, 1)
            .unwrap()
            .sample_output
            .clone()
    }

    #[test]
    fn an_unsaved_linear_graph_yields_one_step_per_node_in_walk_order() {
        let graph = graph(
            vec![
                config_node("c1", json!({ "n": 1 })),
                http_node("c2", "https://example.com/a"),
                customer_node("c3", "Ada"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "c3"]);
        assert!(
            steps
                .iter()
                .all(|step| step.status == DryRunStatus::Succeeded && step.error.is_none())
        );
        assert_eq!(first(&steps, "c1").output, Some(json!({ "n": 1 })));
        assert_eq!(
            first(&steps, "c1").input,
            Some(json!({ "variables": { "n": 1 } }))
        );
        assert_eq!(first(&steps, "c2").output, Some(sample_of("http.request")));
        assert_eq!(
            first(&steps, "c2").input,
            Some(json!({ "method": "GET", "url": "https://example.com/a" }))
        );
        assert_eq!(
            first(&steps, "c3").output,
            Some(sample_of("mestier.customer.create"))
        );
    }

    #[test]
    fn no_connector_but_the_dry_run_entry_is_ever_reached() {
        let graph = graph(
            vec![
                customer_node("c1", "Ada"),
                http_node("c2", "https://example.com/a"),
                node("c3", "odoo.create_partner", json!({ "name": "Ada" })),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        );
        let spy = SpyDispatch::new();

        let steps = walk_with(&spy, &graph, &[], None, &json!({})).expect("the walk completes");

        assert_eq!(
            *spy.reached.borrow(),
            [
                "mestier.customer.create",
                "http.request",
                "odoo.create_partner"
            ]
        );
        assert_eq!(steps.len(), 3);
        assert_eq!(
            first(&steps, "c3").output,
            Some(sample_of("odoo.create_partner"))
        );
    }

    #[test]
    fn a_condition_follows_the_branch_its_trigger_data_selects_and_skips_the_other() {
        let graph = graph(
            vec![
                condition_node("c1", "{{ trigger.paid }}"),
                customer_node("c2", "then"),
                customer_node("c3", "else"),
                customer_node("c4", "after then"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", Some(Branch::Then)),
                edge("c1", "c3", Some(Branch::Else)),
                edge("c2", "c4", None),
            ],
        );

        let unpaid = walk_real(&graph, &json!({ "paid": false }));
        let paid = walk_real(&graph, &json!({ "paid": true }));

        assert_eq!(ids(&unpaid), ["c1", "c2", "c4", "c3"]);
        assert_eq!(
            first(&unpaid, "c1").output,
            Some(json!({ "matched": false }))
        );
        assert_eq!(first(&unpaid, "c2").status, DryRunStatus::Skipped);
        assert_eq!(first(&unpaid, "c4").status, DryRunStatus::Skipped);
        assert_eq!(first(&unpaid, "c3").status, DryRunStatus::Succeeded);
        assert!(first(&unpaid, "c2").input.is_none() && first(&unpaid, "c2").output.is_none());

        assert_eq!(ids(&paid), ["c1", "c3", "c2", "c4"]);
        assert_eq!(first(&paid, "c3").status, DryRunStatus::Skipped);
        assert_eq!(first(&paid, "c2").status, DryRunStatus::Succeeded);
        assert_eq!(first(&paid, "c4").status, DryRunStatus::Succeeded);
    }

    #[test]
    fn a_condition_reads_the_sample_output_of_the_node_before_it() {
        let graph = graph(
            vec![
                http_node("c1", "https://example.com/a"),
                condition_node("c2", "{{ connectors.c1.output.status == 200 }}"),
                customer_node("c3", "then"),
                customer_node("c4", "else"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", Some(Branch::Then)),
                edge("c2", "c4", Some(Branch::Else)),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(first(&steps, "c2").output, Some(json!({ "matched": true })));
        assert_eq!(first(&steps, "c3").status, DryRunStatus::Succeeded);
        assert_eq!(first(&steps, "c4").status, DryRunStatus::Skipped);
    }

    fn loop_graph() -> Graph {
        graph(
            vec![
                loop_node("c1", "{{ trigger.items }}"),
                customer_node("c2", "{{ loop.item }}"),
                customer_node("c3", "done"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", Some(Branch::Each)),
                edge("c1", "c3", Some(Branch::After)),
            ],
        )
    }

    #[test]
    fn a_loop_iterates_at_most_three_sample_items_then_continues_after_it() {
        let steps = walk_real(
            &loop_graph(),
            &json!({ "items": ["a", "b", "c", "d", "e"] }),
        );

        assert_eq!(MAX_DRY_RUN_LOOP_ITEMS, 3);
        assert_eq!(ids(&steps), ["c1", "c2", "c2", "c2", "c3"]);
        let bodies: Vec<&Value> = steps
            .iter()
            .filter(|step| step.connector_id == "c2")
            .map(|step| step.input.as_ref().unwrap())
            .collect();
        assert_eq!(
            bodies,
            [
                &json!({ "name": "a" }),
                &json!({ "name": "b" }),
                &json!({ "name": "c" })
            ]
        );
        assert_eq!(
            first(&steps, "c1").output,
            Some(json!({ "items": ["a", "b", "c", "d", "e"] }))
        );
    }

    #[test]
    fn a_loop_over_fewer_than_three_items_iterates_each_of_them() {
        let two = walk_real(&loop_graph(), &json!({ "items": ["a", "b"] }));
        let none = walk_real(&loop_graph(), &json!({ "items": [] }));

        assert_eq!(ids(&two), ["c1", "c2", "c2", "c3"]);
        assert_eq!(ids(&none), ["c1", "c2", "c3"]);
        assert_eq!(first(&none, "c2").status, DryRunStatus::Skipped);
        assert_eq!(first(&none, "c2").iteration_path, "");
        assert_eq!(first(&none, "c3").status, DryRunStatus::Succeeded);
    }

    #[test]
    fn nested_loops_are_each_capped_at_three() {
        let graph = graph(
            vec![
                loop_node("outer", "{{ trigger.items }}"),
                loop_node("inner", "{{ trigger.items }}"),
                customer_node("leaf", "x"),
            ],
            vec![
                edge("t1", "outer", None),
                edge("outer", "inner", Some(Branch::Each)),
                edge("inner", "leaf", Some(Branch::Each)),
            ],
        );

        let steps = walk_real(&graph, &json!({ "items": [1, 2, 3, 4, 5, 6] }));

        let leaves = steps
            .iter()
            .filter(|step| step.connector_id == "leaf")
            .count();
        assert_eq!(leaves, 9);
    }

    #[test]
    fn a_disabled_node_passes_the_previous_output_through_without_being_reached() {
        let graph = graph(
            vec![
                config_node("c1", json!({ "n": 1 })),
                disabled(customer_node("c2", "never created")),
                http_node("c3", "https://example.com/{{ connectors.c2.output.n }}"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        );
        let spy = RecordingDispatch::new();

        let steps = walk_with(&spy, &graph, &[], None, &json!({})).expect("the walk completes");

        assert_eq!(*spy.reached.borrow(), ["flow.config", "http.request"]);
        let passed = first(&steps, "c2");
        assert_eq!(passed.status, DryRunStatus::Succeeded);
        assert_eq!(passed.output, first(&steps, "c1").output);
        assert!(passed.input.is_none());
        assert_eq!(
            first(&steps, "c3").input,
            Some(json!({ "method": "GET", "url": "https://example.com/1" }))
        );
    }

    #[test]
    fn a_disabled_condition_and_a_disabled_loop_show_the_passed_through_output_as_their_own() {
        let graph = graph(
            vec![
                config_node("c1", json!({ "n": 1 })),
                disabled(condition_node("c2", "{{ not even evaluated")),
                disabled(loop_node("c3", "{{ also not evaluated")),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", Some(Branch::Then)),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(first(&steps, "c2").output, Some(json!({ "n": 1 })));
        assert_eq!(first(&steps, "c3").output, Some(json!({ "n": 1 })));
        assert!(first(&steps, "c2").input.is_none() && first(&steps, "c3").input.is_none());
    }

    #[test]
    fn a_disabled_first_node_passes_the_trigger_payload_through() {
        let graph = graph(
            vec![disabled(customer_node("c1", "x"))],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk_real(&graph, &json!({ "quote_id": "q-1" }));

        assert_eq!(
            first(&steps, "c1").output,
            Some(json!({ "quote_id": "q-1" }))
        );
    }

    #[test]
    fn a_disabled_condition_takes_then_and_a_disabled_loop_iterates_nothing() {
        let graph = graph(
            vec![
                disabled(condition_node("c1", "{{ not even evaluated")),
                customer_node("then", "t"),
                customer_node("else", "e"),
                disabled(loop_node("l1", "{{ also not evaluated")),
                customer_node("body", "b"),
                customer_node("after", "a"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "then", Some(Branch::Then)),
                edge("c1", "else", Some(Branch::Else)),
                edge("then", "l1", None),
                edge("l1", "body", Some(Branch::Each)),
                edge("l1", "after", Some(Branch::After)),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(first(&steps, "then").status, DryRunStatus::Succeeded);
        assert_eq!(first(&steps, "else").status, DryRunStatus::Skipped);
        assert_eq!(first(&steps, "after").status, DryRunStatus::Succeeded);
        assert_eq!(first(&steps, "body").status, DryRunStatus::Skipped);
    }

    #[test]
    fn a_reference_to_an_output_that_does_not_exist_fails_the_node_that_references_it() {
        let graph = graph(
            vec![
                http_node("c1", "https://example.com/a"),
                http_node(
                    "c2",
                    "https://example.com/{{ connectors.c1.output.body.nope }}",
                ),
                customer_node("c3", "never reached"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "c3"]);
        let broken = first(&steps, "c2");
        assert_eq!(broken.status, DryRunStatus::Failed);
        assert!(broken.input.is_none() && broken.output.is_none());
        let message = broken.error.as_deref().unwrap();
        assert!(
            message.contains("connectors.c1.output.body.nope"),
            "{broken:?}"
        );
        assert!(message.starts_with("field `url`"), "{message}");
        assert_eq!(first(&steps, "c1").status, DryRunStatus::Succeeded);
        assert_eq!(first(&steps, "c3").status, DryRunStatus::Skipped);
    }

    #[test]
    fn a_reference_to_a_node_that_is_not_in_the_graph_fails_the_referencing_node() {
        let graph = graph(
            vec![customer_node("c1", "{{ connectors.ghost.output.name }}")],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk_real(&graph, &json!({}));

        let broken = first(&steps, "c1");
        assert_eq!(broken.status, DryRunStatus::Failed);
        assert!(broken.error.as_deref().unwrap().contains("ghost"));
    }

    #[test]
    fn a_reference_to_a_node_on_the_branch_not_taken_fails_the_referencing_node() {
        let graph = graph(
            vec![
                condition_node("c1", "{{ false }}"),
                config_node("then", json!({ "x": 1 })),
                customer_node("else", "{{ connectors.then.output.x }}"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "then", Some(Branch::Then)),
                edge("c1", "else", Some(Branch::Else)),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(first(&steps, "then").status, DryRunStatus::Skipped);
        let broken = first(&steps, "else");
        assert_eq!(broken.status, DryRunStatus::Failed);
        assert!(
            broken
                .error
                .as_deref()
                .unwrap()
                .contains("connectors.then.output.x")
        );
    }

    #[test]
    fn a_connector_that_refuses_its_resolved_config_fails_with_its_own_message() {
        let graph = graph(
            vec![condition_node("c1", "not a boolean")],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk_real(&graph, &json!({}));

        let failed = first(&steps, "c1");
        assert_eq!(failed.status, DryRunStatus::Failed);
        assert_eq!(failed.input, Some(json!({ "predicate": "not a boolean" })));
        assert!(failed.error.as_deref().unwrap().contains("boolean"));
    }

    #[test]
    fn a_failed_node_stops_the_walk_and_every_node_still_to_come_is_skipped() {
        let graph = graph(
            vec![
                condition_node("c1", "not a boolean"),
                customer_node("after_c1", "x"),
                customer_node("c2", "fine"),
                customer_node("after_c2", "y"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("t1", "c2", None),
                edge("c1", "after_c1", Some(Branch::Then)),
                edge("c2", "after_c2", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "after_c1", "after_c2"]);
        assert_eq!(first(&steps, "c1").status, DryRunStatus::Failed);
        for id in ["c2", "after_c1", "after_c2"] {
            let step = first(&steps, id);
            assert_eq!(step.status, DryRunStatus::Skipped, "{id}");
            assert!(
                step.input.is_none() && step.output.is_none() && step.error.is_none(),
                "{id}"
            );
        }
    }

    #[test]
    fn fan_out_targets_are_visited_first_edge_first_and_a_join_waits_for_both_sides() {
        let graph = graph(
            vec![
                config_node("c1", json!({ "n": 1 })),
                config_node("c2", json!({ "from": "c2" })),
                config_node("c3", json!({ "from": "c3" })),
                customer_node("c4", "{{ connectors.c2.output.from }}"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c1", "c3", None),
                edge("c2", "c4", None),
                edge("c3", "c4", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "c3", "c4"]);
        assert!(
            steps
                .iter()
                .all(|step| step.status == DryRunStatus::Succeeded)
        );
        assert_eq!(first(&steps, "c4").input, Some(json!({ "name": "c2" })));
    }

    #[test]
    fn a_node_waits_for_a_predecessor_discovered_later_on_a_longer_path() {
        let graph = graph(
            vec![
                config_node("c1", json!({})),
                config_node("c2", json!({})),
                config_node("c3", json!({})),
                config_node("c5", json!({ "late": "yes" })),
                customer_node("c4", "{{ connectors.c5.output.late }}"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c1", "c3", None),
                edge("c2", "c4", None),
                edge("c3", "c5", None),
                edge("c5", "c4", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "c3", "c5", "c4"]);
        assert_eq!(first(&steps, "c4").status, DryRunStatus::Succeeded);
        assert_eq!(first(&steps, "c4").input, Some(json!({ "name": "yes" })));
    }

    #[test]
    fn entries_are_visited_in_trigger_then_edge_order_never_by_id() {
        let mut graph = graph(
            vec![
                customer_node("z1", "z"),
                customer_node("a1", "a"),
                customer_node("m1", "m"),
            ],
            vec![
                edge("t1", "z1", None),
                edge("t1", "a1", None),
                edge("t2", "m1", None),
            ],
        );
        graph.triggers.push(trigger("t2"));

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["z1", "a1", "m1"]);
    }

    #[test]
    fn every_entry_reads_the_example_of_its_own_trigger() {
        let mut graph = graph(
            vec![
                customer_node("c1", "{{ to_string(trigger.x) }}"),
                customer_node("c2", "{{ to_string(trigger.x) }}"),
                customer_node("c3", "{{ to_string(trigger.x) }}"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("t2", "c2", None),
                edge("c1", "c3", None),
            ],
        );
        graph.triggers.push(trigger("t2"));
        let samples = [json!({ "x": 1 }), json!({ "x": 2 })];

        let all = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[],
            None,
            Payloads::Sampled(&samples),
            far_deadline(),
        )
        .unwrap();
        let only_t2 = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[],
            Some("t2"),
            Payloads::Sampled(&samples),
            far_deadline(),
        )
        .unwrap();

        assert_eq!(first(&all, "c1").input, Some(json!({ "name": "1" })));
        assert_eq!(first(&all, "c2").input, Some(json!({ "name": "2" })));
        assert_eq!(first(&all, "c3").input, Some(json!({ "name": "1" })));
        assert_eq!(ids(&only_t2), ["c2"]);
    }

    #[test]
    fn a_node_fed_by_the_trigger_and_by_another_node_waits_for_that_node() {
        let graph = graph(
            vec![
                config_node("c1", json!({ "v": 1 })),
                customer_node("c2", "{{ to_string(connectors.c1.output.v) }}"),
            ],
            vec![
                edge("t1", "c2", None),
                edge("t1", "c1", None),
                edge("c1", "c2", None),
            ],
        );

        let steps = walk_real(&graph, &json!({}));
        let selected = walk_with(&ConnectorDryRun, &graph, &[], Some("t1"), &json!({})).unwrap();

        assert_eq!(ids(&steps), ["c1", "c2"]);
        assert_eq!(ids(&selected), ["c1", "c2"]);
        assert_eq!(first(&steps, "c2").input, Some(json!({ "name": "1" })));
    }

    #[test]
    fn a_selected_trigger_walks_only_what_it_feeds() {
        let mut graph = graph(
            vec![customer_node("c1", "x"), customer_node("c2", "y")],
            vec![edge("t1", "c1", None), edge("t2", "c2", None)],
        );
        graph.triggers.push(trigger("t2"));

        let from_t2 = walk_with(&ConnectorDryRun, &graph, &[], Some("t2"), &json!({})).unwrap();
        let from_any = walk_real(&graph, &json!({}));

        assert_eq!(ids(&from_t2), ["c2"]);
        assert_eq!(ids(&from_any), ["c1", "c2"]);
    }

    #[test]
    fn an_unknown_trigger_is_refused_and_named() {
        let graph = graph(vec![customer_node("c1", "x")], vec![edge("t1", "c1", None)]);

        let error =
            walk_with(&ConnectorDryRun, &graph, &[], Some("ghost"), &json!({})).unwrap_err();

        assert!(
            matches!(&error, DryRunError::UnknownTrigger { trigger_id } if trigger_id == "ghost"),
            "{error:?}"
        );
    }

    #[test]
    fn a_walk_longer_than_the_ceiling_is_refused_instead_of_running_away() {
        let count = MAX_DRY_RUN_STEPS + 1;
        let connectors: Vec<PlacedConnector> = (0..count)
            .map(|index| config_node(&format!("c{index:05}"), json!({})))
            .collect();
        let mut edges = vec![edge("t1", "c00000", None)];
        edges.extend(
            (1..count)
                .map(|index| edge(&format!("c{:05}", index - 1), &format!("c{index:05}"), None)),
        );

        let error = walk_with(
            &ConnectorDryRun,
            &graph(connectors, edges),
            &[],
            None,
            &json!({}),
        )
        .unwrap_err();

        assert!(
            matches!(
                error,
                DryRunError::TooManySteps {
                    limit: MAX_DRY_RUN_STEPS
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn a_walk_whose_steps_would_weigh_more_than_the_budget_is_refused() {
        let bulky = "x".repeat(1_000_000);
        let graph = graph(
            vec![
                loop_node("l1", "{{ trigger.items }}"),
                loop_node("l2", "{{ trigger.items }}"),
                loop_node("l3", "{{ trigger.items }}"),
                customer_node("leaf", &bulky),
            ],
            vec![
                edge("t1", "l1", None),
                edge("l1", "l2", Some(Branch::Each)),
                edge("l2", "l3", Some(Branch::Each)),
                edge("l3", "leaf", Some(Branch::Each)),
            ],
        );

        let error = walk_with(
            &ConnectorDryRun,
            &graph,
            &[],
            None,
            &json!({ "items": [1, 2, 3] }),
        )
        .unwrap_err();

        assert!(
            matches!(
                error,
                DryRunError::TooLarge {
                    limit: MAX_DRY_RUN_BYTES
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn an_edge_into_a_connector_the_graph_does_not_hold_is_refused_as_the_validation_names_it() {
        let graph = graph(
            vec![config_node("c1", json!({}))],
            vec![edge("t1", "c1", None), edge("c1", "ghost", None)],
        );

        let error = walk_with(&ConnectorDryRun, &graph, &[], None, &json!({})).unwrap_err();

        let DryRunError::InvalidGraph(errors) = &error else {
            panic!("expected InvalidGraph, got {error:?}");
        };
        assert_eq!(
            errors,
            &[GraphError::DanglingEdge {
                from: "c1".to_owned(),
                to: "ghost".to_owned()
            }]
        );
    }

    #[test]
    fn every_step_carries_the_path_of_the_iteration_it_ran_in() {
        let steps = walk_real(&loop_graph(), &json!({ "items": ["a", "b", "c", "d"] }));

        let paths: Vec<(&str, &str)> = steps
            .iter()
            .map(|step| (step.connector_id.as_str(), step.iteration_path.as_str()))
            .collect();
        assert_eq!(
            paths,
            [
                ("c1", ""),
                ("c2", "c1[0]"),
                ("c2", "c1[1]"),
                ("c2", "c1[2]"),
                ("c3", "")
            ]
        );
    }

    #[test]
    fn nested_loops_render_their_paths_outermost_first() {
        let graph = graph(
            vec![
                loop_node("outer", "{{ trigger.items }}"),
                loop_node("inner", "{{ trigger.items }}"),
                customer_node("leaf", "x"),
            ],
            vec![
                edge("t1", "outer", None),
                edge("outer", "inner", Some(Branch::Each)),
                edge("inner", "leaf", Some(Branch::Each)),
            ],
        );

        let steps = walk_real(&graph, &json!({ "items": [1, 2] }));

        let leaves: Vec<&str> = steps
            .iter()
            .filter(|step| step.connector_id == "leaf")
            .map(|step| step.iteration_path.as_str())
            .collect();
        assert_eq!(
            leaves,
            [
                "outer[0].inner[0]",
                "outer[0].inner[1]",
                "outer[1].inner[0]",
                "outer[1].inner[1]"
            ]
        );
        assert_eq!(first(&steps, "inner").iteration_path, "outer[0]");
    }

    #[test]
    fn a_failure_inside_an_iteration_skips_the_rest_of_that_iteration_and_what_follows_the_loop() {
        let graph = graph(
            vec![
                loop_node("l1", "{{ trigger.items }}"),
                condition_node("bad", "not a boolean"),
                customer_node("rest", "x"),
                customer_node("done", "y"),
            ],
            vec![
                edge("t1", "l1", None),
                edge("l1", "bad", Some(Branch::Each)),
                edge("bad", "rest", Some(Branch::Then)),
                edge("l1", "done", Some(Branch::After)),
            ],
        );

        let steps = walk_real(&graph, &json!({ "items": [1, 2, 3] }));

        let seen: Vec<(&str, &str, DryRunStatus)> = steps
            .iter()
            .map(|step| {
                (
                    step.connector_id.as_str(),
                    step.iteration_path.as_str(),
                    step.status,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("l1", "", DryRunStatus::Succeeded),
                ("bad", "l1[0]", DryRunStatus::Failed),
                ("rest", "l1[0]", DryRunStatus::Skipped),
                ("done", "", DryRunStatus::Skipped),
            ]
        );
    }

    #[test]
    fn skipped_rows_count_toward_the_step_ceiling() {
        let mut connectors = vec![
            condition_node("c1", "{{ true }}"),
            customer_node("taken", "x"),
        ];
        let mut edges = vec![
            edge("t1", "c1", None),
            edge("c1", "taken", Some(Branch::Then)),
        ];
        for index in 0..(MAX_DRY_RUN_STEPS + 50) {
            let id = format!("n{index:05}");
            connectors.push(customer_node(&id, "x"));
            edges.push(edge("c1", &id, Some(Branch::Else)));
        }

        let error = walk_with(
            &ConnectorDryRun,
            &graph(connectors, edges),
            &[],
            None,
            &json!({}),
        )
        .unwrap_err();

        assert!(
            matches!(
                error,
                DryRunError::TooManySteps {
                    limit: MAX_DRY_RUN_STEPS
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn three_big_items_and_thousands_of_body_edges_are_refused_without_copying_the_items() {
        let big = "x".repeat(450 * 1024);
        let mut connectors = vec![loop_node("l1", "{{ trigger.items }}")];
        let mut edges = vec![edge("t1", "l1", None)];
        for index in 0..5_000 {
            let id = format!("b{index:05}");
            connectors.push(customer_node(&id, "x"));
            edges.push(edge("l1", &id, Some(Branch::Each)));
        }
        let payload = json!({ "items": [big.clone(), big.clone(), big] });
        let started = Instant::now();

        let error = walk_with(
            &ConnectorDryRun,
            &graph(connectors, edges),
            &[],
            None,
            &payload,
        )
        .unwrap_err();

        assert!(
            matches!(error, DryRunError::TooManySteps { .. }),
            "{error:?}"
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn more_waiting_nodes_than_the_ceiling_are_refused() {
        let mut connectors = vec![loop_node("l1", "{{ trigger.items }}")];
        let mut edges = vec![edge("t1", "l1", None)];
        for index in 0..(MAX_DRY_RUN_PENDING + 1) {
            let id = format!("b{index:05}");
            connectors.push(customer_node(&id, "x"));
            edges.push(edge("l1", &id, Some(Branch::Each)));
        }

        let error = walk_with(
            &ConnectorDryRun,
            &graph(connectors, edges),
            &[],
            None,
            &json!({ "items": [1] }),
        )
        .unwrap_err();

        assert!(
            matches!(
                error,
                DryRunError::TooManySteps {
                    limit: MAX_DRY_RUN_PENDING
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn a_walk_past_its_deadline_is_cut_with_its_own_error() {
        let graph = draft_graph();

        let error = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[],
            None,
            Payloads::Given(&json!({})),
            Instant::now(),
        )
        .unwrap_err();

        assert!(matches!(error, DryRunError::TimedOut), "{error:?}");
    }

    #[test]
    fn a_node_that_resolves_to_more_than_its_budget_fails_its_own_step() {
        let payload = json!({ "p": "x".repeat(1024 * 1024) });
        let graph = graph(
            vec![
                customer_node("c1", &"{{trigger.p}}".repeat(5)),
                customer_node("c2", "after"),
            ],
            vec![edge("t1", "c1", None), edge("c1", "c2", None)],
        );

        let steps = walk_real(&graph, &payload);

        let failed = first(&steps, "c1");
        assert_eq!(failed.status, DryRunStatus::Failed);
        let message = failed.error.as_deref().unwrap();
        assert!(message.starts_with("field `name`"), "{message}");
        assert!(
            message.contains(&MAX_DRY_RUN_NODE_BYTES.to_string()),
            "{message}"
        );
        assert_eq!(first(&steps, "c2").status, DryRunStatus::Skipped);
    }

    #[test]
    fn a_trigger_payload_copied_eighty_thousand_times_is_refused_by_the_node_budget() {
        let payload = json!({ "p": "x".repeat(1024 * 1024) });
        let graph = graph(
            vec![customer_node("c1", &"{{trigger.p}}".repeat(80_000))],
            vec![edge("t1", "c1", None)],
        );
        let started = Instant::now();

        let steps = walk_real(&graph, &payload);

        assert_eq!(first(&steps, "c1").status, DryRunStatus::Failed);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn the_fields_of_a_graph_over_the_structural_limits_are_refused_with_a_static_message() {
        let nodes = graph(
            (0..MAX_DRY_RUN_NODES)
                .map(|index| config_node(&format!("c{index}"), json!({})))
                .collect(),
            vec![edge("t1", "c0", None)],
        );
        let edges = graph(
            vec![config_node("c1", json!({}))],
            (0..=MAX_DRY_RUN_EDGES)
                .map(|_| edge("t1", "c1", None))
                .collect(),
        );
        let field = graph(
            vec![node(
                "c1",
                "flow.config",
                json!({ "variables": { "blob": "x".repeat(MAX_DRY_RUN_FIELD_BYTES) } }),
            )],
            vec![edge("t1", "c1", None)],
        );
        let fine = graph(
            vec![node(
                "c1",
                "flow.config",
                json!({ "variables": { "blob": "x".repeat(MAX_DRY_RUN_FIELD_BYTES - 100) } }),
            )],
            vec![edge("t1", "c1", None)],
        );
        let payload = json!({ "blob": "x".repeat(MAX_DRY_RUN_PAYLOAD_BYTES) });

        let refusals = [
            (ensure_within_limits(&nodes, None), DryRunLimit::Nodes),
            (ensure_within_limits(&edges, None), DryRunLimit::Edges),
            (ensure_within_limits(&field, None), DryRunLimit::Field),
            (
                ensure_within_limits(&fine, Some(&payload)),
                DryRunLimit::Payload,
            ),
        ];

        for (outcome, limit) in refusals {
            let error = outcome.unwrap_err();
            assert!(
                matches!(&error, DryRunError::Oversized(found) if *found == limit),
                "{error:?}"
            );
            assert_eq!(error.to_string(), limit.message());
        }
        assert!(ensure_within_limits(&fine, None).is_ok());
    }

    fn address_draft(response: Option<Value>, status: Option<&str>) -> CollectionDraft {
        let operation = Operation {
            id: "getAddress".to_owned(),
            method: HttpMethod::Get,
            path: "/addresses/{id}".to_owned(),
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
            response,
            response_status: status.map(str::to_owned),
        };
        CollectionDraft::new(CollectionDraftInput {
            title: "Addresses".to_owned(),
            version: "1.0.0".to_owned(),
            description: String::new(),
            base_url: "https://api.example.com".to_owned(),
            source: CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned(),
            },
            security: Vec::new(),
            operations: vec![operation],
            schemas: Map::new(),
        })
        .unwrap()
    }

    fn addresses(response: Option<Value>, status: Option<&str>) -> Collection {
        Collection::from_draft(
            address_draft(response, status),
            Uuid::from_u128(77),
            OrganizationId(Uuid::from_u128(1)),
            "addresses".to_owned(),
            Utc::now(),
        )
    }

    fn operation_node(id: &str, collection: u128, operation: &str, path: Value) -> PlacedConnector {
        let mut connector = node(id, "collection.operation", json!({ "path": path }));
        connector.operation = Some(OperationRef {
            collection_id: Uuid::from_u128(collection),
            operation_id: operation.to_owned(),
        });
        connector
    }

    #[test]
    fn a_collection_operation_answers_with_the_response_example_stored_on_it() {
        let collection = addresses(Some(json!({ "city": "Lyon" })), Some("201"));
        let graph = graph(
            vec![operation_node(
                "c1",
                77,
                "getAddress",
                json!({ "id": "{{ trigger.id }}" }),
            )],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk_with(
            &ConnectorDryRun,
            &graph,
            &[collection],
            None,
            &json!({ "id": "a-1" }),
        )
        .unwrap();

        assert_eq!(
            first(&steps, "c1").output,
            Some(json!({ "status": 201, "headers": {}, "body": { "city": "Lyon" } }))
        );
        assert_eq!(
            first(&steps, "c1").input,
            Some(json!({ "path": { "id": "a-1" } }))
        );
    }

    #[test]
    fn a_collection_operation_whose_collection_or_operation_is_gone_fails_its_step() {
        for (collection, operation) in [(77, "deleteAddress"), (99, "getAddress")] {
            let graph = graph(
                vec![operation_node("c1", collection, operation, json!({}))],
                vec![edge("t1", "c1", None)],
            );

            let steps = walk_with(
                &ConnectorDryRun,
                &graph,
                &[addresses(None, None)],
                None,
                &json!({}),
            )
            .unwrap();

            assert_eq!(
                first(&steps, "c1").status,
                DryRunStatus::Failed,
                "{operation}"
            );
        }
    }

    #[test]
    fn a_collection_operation_whose_config_no_longer_fits_fails_naming_the_field() {
        let collection = addresses(None, None);
        let graph = graph(
            vec![operation_node("c1", 77, "getAddress", json!({}))],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk_with(&ConnectorDryRun, &graph, &[collection], None, &json!({})).unwrap();

        let failed = first(&steps, "c1");
        assert_eq!(failed.status, DryRunStatus::Failed);
        assert!(failed.error.as_deref().unwrap().contains("path.id"));
    }

    async fn seed_organization(pool: &sqlx::PgPool) -> OrganizationId {
        let owner_id = common::generate_uuid_v7();
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

        let org_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Dry run Org",
            format!("dry-run-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    async fn footprint(pool: &sqlx::PgPool, org_id: OrganizationId) -> (i64, i64, i64, i64) {
        let count = async |statement: &str| -> i64 {
            sqlx::query_scalar(statement)
                .bind(org_id.0)
                .fetch_one(pool)
                .await
                .unwrap()
        };
        (
            count("SELECT COUNT(*) FROM automation.run WHERE org_id = $1").await,
            count(
                "SELECT COUNT(*) FROM automation.run_step s \
                 JOIN automation.run r ON r.id = s.run_id WHERE r.org_id = $1",
            )
            .await,
            count("SELECT COUNT(*) FROM customers WHERE org_id = $1").await,
            count("SELECT COUNT(*) FROM automation.event WHERE org_id = $1").await,
        )
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_dry_run_through_the_real_connectors_calls_nothing_and_writes_no_row() {
        use crate::infrastructure::automation::connectors::test_support::{
            StubResponse, TestServer,
        };

        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let graph = graph(
            vec![
                customer_node("c1", "Ada Lovelace"),
                http_node("c2", &format!("{}/webhook", server.url)),
                node("c3", "mestier.task_recurrence.extend_horizon", json!({})),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        );
        let before = footprint(&pool, org_id).await;

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "c2", "c3"]);
        assert!(
            steps
                .iter()
                .all(|step| step.status == DryRunStatus::Succeeded)
        );
        assert_eq!(
            footprint(&pool, org_id).await,
            before,
            "runs, steps, customers and events must be untouched"
        );
        assert!(
            server.requests().is_empty(),
            "no request may leave a dry run"
        );
    }

    fn use_case(pool: sqlx::PgPool) -> MestierUseCase {
        MestierUseCase::new(
            pool,
            crate::application::default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        )
    }

    fn draft_graph() -> Graph {
        graph(
            vec![
                config_node("c1", json!({ "n": 1 })),
                http_node("c2", "https://example.com/a"),
                customer_node("c3", "Ada"),
            ],
            vec![
                edge("t1", "c1", None),
                edge("c1", "c2", None),
                edge("c2", "c3", None),
            ],
        )
    }

    fn offline_use_case() -> MestierUseCase {
        use_case(
            sqlx::PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
                .expect("a lazy pool needs no server"),
        )
    }

    #[tokio::test]
    async fn a_chain_of_twenty_thousand_nodes_is_refused_before_anything_walks_its_ancestors() {
        let count = 20_000;
        let connectors: Vec<PlacedConnector> = (0..count)
            .map(|index| config_node(&format!("c{index}"), json!({})))
            .collect();
        let mut edges = vec![edge("t1", "c0", None)];
        edges.extend(
            (1..count).map(|index| edge(&format!("c{}", index - 1), &format!("c{index}"), None)),
        );
        let started = Instant::now();

        let error = offline_use_case()
            .dry_run_graph(
                OrganizationId(Uuid::from_u128(1)),
                &graph(connectors, edges),
                None,
                None,
            )
            .await
            .unwrap_err();

        assert!(
            matches!(error, DryRunError::Oversized(DryRunLimit::Nodes)),
            "{error:?}"
        );
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[tokio::test]
    async fn an_oversized_trigger_payload_is_refused_before_the_database_is_read() {
        let payload = json!({ "blob": "x".repeat(MAX_DRY_RUN_PAYLOAD_BYTES) });

        let error = offline_use_case()
            .dry_run_graph(
                OrganizationId(Uuid::from_u128(1)),
                &draft_graph(),
                None,
                Some(&payload),
            )
            .await
            .unwrap_err();

        assert!(
            matches!(error, DryRunError::Oversized(DryRunLimit::Payload)),
            "{error:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_unsaved_draft_returns_one_step_per_node_and_leaves_no_row_behind() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool.clone());
        let before = footprint(&pool, org_id).await;

        let steps = usecase
            .dry_run_graph(org_id, &draft_graph(), None, None)
            .await
            .expect("a valid draft dry-runs");

        assert_eq!(ids(&steps), ["c1", "c2", "c3"]);
        assert!(
            steps
                .iter()
                .all(|step| step.status == DryRunStatus::Succeeded)
        );
        assert_eq!(footprint(&pool, org_id).await, before);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_invalid_graph_is_refused_with_the_very_errors_a_save_gives() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool.clone());
        let invalid = graph(
            vec![node("c1", "flow.condition", json!({}))],
            vec![edge("t1", "c1", None)],
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id,
                name: "Draft".to_owned(),
                description: None,
            })
            .await
            .unwrap();
        let saved = usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph: invalid.clone(),
                    layout: None,
                    created_by: None,
                },
            )
            .await
            .unwrap_err();

        let error = usecase
            .dry_run_graph(org_id, &invalid, None, None)
            .await
            .unwrap_err();

        let DryRunError::InvalidGraph(errors) = &error else {
            panic!("expected InvalidGraph, got {error:?}");
        };
        assert!(errors.contains(
            &crate::domain::automation::workflow::GraphError::MissingRequiredField {
                connector_id: "c1".to_owned(),
                field: "predicate".to_owned(),
            }
        ));
        let CoreError::Conflict(message) = saved else {
            panic!("expected Conflict, got {saved:?}");
        };
        assert_eq!(error.to_string(), message);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_collection_operation_answers_with_the_stored_example_and_a_deleted_collection_is_refused()
     {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool.clone());
        let stored = usecase
            .import_collection(
                crate::domain::automation::collection::ImportCollectionCommand {
                    org_id,
                    draft: address_draft(Some(json!({ "city": "Lyon" })), Some("201")),
                },
            )
            .await
            .unwrap();
        let mut call = operation_node("c1", 0, "getAddress", json!({ "id": "1" }));
        call.operation = Some(OperationRef {
            collection_id: stored.id,
            operation_id: "getAddress".to_owned(),
        });
        let graph = graph(vec![call], vec![edge("t1", "c1", None)]);

        let steps = usecase
            .dry_run_graph(org_id, &graph, None, None)
            .await
            .expect("the stored collection resolves");
        usecase.delete_collection(org_id, stored.id).await.unwrap();
        let refused = usecase
            .dry_run_graph(org_id, &graph, None, None)
            .await
            .unwrap_err();

        assert_eq!(
            first(&steps, "c1").output,
            Some(json!({ "status": 201, "headers": {}, "body": { "city": "Lyon" } }))
        );
        let DryRunError::InvalidGraph(errors) = &refused else {
            panic!("expected InvalidGraph, got {refused:?}");
        };
        assert!(errors.contains(
            &crate::domain::automation::workflow::GraphError::UnknownCollection {
                connector_id: "c1".to_owned(),
                collection_id: stored.id,
            }
        ));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_trigger_without_a_payload_reads_the_example_of_its_event() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool);
        let mut by_event = graph(
            vec![customer_node("c1", "{{ trigger.quote_id }}")],
            vec![edge("t1", "c1", None)],
        );
        by_event.triggers[0].kind = TriggerKind::Events(vec!["quote.deleted".to_owned()]);

        let sampled = usecase
            .dry_run_graph(org_id, &by_event, None, None)
            .await
            .unwrap();
        let given = usecase
            .dry_run_graph(org_id, &by_event, None, Some(&json!({ "quote_id": "q-9" })))
            .await
            .unwrap();

        assert_eq!(
            first(&sampled, "c1").input,
            Some(json!({ "name": "018f3b2a-0000-7000-8000-000000000001" }))
        );
        assert_eq!(first(&given, "c1").input, Some(json!({ "name": "q-9" })));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn every_root_reads_the_example_of_its_own_trigger() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool);
        let mut two = graph(
            vec![
                customer_node("c1", "{{ trigger.quote_id }}"),
                customer_node("c2", "{{ to_string(trigger) }}"),
            ],
            vec![edge("t1", "c1", None), edge("t2", "c2", None)],
        );
        two.triggers[0].kind = TriggerKind::Events(vec!["quote.deleted".to_owned()]);
        two.triggers.push(trigger("t2"));

        let steps = usecase
            .dry_run_graph(org_id, &two, None, None)
            .await
            .unwrap();

        assert_eq!(
            first(&steps, "c1").input,
            Some(json!({ "name": "018f3b2a-0000-7000-8000-000000000001" }))
        );
        assert_eq!(first(&steps, "c2").input, Some(json!({ "name": "{}" })));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_collection_of_another_organization_is_refused_exactly_like_one_that_does_not_exist()
    {
        let pool = crate::application::test_support::automation_pool().await;
        let owner = seed_organization(&pool).await;
        let outsider = seed_organization(&pool).await;
        let usecase = use_case(pool);
        let foreign = usecase
            .import_collection(
                crate::domain::automation::collection::ImportCollectionCommand {
                    org_id: owner,
                    draft: address_draft(None, None),
                },
            )
            .await
            .unwrap();
        let referencing = |collection_id: Uuid| {
            let mut call = operation_node("c1", 0, "getAddress", json!({ "id": "1" }));
            call.operation = Some(OperationRef {
                collection_id,
                operation_id: "getAddress".to_owned(),
            });
            graph(vec![call], vec![edge("t1", "c1", None)])
        };

        let foreign_refusal = usecase
            .dry_run_graph(outsider, &referencing(foreign.id), None, None)
            .await
            .unwrap_err();
        let missing = Uuid::from_u128(4242);
        let missing_refusal = usecase
            .dry_run_graph(outsider, &referencing(missing), None, None)
            .await
            .unwrap_err();

        let normalized =
            |error: &DryRunError, id: Uuid| error.to_string().replace(&id.to_string(), "<id>");
        assert!(matches!(foreign_refusal, DryRunError::InvalidGraph(_)));
        assert_eq!(
            normalized(&foreign_refusal, foreign.id),
            normalized(&missing_refusal, missing)
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_trigger_id_the_graph_does_not_hold_is_refused() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = use_case(pool);

        let error = usecase
            .dry_run_graph(org_id, &draft_graph(), Some("ghost"), None)
            .await
            .unwrap_err();

        assert!(
            matches!(&error, DryRunError::UnknownTrigger { trigger_id } if trigger_id == "ghost"),
            "{error:?}"
        );
    }
}

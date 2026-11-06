use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::Utc;
use common::{CoreError, OrganizationId};
use events::EventCatalogue;
use mestier_macros::transactional;
use serde_json::{Value, json};
use uuid::Uuid;

use super::run::{Frame, LoopStep};
use crate::{
    application::MestierUseCase,
    domain::automation::{
        catalogue::event_catalogue,
        collection::{COLLECTION_OPERATION_KIND, Collection, NESTED_EXPRESSION_FIELDS},
        connector::{ConnectorCatalogue, connector_catalogue},
        credential::Credential,
        expression::{ExpressionContext, LoopFrame},
        ports::{CollectionRepository, CredentialRepository},
        run::{
            ConnectorOutcome, DryRunDispatch, DryRunError, DryRunInput, DryRunStep,
            MAX_DRY_RUN_BYTES, MAX_DRY_RUN_LOOP_ITEMS, MAX_DRY_RUN_STEPS, Next, StoredOperation,
            connectors_by_id, disabled_outcome, next_after, pass_through_output, resolve_config,
            resolve_config_nested, roots,
        },
        workflow::{Graph, PlacedConnector, TriggerKind, validate_graph},
    },
    infrastructure::automation::connectors::ConnectorDryRun,
};

type Visit = (String, String);

enum Settled {
    Succeeded(usize),
    Skipped,
    Failed,
}

fn entry_points<'g>(
    graph: &'g Graph,
    trigger_id: Option<&str>,
) -> Result<Vec<&'g str>, DryRunError> {
    let roots = roots(graph);
    let Some(trigger_id) = trigger_id else {
        return Ok(roots);
    };
    if !graph
        .triggers
        .iter()
        .any(|trigger| trigger.id == trigger_id)
    {
        return Err(DryRunError::UnknownTrigger {
            trigger_id: trigger_id.to_owned(),
        });
    }
    let fed: HashSet<&str> = graph
        .edges
        .iter()
        .filter(|edge| edge.from == trigger_id)
        .map(|edge| edge.to.as_str())
        .collect();
    Ok(roots.into_iter().filter(|id| fed.contains(id)).collect())
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

fn succeeded_output_in_scope(
    settled: &HashMap<Visit, Settled>,
    steps: &[DryRunStep],
    connector_id: &str,
    scope_paths: &[String],
) -> Option<Value> {
    let found = scope_paths
        .iter()
        .find_map(|path| settled.get(&(connector_id.to_owned(), path.clone())))?;
    match found {
        Settled::Succeeded(position) => {
            Some(steps[*position].output.clone().unwrap_or(Value::Null))
        }
        Settled::Skipped | Settled::Failed => None,
    }
}

fn advance(
    graph: &Graph,
    frame: &Frame,
    outcome: &ConnectorOutcome,
    stack: &mut Vec<Frame>,
    settled: &mut HashMap<Visit, Settled>,
    steps: &mut Vec<DryRunStep>,
) {
    match next_after(graph, &frame.connector_id, outcome) {
        Next::Follow { targets, skipped } => {
            let iteration_path = frame.iteration_path();
            for id in skipped {
                let visit = (id.to_owned(), iteration_path.clone());
                if settled.contains_key(&visit) {
                    continue;
                }
                settled.insert(visit, Settled::Skipped);
                steps.push(DryRunStep::skipped(id));
            }
            for target in targets {
                stack.push(Frame {
                    connector_id: target.to_owned(),
                    path: frame.path.clone(),
                });
            }
        }
        Next::Iterate { after, each, items } => {
            for after in after {
                stack.push(Frame {
                    connector_id: after.to_owned(),
                    path: frame.path.clone(),
                });
            }
            for (index, item) in items.iter().take(MAX_DRY_RUN_LOOP_ITEMS).enumerate().rev() {
                for target in &each {
                    let mut path = frame.path.clone();
                    path.push(LoopStep {
                        loop_id: frame.connector_id.clone(),
                        index,
                        item: item.clone(),
                    });
                    stack.push(Frame {
                        connector_id: (*target).to_owned(),
                        path,
                    });
                }
            }
        }
    }
}

fn sample_trigger_payload(
    graph: &Graph,
    trigger_id: Option<&str>,
    events: &EventCatalogue,
) -> Value {
    let trigger = match trigger_id {
        Some(id) => graph.triggers.iter().find(|trigger| trigger.id == id),
        None => graph.triggers.first(),
    };
    match trigger.map(|trigger| &trigger.kind) {
        Some(TriggerKind::Events(names)) => names
            .first()
            .and_then(|name| events.descriptors().find(|event| event.name == name))
            .map(|event| event.payload_example.clone())
            .unwrap_or_else(|| json!({})),
        Some(TriggerKind::Manual) | None => json!({}),
    }
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
        let (credentials, collections) = self
            .dry_run_reference_data(org_id, &graph.referenced_collection_ids())
            .await?;
        let catalogue = connector_catalogue();
        let events = event_catalogue();
        validate_graph(graph, &catalogue, &credentials, &events, &collections)
            .map_err(DryRunError::InvalidGraph)?;

        let sampled;
        let payload = match trigger_payload {
            Some(payload) => payload,
            None => {
                sampled = sample_trigger_payload(graph, trigger_id, &events);
                &sampled
            }
        };
        walk(
            &ConnectorDryRun,
            graph,
            &catalogue,
            &collections,
            trigger_id,
            payload,
        )
    }
}

fn within_budget(
    steps: &[DryRunStep],
    weighed: &mut usize,
    spent: &mut usize,
) -> Result<(), DryRunError> {
    *spent += steps[*weighed..]
        .iter()
        .map(DryRunStep::weight)
        .sum::<usize>();
    *weighed = steps.len();
    if *spent > MAX_DRY_RUN_BYTES {
        return Err(DryRunError::TooLarge {
            limit: MAX_DRY_RUN_BYTES,
        });
    }
    Ok(())
}

pub(crate) fn walk<D: DryRunDispatch>(
    dispatch: &D,
    graph: &Graph,
    catalogue: &ConnectorCatalogue,
    collections: &[Collection],
    trigger_id: Option<&str>,
    trigger_payload: &Value,
) -> Result<Vec<DryRunStep>, DryRunError> {
    let placed_by_id = connectors_by_id(graph);
    let mut steps: Vec<DryRunStep> = Vec::new();
    let mut settled: HashMap<Visit, Settled> = HashMap::new();
    let mut outputs: BTreeMap<String, Value> = BTreeMap::new();
    let mut weighed = 0usize;
    let mut spent = 0usize;
    let mut stack: Vec<Frame> = entry_points(graph, trigger_id)?
        .into_iter()
        .rev()
        .map(|id| Frame {
            connector_id: id.to_owned(),
            path: Vec::new(),
        })
        .collect();

    while let Some(frame) = stack.pop() {
        let iteration_path = frame.iteration_path();
        let visit = (frame.connector_id.clone(), iteration_path);
        if settled.contains_key(&visit) {
            continue;
        }
        if steps.len() >= MAX_DRY_RUN_STEPS {
            return Err(DryRunError::TooManySteps {
                limit: MAX_DRY_RUN_STEPS,
            });
        }
        within_budget(&steps, &mut weighed, &mut spent)?;

        let Some(placed) = placed_by_id.get(frame.connector_id.as_str()).copied() else {
            settled.insert(visit, Settled::Failed);
            steps.push(DryRunStep::failed(
                &frame.connector_id,
                None,
                format!("the graph has no connector `{}`", frame.connector_id),
            ));
            continue;
        };

        if placed.disabled {
            let scope_paths = frame.scope_paths();
            let output = pass_through_output(
                graph,
                &frame.connector_id,
                |source| succeeded_output_in_scope(&settled, &steps, source, &scope_paths),
                Some(trigger_payload),
            );
            let outcome = disabled_outcome(&placed.kind, output.clone());
            outputs.insert(
                frame.connector_id.clone(),
                json!({ "output": output.clone() }),
            );
            steps.push(DryRunStep::succeeded(&frame.connector_id, None, output));
            settled.insert(visit, Settled::Succeeded(steps.len() - 1));
            advance(
                graph,
                &frame,
                &outcome,
                &mut stack,
                &mut settled,
                &mut steps,
            );
            continue;
        }

        let loop_frame = frame.path.last().map(|step| LoopFrame {
            item: &step.item,
            index: step.index,
        });
        let ctx = ExpressionContext {
            trigger: Some(trigger_payload),
            connectors: &outputs,
            loop_frame,
            now: Utc::now(),
        };
        let resolved = if placed.kind == COLLECTION_OPERATION_KIND {
            resolve_config_nested(&placed.config, &ctx, &NESTED_EXPRESSION_FIELDS)
        } else {
            resolve_config(&placed.config, &ctx)
        };
        let config = match resolved {
            Ok(config) => config,
            Err(error) => {
                settled.insert(visit, Settled::Failed);
                steps.push(DryRunStep::failed(
                    &frame.connector_id,
                    None,
                    error.to_string(),
                ));
                continue;
            }
        };

        let Some(descriptor) = catalogue.get(&placed.kind, placed.version) else {
            settled.insert(visit, Settled::Failed);
            steps.push(DryRunStep::failed(
                &frame.connector_id,
                Some(Value::Object(config)),
                format!(
                    "no connector `{}` version {} in the catalogue",
                    placed.kind, placed.version
                ),
            ));
            continue;
        };
        let answered = dispatch.dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: stored_operation(placed, collections),
            },
        );
        let Some(outcome) = answered else {
            settled.insert(visit, Settled::Failed);
            steps.push(DryRunStep::failed(
                &frame.connector_id,
                Some(Value::Object(config)),
                format!(
                    "no connector implementation for `{}` version {}",
                    placed.kind, placed.version
                ),
            ));
            continue;
        };

        match stored_output(&outcome) {
            Err(error) => {
                let error = error.to_owned();
                settled.insert(visit, Settled::Failed);
                steps.push(DryRunStep::failed(
                    &frame.connector_id,
                    Some(Value::Object(config)),
                    error,
                ));
            }
            Ok(output) => {
                outputs.insert(
                    frame.connector_id.clone(),
                    json!({ "output": output.clone() }),
                );
                steps.push(DryRunStep::succeeded(
                    &frame.connector_id,
                    Some(Value::Object(config)),
                    output,
                ));
                settled.insert(visit, Settled::Succeeded(steps.len() - 1));
                advance(
                    graph,
                    &frame,
                    &outcome,
                    &mut stack,
                    &mut settled,
                    &mut steps,
                );
            }
        }
    }

    within_budget(&steps, &mut weighed, &mut spent)?;
    Ok(steps)
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

    fn walk_real(graph: &Graph, payload: &Value) -> Vec<DryRunStep> {
        walk(
            &ConnectorDryRun,
            graph,
            &connector_catalogue(),
            &[],
            None,
            payload,
        )
        .expect("the walk completes")
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

        let steps = walk(&spy, &graph, &connector_catalogue(), &[], None, &json!({}))
            .expect("the walk completes");

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
        assert_eq!(ids(&none), ["c1", "c3"]);
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

        let steps = walk(&spy, &graph, &connector_catalogue(), &[], None, &json!({}))
            .expect("the walk completes");

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
        assert!(steps.iter().all(|step| step.connector_id != "body"));
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

        assert_eq!(ids(&steps), ["c1", "c2"]);
        let broken = first(&steps, "c2");
        assert_eq!(broken.status, DryRunStatus::Failed);
        assert!(broken.input.is_none() && broken.output.is_none());
        assert!(
            broken
                .error
                .as_deref()
                .unwrap()
                .contains("connectors.c1.output.body.nope"),
            "{broken:?}"
        );
        assert_eq!(first(&steps, "c1").status, DryRunStatus::Succeeded);
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
    fn a_failed_node_stops_its_own_branch_but_not_its_siblings() {
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

        assert_eq!(ids(&steps), ["c1", "c2", "after_c2"]);
        assert_eq!(first(&steps, "c1").status, DryRunStatus::Failed);
        assert_eq!(first(&steps, "after_c2").status, DryRunStatus::Succeeded);
    }

    #[test]
    fn fan_out_targets_are_visited_in_the_order_the_engine_visits_them() {
        let graph = graph(
            vec![
                config_node("c1", json!({})),
                customer_node("c2", "x"),
                customer_node("c3", "y"),
                customer_node("c4", "z"),
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

        assert_eq!(ids(&steps), ["c1", "c3", "c4", "c2"]);
    }

    #[test]
    fn a_selected_trigger_walks_only_what_it_feeds() {
        let mut graph = graph(
            vec![customer_node("c1", "x"), customer_node("c2", "y")],
            vec![edge("t1", "c1", None), edge("t2", "c2", None)],
        );
        graph.triggers.push(trigger("t2"));

        let from_t2 = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[],
            Some("t2"),
            &json!({}),
        )
        .unwrap();
        let from_any = walk_real(&graph, &json!({}));

        assert_eq!(ids(&from_t2), ["c2"]);
        assert_eq!(ids(&from_any), ["c1", "c2"]);
    }

    #[test]
    fn an_unknown_trigger_is_refused_and_named() {
        let graph = graph(vec![customer_node("c1", "x")], vec![edge("t1", "c1", None)]);

        let error = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[],
            Some("ghost"),
            &json!({}),
        )
        .unwrap_err();

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

        let error = walk(
            &ConnectorDryRun,
            &graph(connectors, edges),
            &connector_catalogue(),
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

        let error = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
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
    fn an_edge_into_a_connector_the_graph_does_not_hold_is_a_failed_step_naming_it() {
        let graph = graph(
            vec![config_node("c1", json!({}))],
            vec![edge("t1", "c1", None), edge("c1", "ghost", None)],
        );

        let steps = walk_real(&graph, &json!({}));

        assert_eq!(ids(&steps), ["c1", "ghost"]);
        let missing = first(&steps, "ghost");
        assert_eq!(missing.status, DryRunStatus::Failed);
        assert!(missing.error.as_deref().unwrap().contains("ghost"));
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

        let steps = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
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
        let collection = addresses(None, None);
        let graph = graph(
            vec![
                operation_node("c1", 77, "deleteAddress", json!({})),
                operation_node("c2", 99, "getAddress", json!({})),
            ],
            vec![edge("t1", "c1", None), edge("t1", "c2", None)],
        );

        let steps = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[collection],
            None,
            &json!({}),
        )
        .unwrap();

        assert_eq!(first(&steps, "c1").status, DryRunStatus::Failed);
        assert_eq!(first(&steps, "c2").status, DryRunStatus::Failed);
    }

    #[test]
    fn a_collection_operation_whose_config_no_longer_fits_fails_naming_the_field() {
        let collection = addresses(None, None);
        let graph = graph(
            vec![operation_node("c1", 77, "getAddress", json!({}))],
            vec![edge("t1", "c1", None)],
        );

        let steps = walk(
            &ConnectorDryRun,
            &graph,
            &connector_catalogue(),
            &[collection],
            None,
            &json!({}),
        )
        .unwrap();

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

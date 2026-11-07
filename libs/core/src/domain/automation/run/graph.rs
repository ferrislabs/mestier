//! Pure graph-walking helpers the engine (`application::automation::run`)
//! composes into a traversal. Kept separate and side-effect free so the
//! shape of "what comes next" is unit-testable without a database.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::{Map, Value};

use super::items::StepItems;
use crate::domain::automation::expression::{
    EvalBudget, ExpressionContext, ExpressionError, parse_template,
};
use crate::domain::automation::workflow::{Branch, Graph, PlacedConnector};

/// One level of loop nesting a connector currently sits inside:
/// `(loop_connector_id, item_index)`. The outermost loop is first, matching
/// how [`format_iteration_path`] renders it.
pub type LoopStackEntry = (String, usize);

/// `graph.connectors`, indexed by id, for O(1) lookup during a walk.
pub fn connectors_by_id(graph: &Graph) -> HashMap<&str, &PlacedConnector> {
    graph
        .connectors
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect()
}

/// Connectors with no incoming edge: triggered directly once the run starts,
/// exactly the definition `workflow::validation` already uses for the same
/// question at save time. Sorted so a fresh walk and a resumed one visit
/// them in the same order.
pub fn roots(graph: &Graph) -> Vec<&str> {
    let all_ids: HashSet<&str> = graph.connectors.iter().map(|c| c.id.as_str()).collect();
    let mut has_incoming: HashSet<&str> = HashSet::new();
    for edge in &graph.edges {
        if all_ids.contains(edge.to.as_str()) && all_ids.contains(edge.from.as_str()) {
            has_incoming.insert(edge.to.as_str());
        }
    }
    let mut roots: Vec<&str> = all_ids.difference(&has_incoming).copied().collect();
    roots.sort_unstable();
    roots
}

/// Targets of every edge leaving `connector_id` on `branch` — `None` for a
/// plain sequence edge, `Some(Then)`/`Some(Else)` for a condition,
/// `Some(Each)`/`Some(After)` for a loop.
pub fn branch_targets<'a>(
    graph: &'a Graph,
    connector_id: &str,
    branch: Option<Branch>,
) -> Vec<&'a str> {
    graph
        .edges
        .iter()
        .filter(|e| e.from == connector_id && e.branch == branch)
        .map(|e| e.to.as_str())
        .collect()
}

/// Every connector reachable forward from `start`, **including `start`
/// itself**. Branch-blind on purpose (same approximation
/// `workflow::validation`'s own reachability walk makes): once past the
/// connector that decided a branch, what is downstream of it is downstream
/// regardless of what kind of edge got it there.
pub fn descendants_via<'a>(graph: &'a Graph, start: &[&'a str]) -> HashSet<&'a str> {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut frontier: VecDeque<&str> = VecDeque::new();
    for &id in start {
        if visited.insert(id) {
            frontier.push_back(id);
        }
    }
    while let Some(node) = frontier.pop_front() {
        for &child in adjacency.get(node).into_iter().flatten() {
            if visited.insert(child) {
                frontier.push_back(child);
            }
        }
    }
    visited
}

/// Renders a loop-nesting stack as `run_step.iteration_path`: empty outside
/// any loop, `"c2[3]"` inside one, `"c2[3].c5[1]"` nested — outermost loop
/// first.
pub fn format_iteration_path(stack: &[LoopStackEntry]) -> String {
    stack
        .iter()
        .map(|(id, index)| format!("{id}[{index}]"))
        .collect::<Vec<_>>()
        .join(".")
}

#[derive(Debug, PartialEq)]
pub struct FieldError {
    pub field: String,
    pub source: ExpressionError,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "field `{}`: {}", self.field, self.source)
    }
}

impl std::error::Error for FieldError {}

/// Evaluates every field of a connector's config against `ctx`, stopping at
/// the first expression that fails. `ConnectorInput::config` is exactly this
/// result — a connector never sees a template, only what it evaluated to.
pub fn resolve_config(
    config: &Map<String, Value>,
    ctx: &ExpressionContext<'_>,
) -> Result<Map<String, Value>, ExpressionError> {
    resolve_config_within(config, ctx, &mut EvalBudget::default()).map_err(|error| error.source)
}

pub fn resolve_config_within(
    config: &Map<String, Value>,
    ctx: &ExpressionContext<'_>,
    budget: &mut EvalBudget,
) -> Result<Map<String, Value>, FieldError> {
    let mut resolved = Map::with_capacity(config.len());
    for (key, raw) in config {
        let value = parse_template(raw)
            .and_then(|template| template.evaluate_within(ctx, budget))
            .map_err(|source| FieldError {
                field: key.clone(),
                source,
            })?;
        resolved.insert(key.clone(), value);
    }
    Ok(resolved)
}

fn resolve_value(
    raw: &Value,
    ctx: &ExpressionContext<'_>,
    budget: &mut EvalBudget,
) -> Result<Value, ExpressionError> {
    match raw {
        Value::Object(map) => map
            .iter()
            .map(|(key, value)| Ok((key.clone(), resolve_value(value, ctx, budget)?)))
            .collect::<Result<Map<String, Value>, ExpressionError>>()
            .map(Value::Object),
        Value::Array(items) => items
            .iter()
            .map(|value| resolve_value(value, ctx, budget))
            .collect::<Result<Vec<Value>, ExpressionError>>()
            .map(Value::Array),
        other => parse_template(other)?.evaluate_within(ctx, budget),
    }
}

pub fn resolve_config_nested(
    config: &Map<String, Value>,
    ctx: &ExpressionContext<'_>,
    nested_fields: &[&str],
) -> Result<Map<String, Value>, ExpressionError> {
    resolve_config_nested_within(config, ctx, nested_fields, &mut EvalBudget::default())
        .map_err(|error| error.source)
}

pub fn resolve_config_nested_within(
    config: &Map<String, Value>,
    ctx: &ExpressionContext<'_>,
    nested_fields: &[&str],
    budget: &mut EvalBudget,
) -> Result<Map<String, Value>, FieldError> {
    let mut resolved = Map::with_capacity(config.len());
    for (key, raw) in config {
        let value = if nested_fields.contains(&key.as_str()) {
            resolve_value(raw, ctx, budget)
        } else {
            parse_template(raw).and_then(|template| template.evaluate_within(ctx, budget))
        }
        .map_err(|source| FieldError {
            field: key.clone(),
            source,
        })?;
        resolved.insert(key.clone(), value);
    }
    Ok(resolved)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Next<'g, 'o> {
    Follow {
        targets: Vec<&'g str>,
        skipped: Vec<&'g str>,
    },
    Iterate {
        after: Vec<&'g str>,
        each: Vec<&'g str>,
        items: &'o [Value],
    },
}

pub fn next_after<'g, 'o>(
    graph: &'g Graph,
    connector_id: &str,
    output: &'o StepItems,
) -> Next<'g, 'o> {
    let StepItems::Branched(lists) = output else {
        return Next::Follow {
            targets: branch_targets(graph, connector_id, None),
            skipped: Vec::new(),
        };
    };

    if lists
        .iter()
        .any(|(branch, _)| matches!(branch, Branch::Each | Branch::After))
    {
        return Next::Iterate {
            after: branch_targets(graph, connector_id, Some(Branch::After)),
            each: branch_targets(graph, connector_id, Some(Branch::Each)),
            items: output.list(Some(Branch::Each)),
        };
    }

    let mut targets: Vec<&str> = Vec::new();
    let mut untaken_targets: Vec<&str> = Vec::new();
    for (branch, items) in lists {
        let reached = branch_targets(graph, connector_id, Some(*branch));
        if items.is_empty() {
            untaken_targets.extend(reached);
        } else {
            for target in reached {
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
    }
    let taken_reach = descendants_via(graph, &targets);
    let untaken_reach = descendants_via(graph, &untaken_targets);
    let mut skipped: Vec<&str> = untaken_reach.difference(&taken_reach).copied().collect();
    skipped.sort_unstable();
    Next::Follow { targets, skipped }
}

pub const MERGE_KIND: &str = "flow.merge";

pub fn merge_blockers<'a>(graph: &'a Graph, connector_id: &str) -> Vec<&'a str> {
    let connectors: HashSet<&str> = graph
        .connectors
        .iter()
        .map(|connector| connector.id.as_str())
        .collect();
    let mut seen: HashSet<&str> = HashSet::new();

    graph
        .edges
        .iter()
        .filter(|edge| edge.to == connector_id)
        .map(|edge| edge.from.as_str())
        .filter(|from| *from != connector_id && connectors.contains(from))
        .filter(|from| seen.insert(from))
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use std::collections::BTreeMap;

    use super::*;
    use crate::domain::automation::workflow::Edge;

    fn connector(id: &str, kind: &str) -> PlacedConnector {
        PlacedConnector {
            id: id.to_string(),
            kind: kind.to_string(),
            version: 1,
            credential_id: None,
            config: Map::new(),
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

    #[test]
    fn a_connector_with_no_incoming_edge_is_a_root() {
        let graph = Graph {
            connectors: vec![
                connector("c1", "flow.condition"),
                connector("c2", "flow.condition"),
            ],
            edges: vec![edge("c1", "c2", Some(Branch::Then))],
            triggers: Vec::new(),
        };

        assert_eq!(roots(&graph), vec!["c1"]);
    }

    #[test]
    fn roots_are_returned_sorted_for_a_deterministic_walk() {
        let graph = Graph {
            connectors: vec![
                connector("cz", "flow.condition"),
                connector("ca", "flow.condition"),
            ],
            edges: vec![],
            triggers: Vec::new(),
        };

        assert_eq!(roots(&graph), vec!["ca", "cz"]);
    }

    #[test]
    fn branch_targets_filters_by_the_exact_branch() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("c1", "c2", Some(Branch::Then)),
                edge("c1", "c3", Some(Branch::Else)),
                edge("c1", "c4", None),
            ],
            triggers: Vec::new(),
        };

        assert_eq!(branch_targets(&graph, "c1", Some(Branch::Then)), vec!["c2"]);
        assert_eq!(branch_targets(&graph, "c1", Some(Branch::Else)), vec!["c3"]);
        assert_eq!(branch_targets(&graph, "c1", None), vec!["c4"]);
        assert_eq!(
            branch_targets(&graph, "c1", Some(Branch::Each)),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn descendants_via_includes_the_start_and_everything_downstream() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![edge("a", "b", None), edge("b", "c", None)],
            triggers: Vec::new(),
        };

        let reached = descendants_via(&graph, &["a"]);

        assert_eq!(reached, HashSet::from_iter(["a", "b", "c"]));
    }

    #[test]
    fn descendants_via_does_not_loop_forever_on_a_cycle() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![edge("a", "b", None), edge("b", "a", None)],
            triggers: Vec::new(),
        };

        let reached = descendants_via(&graph, &["a"]);

        assert_eq!(reached, HashSet::from_iter(["a", "b"]));
    }

    #[test]
    fn a_leaf_with_no_outgoing_edge_reaches_only_itself() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![],
            triggers: Vec::new(),
        };

        assert_eq!(
            descendants_via(&graph, &["only"]),
            HashSet::from_iter(["only"])
        );
    }

    fn produced_outcome() -> StepItems {
        StepItems::Main(vec![json!({ "n": 1 })])
    }

    fn graph_of(connectors: &[&str], edges: Vec<Edge>) -> Graph {
        Graph {
            connectors: connectors
                .iter()
                .map(|id| connector(id, "flow.config"))
                .collect(),
            edges,
            triggers: Vec::new(),
        }
    }

    #[test]
    fn a_merge_waits_for_each_direct_predecessor_once_in_edge_order() {
        let graph = graph_of(
            &["a", "b", "m"],
            vec![
                edge("b", "m", None),
                edge("a", "m", None),
                edge("b", "m", Some(Branch::Else)),
            ],
        );

        assert_eq!(merge_blockers(&graph, "m"), vec!["b", "a"]);
    }

    #[test]
    fn a_trigger_or_the_merge_itself_never_blocks_a_merge() {
        let mut graph = graph_of(
            &["a", "m"],
            vec![edge("a", "m", None), edge("m", "m", None)],
        );
        graph.edges.push(edge("t1", "m", None));

        assert_eq!(merge_blockers(&graph, "m"), vec!["a"]);
    }

    fn branch(taken: Branch) -> StepItems {
        let (then, otherwise) = match taken {
            Branch::Then => (vec![json!({ "n": 1 })], Vec::new()),
            _ => (Vec::new(), vec![json!({ "n": 1 })]),
        };
        StepItems::Branched(vec![(Branch::Then, then), (Branch::Else, otherwise)])
    }

    #[test]
    fn a_produced_outcome_follows_the_plain_edges_and_skips_nothing() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![edge("a", "b", None), edge("a", "c", None)],
            triggers: Vec::new(),
        };

        assert_eq!(
            next_after(&graph, "a", &produced_outcome()),
            Next::Follow {
                targets: vec!["b", "c"],
                skipped: Vec::new(),
            }
        );
    }

    #[test]
    fn a_branch_follows_its_taken_side_and_skips_what_only_the_other_side_reaches() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("c", "yes", Some(Branch::Then)),
                edge("c", "no", Some(Branch::Else)),
                edge("yes", "merge", None),
                edge("no", "merge", None),
                edge("no", "only_no", None),
            ],
            triggers: Vec::new(),
        };

        assert_eq!(
            next_after(&graph, "c", &branch(Branch::Then)),
            Next::Follow {
                targets: vec!["yes"],
                skipped: vec!["no", "only_no"],
            }
        );
        assert_eq!(
            next_after(&graph, "c", &branch(Branch::Else)),
            Next::Follow {
                targets: vec!["no"],
                skipped: vec!["yes"],
            }
        );
    }

    #[test]
    fn what_is_skipped_comes_back_sorted_whatever_the_edge_order() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("c", "t", Some(Branch::Then)),
                edge("c", "z", Some(Branch::Else)),
                edge("c", "m", Some(Branch::Else)),
                edge("c", "a", Some(Branch::Else)),
            ],
            triggers: Vec::new(),
        };

        let Next::Follow { skipped, .. } = next_after(&graph, "c", &branch(Branch::Then)) else {
            panic!("a branch follows");
        };

        assert_eq!(skipped, vec!["a", "m", "z"]);
    }

    #[test]
    fn an_iterate_outcome_splits_the_body_from_the_continuation_and_lends_its_items() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("l", "body", Some(Branch::Each)),
                edge("l", "done", Some(Branch::After)),
            ],
            triggers: Vec::new(),
        };
        let outcome = StepItems::Branched(vec![
            (Branch::Each, vec![Value::from(1), Value::from(2)]),
            (Branch::After, vec![Value::from(1), Value::from(2)]),
        ]);

        let Next::Iterate { after, each, items } = next_after(&graph, "l", &outcome) else {
            panic!("an iteration iterates");
        };

        assert_eq!(after, vec!["done"]);
        assert_eq!(each, vec!["body"]);
        assert_eq!(items, [Value::from(1), Value::from(2)]);
    }

    #[test]
    fn a_branch_that_carries_items_on_both_sides_follows_both_and_skips_nothing() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("c", "yes", Some(Branch::Then)),
                edge("c", "no", Some(Branch::Else)),
                edge("c", "both", Some(Branch::Then)),
                edge("c", "both", Some(Branch::Else)),
            ],
            triggers: Vec::new(),
        };
        let output = StepItems::Branched(vec![
            (Branch::Then, vec![json!({ "n": 1 })]),
            (Branch::Else, vec![json!({ "n": 2 })]),
        ]);

        assert_eq!(
            next_after(&graph, "c", &output),
            Next::Follow {
                targets: vec!["yes", "both", "no"],
                skipped: Vec::new(),
            }
        );
    }

    #[test]
    fn a_branch_that_carries_nothing_anywhere_skips_both_sides() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("c", "yes", Some(Branch::Then)),
                edge("c", "no", Some(Branch::Else)),
            ],
            triggers: Vec::new(),
        };
        let output =
            StepItems::Branched(vec![(Branch::Then, Vec::new()), (Branch::Else, Vec::new())]);

        assert_eq!(
            next_after(&graph, "c", &output),
            Next::Follow {
                targets: Vec::new(),
                skipped: vec!["no", "yes"],
            }
        );
    }

    #[test]
    fn a_loop_with_no_item_still_continues_with_its_after_branch() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![
                edge("l", "body", Some(Branch::Each)),
                edge("l", "done", Some(Branch::After)),
            ],
            triggers: Vec::new(),
        };
        let output = StepItems::Branched(vec![
            (Branch::Each, Vec::new()),
            (Branch::After, Vec::new()),
        ]);

        let Next::Iterate { after, each, items } = next_after(&graph, "l", &output) else {
            panic!("a loop iterates");
        };

        assert_eq!(after, vec!["done"]);
        assert_eq!(each, vec!["body"]);
        assert!(items.is_empty());
    }

    #[test]
    fn an_empty_stack_formats_as_an_empty_path() {
        assert_eq!(format_iteration_path(&[]), "");
    }

    #[test]
    fn a_single_loop_level_formats_as_id_bracket_index() {
        assert_eq!(format_iteration_path(&[("c2".to_string(), 3)]), "c2[3]");
    }

    #[test]
    fn nested_loop_levels_join_outermost_first() {
        assert_eq!(
            format_iteration_path(&[("c2".to_string(), 3), ("c5".to_string(), 1)]),
            "c2[3].c5[1]"
        );
    }

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    fn nested_ctx<'a>(
        trigger: &'a Value,
        nodes: &'a BTreeMap<String, Vec<Value>>,
    ) -> ExpressionContext<'a> {
        ExpressionContext {
            trigger: Some(trigger),
            nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        }
    }

    #[test]
    fn nested_fields_resolve_every_string_inside_objects_and_arrays() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "path": { "id": "{{ trigger.id }}", "kept": 5 },
            "query": { "tags": ["a", "{{ trigger.tag }}", { "deep": "x-{{ trigger.id }}" }] },
            "headers": "{{ trigger.headers }}",
            "body": { "list": [{ "name": "{{ upper(trigger.tag) }}" }], "flag": true, "none": null }
        }))
        .unwrap();
        let trigger =
            json!({ "id": "42", "tag": "red", "headers": { "X-Raw": "{{ not evaluated }}" } });
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let resolved =
            resolve_config_nested(&config, &ctx, &["path", "query", "headers", "body"]).unwrap();

        assert_eq!(resolved["path"], json!({ "id": "42", "kept": 5 }));
        assert_eq!(
            resolved["query"],
            json!({ "tags": ["a", "red", { "deep": "x-42" }] })
        );
        assert_eq!(
            resolved["headers"],
            json!({ "X-Raw": "{{ not evaluated }}" })
        );
        assert_eq!(
            resolved["body"],
            json!({ "list": [{ "name": "RED" }], "flag": true, "none": null })
        );
    }

    #[test]
    fn a_field_outside_the_nested_list_keeps_its_inner_text_literal() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "path": { "id": "{{ trigger.id }}" },
            "variables": { "name": "{{ trigger.id }}" }
        }))
        .unwrap();
        let trigger = json!({ "id": "42" });
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let resolved = resolve_config_nested(&config, &ctx, &["path"]).unwrap();

        assert_eq!(resolved["path"], json!({ "id": "42" }));
        assert_eq!(resolved["variables"], json!({ "name": "{{ trigger.id }}" }));
    }

    #[test]
    fn the_plain_resolution_every_other_connector_uses_is_unchanged() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "headers": { "X-Id": "{{ trigger.id }}" },
            "body": { "name": "{{ trigger.id }}" }
        }))
        .unwrap();
        let trigger = json!({ "id": "42" });
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let resolved = resolve_config(&config, &ctx).unwrap();

        assert_eq!(resolved["headers"], json!({ "X-Id": "{{ trigger.id }}" }));
        assert_eq!(resolved["body"], json!({ "name": "{{ trigger.id }}" }));
    }

    #[test]
    fn a_failing_field_is_named_with_the_error_it_raised() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "name": "fine",
            "url": "https://example.com/{{ $('Devis').item.json.nope }}"
        }))
        .unwrap();
        let trigger = json!({});
        let mut connectors = BTreeMap::new();
        connectors.insert("Devis".to_string(), vec![json!({})]);
        let ctx = nested_ctx(&trigger, &connectors);

        let error = resolve_config_within(&config, &ctx, &mut EvalBudget::default()).unwrap_err();

        assert_eq!(error.field, "url");
        assert_eq!(
            error.source,
            ExpressionError::MissingPath {
                path: "$('Devis').item.json.nope".to_string()
            }
        );
        assert_eq!(
            error.to_string(),
            "field `url`: missing path `$('Devis').item.json.nope`"
        );
    }

    #[test]
    fn a_failing_nested_value_is_named_by_its_top_level_field() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "body": { "list": [{ "name": "{{ trigger.absent }}" }] }
        }))
        .unwrap();
        let trigger = json!({});
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let error =
            resolve_config_nested_within(&config, &ctx, &["body"], &mut EvalBudget::default())
                .unwrap_err();

        assert_eq!(error.field, "body");
    }

    #[test]
    fn one_budget_covers_every_field_of_a_config() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "a": "{{ trigger.p }}",
            "b": "{{ trigger.p }}"
        }))
        .unwrap();
        let trigger = json!({ "p": "x".repeat(600) });
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let error = resolve_config_within(&config, &ctx, &mut EvalBudget::new(1000)).unwrap_err();

        assert_eq!(error.field, "b");
        assert_eq!(error.source, ExpressionError::TooLarge { limit: 1000 });
    }

    #[test]
    fn the_engines_own_resolution_still_raises_the_bare_expression_error() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "url": "{{ trigger.absent }}"
        }))
        .unwrap();
        let trigger = json!({});
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let error = resolve_config(&config, &ctx).unwrap_err();

        assert_eq!(
            error,
            ExpressionError::MissingPath {
                path: "trigger.absent".to_string()
            }
        );
    }

    #[test]
    fn a_nested_expression_that_fails_surfaces_its_error() {
        let config = serde_json::from_value::<Map<String, Value>>(json!({
            "path": { "id": "{{ trigger.missing }}" }
        }))
        .unwrap();
        let trigger = json!({});
        let connectors = BTreeMap::new();
        let ctx = nested_ctx(&trigger, &connectors);

        let error = resolve_config_nested(&config, &ctx, &["path"]).unwrap_err();

        assert_eq!(
            error,
            ExpressionError::MissingPath {
                path: "trigger.missing".to_string()
            }
        );
    }

    #[test]
    fn resolve_config_evaluates_every_field() {
        let mut config = Map::new();
        config.insert("literal".to_string(), json!("plain text"));
        config.insert("expr".to_string(), json!("{{ upper(\"hi\") }}"));
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            nodes: &nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        };

        let resolved = resolve_config(&config, &ctx).expect("both fields evaluate");

        assert_eq!(resolved.get("literal"), Some(&json!("plain text")));
        assert_eq!(resolved.get("expr"), Some(&json!("HI")));
    }

    #[test]
    fn resolve_config_on_an_empty_map_is_an_empty_map() {
        let config = Map::new();
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            nodes: &nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        };

        assert_eq!(resolve_config(&config, &ctx).unwrap(), Map::new());
    }

    #[test]
    fn resolve_config_surfaces_the_first_evaluation_error() {
        let mut config = Map::new();
        config.insert("bad".to_string(), json!("{{ trigger.missing }}"));
        let nodes = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            nodes: &nodes,
            position: 0,
            loop_frame: None,
            now: fixed_now(),
        };

        let error = resolve_config(&config, &ctx).expect_err("trigger is absent");

        assert_eq!(
            error,
            ExpressionError::MissingPath {
                path: "trigger.missing".to_string()
            }
        );
    }

    /// Kept so `connectors_by_id` stays exercised: it is trivial, but a typo
    /// swapping key/value would otherwise only surface once the engine uses
    /// it against a real graph.
    #[test]
    fn connectors_by_id_indexes_by_the_connectors_own_id() {
        let graph = Graph {
            connectors: vec![connector("c1", "flow.condition")],
            edges: vec![],
            triggers: Vec::new(),
        };

        let index = connectors_by_id(&graph);

        assert_eq!(index.get("c1").map(|c| c.id.as_str()), Some("c1"));
        assert_eq!(index.get("missing"), None);
    }

    #[test]
    fn a_connector_a_trigger_points_at_is_still_a_root() {
        use crate::domain::automation::workflow::{PlacedTrigger, TriggerKind};

        let graph = Graph {
            connectors: vec![
                connector("c1", "http.request"),
                connector("c2", "http.request"),
            ],
            edges: vec![
                Edge {
                    from: "t1".to_string(),
                    to: "c1".to_string(),
                    branch: None,
                },
                Edge {
                    from: "c1".to_string(),
                    to: "c2".to_string(),
                    branch: None,
                },
            ],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Manual,
                name: None,
                description: None,
            }],
        };

        assert_eq!(roots(&graph), vec!["c1"]);
    }
}

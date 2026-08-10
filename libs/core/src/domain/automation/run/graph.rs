//! Pure graph-walking helpers the engine (`application::automation::run`)
//! composes into a traversal. Kept separate and side-effect free so the
//! shape of "what comes next" is unit-testable without a database.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::{Map, Value};

use crate::domain::automation::expression::{ExpressionContext, ExpressionError, parse_template};
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
        if all_ids.contains(edge.to.as_str()) {
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

/// Evaluates every field of a connector's config against `ctx`, stopping at
/// the first expression that fails. `ConnectorInput::config` is exactly this
/// result — a connector never sees a template, only what it evaluated to.
pub fn resolve_config(
    config: &Map<String, Value>,
    ctx: &ExpressionContext<'_>,
) -> Result<Map<String, Value>, ExpressionError> {
    let mut resolved = Map::with_capacity(config.len());
    for (key, raw) in config {
        let template = parse_template(raw)?;
        resolved.insert(key.clone(), template.evaluate(ctx)?);
    }
    Ok(resolved)
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
        };

        let reached = descendants_via(&graph, &["a"]);

        assert_eq!(reached, HashSet::from_iter(["a", "b", "c"]));
    }

    #[test]
    fn descendants_via_does_not_loop_forever_on_a_cycle() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![edge("a", "b", None), edge("b", "a", None)],
        };

        let reached = descendants_via(&graph, &["a"]);

        assert_eq!(reached, HashSet::from_iter(["a", "b"]));
    }

    #[test]
    fn a_leaf_with_no_outgoing_edge_reaches_only_itself() {
        let graph = Graph {
            connectors: vec![],
            edges: vec![],
        };

        assert_eq!(
            descendants_via(&graph, &["only"]),
            HashSet::from_iter(["only"])
        );
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

    #[test]
    fn resolve_config_evaluates_every_field() {
        let mut config = Map::new();
        config.insert("literal".to_string(), json!("plain text"));
        config.insert("expr".to_string(), json!("{{ upper(\"hi\") }}"));
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            connectors: &connectors,
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
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            connectors: &connectors,
            loop_frame: None,
            now: fixed_now(),
        };

        assert_eq!(resolve_config(&config, &ctx).unwrap(), Map::new());
    }

    #[test]
    fn resolve_config_surfaces_the_first_evaluation_error() {
        let mut config = Map::new();
        config.insert("bad".to_string(), json!("{{ trigger.missing }}"));
        let connectors = BTreeMap::new();
        let ctx = ExpressionContext {
            trigger: None,
            connectors: &connectors,
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
        };

        let index = connectors_by_id(&graph);

        assert_eq!(index.get("c1").map(|c| c.id.as_str()), Some("c1"));
        assert_eq!(index.get("missing"), None);
    }
}

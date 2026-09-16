//! Validates a [`Graph`] at save time (#199), so a broken workflow is
//! refused at the editor rather than discovered mid-run in production.
//!
//! [`validate_graph`] collects **every** error rather than stopping at the
//! first: the editor shows them together, and fixing one field only to
//! discover the next is exactly the workflow this exists to avoid.

use std::collections::{HashMap, HashSet, VecDeque};

use events::EventCatalogue;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::automation::connector::{ConnectorCatalogue, FieldKind};
use crate::domain::automation::credential::Credential;
use crate::domain::automation::expression::parse_template;

use super::graph::{Branch, Edge, Graph, PlacedConnector, TriggerKind};

/// Every way a graph can be refused. Each variant names the connector at
/// fault (`connector_id`) and, when the mistake is inside one field, the
/// field too — that is what lets the editor put the error on the field
/// itself instead of a toast nobody can act on.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum GraphError {
    #[error("connector `{id}` is used more than once in this graph")]
    DuplicateConnectorId { id: String },

    #[error("connector `{connector_id}` has unknown kind `{kind}` version {version}")]
    UnknownConnectorKind {
        connector_id: String,
        kind: String,
        version: u16,
    },

    #[error("connector `{connector_id}` sets unknown field `{field}`")]
    UnknownConfigField { connector_id: String, field: String },

    #[error("connector `{connector_id}` is missing required field `{field}`")]
    MissingRequiredField { connector_id: String, field: String },

    #[error("connector `{connector_id}` field `{field}` expected {expected}, got {got}")]
    FieldTypeMismatch {
        connector_id: String,
        field: String,
        expected: &'static str,
        got: &'static str,
    },

    #[error("connector `{connector_id}` field `{field}` does not accept an expression")]
    ExpressionNotAllowed { connector_id: String, field: String },

    #[error("connector `{connector_id}` field `{field}`: {message}")]
    InvalidExpression {
        connector_id: String,
        field: String,
        message: String,
    },

    #[error("connector `{connector_id}` references unknown credential `{credential_id}`")]
    UnknownCredential {
        connector_id: String,
        credential_id: Uuid,
    },

    #[error(
        "connector `{connector_id}` credential `{credential_id}` has scheme `{scheme}`, which it does not accept"
    )]
    CredentialSchemeNotAccepted {
        connector_id: String,
        credential_id: Uuid,
        scheme: String,
    },

    #[error("connector `{connector_id}` requires a credential but none is set")]
    MissingCredential { connector_id: String },

    #[error(
        "connector `{connector_id}` field `{field}` references unknown connector `{referenced_id}`"
    )]
    UnknownConnectorReference {
        connector_id: String,
        field: String,
        referenced_id: String,
    },

    #[error(
        "connector `{connector_id}` field `{field}` references `{referenced_id}`, which runs after it"
    )]
    DownstreamConnectorReference {
        connector_id: String,
        field: String,
        referenced_id: String,
    },

    #[error("connector `{connector_id}` field `{field}` reads `loop` outside a `flow.loop` body")]
    LoopUsedOutsideLoop { connector_id: String, field: String },

    #[error(
        "edge from `{from}` to `{to}` references a connector that does not exist in this graph"
    )]
    DanglingEdge { from: String, to: String },

    #[error("connector `{connector_id}` uses a branch its kind does not define")]
    InvalidBranch { connector_id: String },

    #[error("the graph has a cycle through: {}", .connector_ids.join(", "))]
    Cycle { connector_ids: Vec<String> },

    #[error("connector `{connector_id}` is unreachable from any trigger")]
    UnreachableConnector { connector_id: String },

    #[error("trigger `{id}` is used more than once in this graph")]
    DuplicateTriggerId { id: String },

    #[error("trigger `{id}` has the same id as a connector in this graph")]
    TriggerIdCollidesWithConnectorId { id: String },

    #[error(
        "edge from `{from}` into trigger `{trigger_id}` is refused: a trigger has no incoming edge"
    )]
    EdgeIntoTrigger { trigger_id: String, from: String },

    #[error(
        "edge from trigger `{trigger_id}` to `{to}` carries a branch, which a trigger's edge never does"
    )]
    BranchedTriggerEdge { trigger_id: String, to: String },

    #[error("trigger `{trigger_id}` has no outgoing edge")]
    InertTrigger { trigger_id: String },

    #[error("trigger `{trigger_id}` subscribes to no event")]
    EmptyEventTrigger { trigger_id: String },

    #[error("trigger `{trigger_id}` subscribes to unknown event `{name}`")]
    UnknownEventName { trigger_id: String, name: String },
}

/// Rejects the JSON type an expression-free field's literal value carries
/// against what its [`FieldKind`] expects. `Json` accepts anything, so it
/// never mismatches.
fn type_mismatch(value: &Value, kind: &FieldKind) -> Option<(&'static str, &'static str)> {
    let (expected, matches) = match kind {
        FieldKind::Text | FieldKind::Select { .. } => ("string", value.is_string()),
        FieldKind::Number => ("number", value.is_number()),
        FieldKind::Bool => ("boolean", value.is_boolean()),
        FieldKind::Json => return None,
    };

    if matches {
        None
    } else {
        Some((expected, json_type_name(value)))
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Every connector that can reach `of` by forward edges — the set a
/// reference from `of` is allowed to name, per `referenced_connectors()`'s
/// own doc comment: reject a reference to a connector "located downstream".
/// Walks `predecessors` (the reverse of the edge direction) so it costs one
/// pass regardless of how deep `of` sits in the graph, and tolerates a cycle
/// elsewhere in the graph (already reported by [`GraphError::Cycle`]) by
/// never revisiting a node.
fn ancestors_of<'a>(predecessors: &HashMap<&'a str, Vec<&'a str>>, of: &str) -> HashSet<&'a str> {
    let mut visited: HashSet<&'a str> = HashSet::new();
    let mut frontier: VecDeque<&'a str> = VecDeque::new();
    for &predecessor in predecessors.get(of).into_iter().flatten() {
        if visited.insert(predecessor) {
            frontier.push_back(predecessor);
        }
    }
    while let Some(node) = frontier.pop_front() {
        for &predecessor in predecessors.get(node).into_iter().flatten() {
            if visited.insert(predecessor) {
                frontier.push_back(predecessor);
            }
        }
    }
    visited
}

/// Rends **toutes** les erreurs, pas la première.
pub fn validate_graph(
    graph: &Graph,
    catalogue: &ConnectorCatalogue,
    credentials: &[Credential],
    events: &EventCatalogue,
) -> Result<(), Vec<GraphError>> {
    let mut errors = Vec::new();

    let mut seen_ids: HashSet<&str> = HashSet::new();
    for connector in &graph.connectors {
        if !seen_ids.insert(connector.id.as_str()) {
            errors.push(GraphError::DuplicateConnectorId {
                id: connector.id.clone(),
            });
        }
    }

    // `connectors_by_id` overwrites on a duplicate id (last write wins): the
    // duplicate is already reported above, and everything below is
    // best-effort once that has happened.
    let connectors_by_id: HashMap<&str, &PlacedConnector> = graph
        .connectors
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();
    let connector_ids: HashSet<&str> = connectors_by_id.keys().copied().collect();

    let mut seen_trigger_ids: HashSet<&str> = HashSet::new();
    for trigger in &graph.triggers {
        if !seen_trigger_ids.insert(trigger.id.as_str()) {
            errors.push(GraphError::DuplicateTriggerId {
                id: trigger.id.clone(),
            });
        }
    }

    let trigger_ids: HashSet<&str> = graph.triggers.iter().map(|t| t.id.as_str()).collect();
    for trigger in &graph.triggers {
        if connector_ids.contains(trigger.id.as_str()) {
            errors.push(GraphError::TriggerIdCollidesWithConnectorId {
                id: trigger.id.clone(),
            });
        }
    }

    let triggers_with_an_outgoing_edge: HashSet<&str> =
        graph.edges.iter().map(|e| e.from.as_str()).collect();
    for trigger in &graph.triggers {
        if !triggers_with_an_outgoing_edge.contains(trigger.id.as_str()) {
            errors.push(GraphError::InertTrigger {
                trigger_id: trigger.id.clone(),
            });
        }

        if let TriggerKind::Events(names) = &trigger.kind {
            if names.is_empty() {
                errors.push(GraphError::EmptyEventTrigger {
                    trigger_id: trigger.id.clone(),
                });
            }
            for name in names {
                if !events
                    .descriptors()
                    .any(|descriptor| descriptor.name == name)
                {
                    errors.push(GraphError::UnknownEventName {
                        trigger_id: trigger.id.clone(),
                        name: name.clone(),
                    });
                }
            }
        }
    }

    // Every edge is classified by what its endpoints name. An edge into a
    // trigger, or a branched edge out of one, is refused and excluded from
    // every graph-shape computation below, the same way a dangling edge
    // always was: a branch check, a cycle walk or a reachability walk built
    // from a half-real edge would only add noise on top of the real problem.
    let mut connector_edges: Vec<&Edge> = Vec::new();
    let mut trigger_edges: Vec<&Edge> = Vec::new();
    for edge in &graph.edges {
        if trigger_ids.contains(edge.to.as_str()) {
            errors.push(GraphError::EdgeIntoTrigger {
                trigger_id: edge.to.clone(),
                from: edge.from.clone(),
            });
            continue;
        }

        if trigger_ids.contains(edge.from.as_str()) {
            if !connector_ids.contains(edge.to.as_str()) {
                errors.push(GraphError::DanglingEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                });
                continue;
            }
            if edge.branch.is_some() {
                errors.push(GraphError::BranchedTriggerEdge {
                    trigger_id: edge.from.clone(),
                    to: edge.to.clone(),
                });
                continue;
            }
            trigger_edges.push(edge);
            continue;
        }

        if connector_ids.contains(edge.from.as_str()) && connector_ids.contains(edge.to.as_str()) {
            connector_edges.push(edge);
        } else {
            errors.push(GraphError::DanglingEdge {
                from: edge.from.clone(),
                to: edge.to.clone(),
            });
        }
    }

    for edge in &connector_edges {
        let source = connectors_by_id[edge.from.as_str()];
        let Some(allowed) = catalogue
            .get(source.kind.as_str(), source.version)
            .map(|descriptor| descriptor.branches)
        else {
            continue;
        };
        let ok = match edge.branch {
            Some(branch) => allowed.contains(&branch),
            None => allowed.is_empty(),
        };
        if !ok {
            errors.push(GraphError::InvalidBranch {
                connector_id: source.id.clone(),
            });
        }
    }

    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut predecessors: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> =
        connector_ids.iter().map(|id| (*id, 0usize)).collect();
    for edge in &connector_edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        predecessors
            .entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
        *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Kahn's algorithm: repeatedly remove connectors with no remaining
    // incoming edge. Whatever is left once nothing more can be removed is
    // exactly the set of connectors sitting on (or only reachable through) a
    // cycle.
    let roots: Vec<&str> = in_degree
        .iter()
        .filter(|&(_, &degree)| degree == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut remaining_in_degree = in_degree.clone();
    let mut queue: VecDeque<&str> = roots.iter().copied().collect();
    let mut resolved = 0usize;
    while let Some(node) = queue.pop_front() {
        resolved += 1;
        for &child in adjacency.get(node).into_iter().flatten() {
            if let Some(degree) = remaining_in_degree.get_mut(child) {
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(child);
                }
            }
        }
    }
    if resolved < connector_ids.len() {
        let mut cyclic: Vec<String> = remaining_in_degree
            .iter()
            .filter(|&(_, &degree)| degree > 0)
            .map(|(id, _)| (*id).to_string())
            .collect();
        cyclic.sort();
        errors.push(GraphError::Cycle {
            connector_ids: cyclic,
        });
    }

    let mut reachable: HashSet<&str> = HashSet::new();
    let mut frontier: VecDeque<&str> = VecDeque::new();
    for edge in &trigger_edges {
        if reachable.insert(edge.to.as_str()) {
            frontier.push_back(edge.to.as_str());
        }
    }
    while let Some(node) = frontier.pop_front() {
        for &child in adjacency.get(node).into_iter().flatten() {
            if reachable.insert(child) {
                frontier.push_back(child);
            }
        }
    }
    let mut unreachable: Vec<&str> = connector_ids
        .iter()
        .copied()
        .filter(|id| !reachable.contains(id))
        .collect();
    unreachable.sort();
    for id in unreachable {
        errors.push(GraphError::UnreachableConnector {
            connector_id: id.to_string(),
        });
    }

    // Every connector reachable from a `flow.loop`'s `Each` edge is "inside"
    // that loop's body for the purposes of `loop.*`. A connector also
    // reachable from the loop's `After` edge (because the graph rejoins
    // after the loop) is still counted as inside the body by this
    // approximation — refining that is deferred until the engine (#200)
    // shows it matters.
    let mut loop_body: HashSet<&str> = HashSet::new();
    for connector in &graph.connectors {
        let opens_a_loop = catalogue
            .get(connector.kind.as_str(), connector.version)
            .is_some_and(|descriptor| descriptor.branches.contains(&Branch::Each));
        if !opens_a_loop {
            continue;
        }
        for edge in connector_edges
            .iter()
            .filter(|e| e.from == connector.id && e.branch == Some(Branch::Each))
        {
            let mut frontier: VecDeque<&str> = VecDeque::new();
            if loop_body.insert(edge.to.as_str()) {
                frontier.push_back(edge.to.as_str());
            }
            while let Some(node) = frontier.pop_front() {
                for &child in adjacency.get(node).into_iter().flatten() {
                    if loop_body.insert(child) {
                        frontier.push_back(child);
                    }
                }
            }
        }
    }

    for connector in &graph.connectors {
        let Some(descriptor) = catalogue.get(&connector.kind, connector.version) else {
            errors.push(GraphError::UnknownConnectorKind {
                connector_id: connector.id.clone(),
                kind: connector.kind.clone(),
                version: connector.version,
            });
            continue;
        };

        let known_fields: HashSet<&str> = descriptor.fields.iter().map(|f| f.name).collect();
        for key in connector.config.keys() {
            if !known_fields.contains(key.as_str()) {
                errors.push(GraphError::UnknownConfigField {
                    connector_id: connector.id.clone(),
                    field: key.clone(),
                });
            }
        }

        let ancestors = ancestors_of(&predecessors, &connector.id);
        for field in descriptor.fields {
            let Some(raw) = connector.config.get(field.name) else {
                if field.required {
                    errors.push(GraphError::MissingRequiredField {
                        connector_id: connector.id.clone(),
                        field: field.name.to_string(),
                    });
                }
                continue;
            };

            let contains_braces = raw.as_str().is_some_and(|s| s.contains("{{"));
            if !field.expression && contains_braces {
                errors.push(GraphError::ExpressionNotAllowed {
                    connector_id: connector.id.clone(),
                    field: field.name.to_string(),
                });
                continue;
            }

            let template = match parse_template(raw) {
                Ok(template) => template,
                Err(error) => {
                    errors.push(GraphError::InvalidExpression {
                        connector_id: connector.id.clone(),
                        field: field.name.to_string(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            if template.is_static()
                && let Some((expected, got)) = type_mismatch(raw, &field.kind)
            {
                errors.push(GraphError::FieldTypeMismatch {
                    connector_id: connector.id.clone(),
                    field: field.name.to_string(),
                    expected,
                    got,
                });
            }

            for referenced_id in template.referenced_connectors() {
                if !connector_ids.contains(referenced_id.as_str()) {
                    errors.push(GraphError::UnknownConnectorReference {
                        connector_id: connector.id.clone(),
                        field: field.name.to_string(),
                        referenced_id,
                    });
                } else if !ancestors.contains(referenced_id.as_str()) {
                    errors.push(GraphError::DownstreamConnectorReference {
                        connector_id: connector.id.clone(),
                        field: field.name.to_string(),
                        referenced_id,
                    });
                }
            }

            if template.uses_loop() && !loop_body.contains(connector.id.as_str()) {
                errors.push(GraphError::LoopUsedOutsideLoop {
                    connector_id: connector.id.clone(),
                    field: field.name.to_string(),
                });
            }
        }

        match connector.credential_id {
            None => {
                if !descriptor.auth.is_satisfied_without_a_credential() {
                    errors.push(GraphError::MissingCredential {
                        connector_id: connector.id.clone(),
                    });
                }
            }
            Some(credential_id) => match credentials.iter().find(|c| c.id == credential_id) {
                None => errors.push(GraphError::UnknownCredential {
                    connector_id: connector.id.clone(),
                    credential_id,
                }),
                // Also catches a connector whose `auth` is `None` given a
                // credential anyway: `AuthRequirement::None.accepts(_)` is
                // always `false`, so no separate rule is needed for it.
                Some(credential) => {
                    if !descriptor.auth.accepts(&credential.kind) {
                        errors.push(GraphError::CredentialSchemeNotAccepted {
                            connector_id: connector.id.clone(),
                            credential_id,
                            scheme: credential.kind.clone(),
                        });
                    }
                }
            },
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::graph::PlacedTrigger;
    use super::*;
    use crate::domain::automation::connector::AuthRequirement;
    use crate::domain::automation::connector::connector_catalogue;
    use events::EventDescriptor;

    fn condition(id: &str, predicate: &str) -> super::super::graph::PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(predicate));
        super::super::graph::PlacedConnector {
            id: id.to_string(),
            kind: "flow.condition".to_string(),
            version: 1,
            credential_id: None,
            config,
        }
    }

    fn graph_of(connectors: Vec<super::super::graph::PlacedConnector>) -> Graph {
        Graph {
            connectors,
            edges: Vec::new(),
            triggers: Vec::new(),
        }
    }

    fn with_trigger(mut graph: Graph, target: &str) -> Graph {
        graph.triggers.push(PlacedTrigger {
            id: "t0".to_string(),
            kind: TriggerKind::Manual,
        });
        graph.edges.push(Edge {
            from: "t0".to_string(),
            to: target.to_string(),
            branch: None,
        });
        graph
    }

    #[test]
    fn an_empty_graph_is_valid() {
        let catalogue = connector_catalogue();
        assert_eq!(
            validate_graph(&Graph::default(), &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn a_duplicate_connector_id_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c1", "{{ false }}"),
        ]);

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("duplicate id refused");

        assert!(
            errors.contains(&GraphError::DuplicateConnectorId {
                id: "c1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_unknown_connector_kind_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "not.a.real.kind".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("unknown kind refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownConnectorKind {
                connector_id: "c1".to_string(),
                kind: "not.a.real.kind".to_string(),
                version: 1,
            }]
        );
    }

    #[test]
    fn an_unknown_connector_version_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 99,
                credential_id: None,
                config: serde_json::Map::new(),
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("unknown version refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownConnectorKind {
                connector_id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 99,
            }]
        );
    }

    #[test]
    fn an_unknown_config_field_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        config.insert("typo_field".to_string(), json!("oops"));
        let graph = graph_of(vec![super::super::graph::PlacedConnector {
            id: "c1".to_string(),
            kind: "flow.condition".to_string(),
            version: 1,
            credential_id: None,
            config,
        }]);

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("unknown field refused");

        assert!(
            errors.contains(&GraphError::UnknownConfigField {
                connector_id: "c1".to_string(),
                field: "typo_field".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_missing_required_field_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("missing required field refused");

        assert_eq!(
            errors,
            vec![GraphError::MissingRequiredField {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
            }]
        );
    }

    #[test]
    fn a_field_type_mismatch_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut config = serde_json::Map::new();
        // `items` on `flow.loop` is `FieldKind::Json` (anything goes) — use
        // a connector whose field is a narrower kind instead: `predicate` on
        // `flow.condition` is `FieldKind::Text`, so a bare number mismatches.
        config.insert("predicate".to_string(), json!(42));
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("type mismatch refused");

        assert_eq!(
            errors,
            vec![GraphError::FieldTypeMismatch {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                expected: "string",
                got: "number",
            }]
        );
    }

    #[test]
    fn a_literal_string_field_with_no_braces_is_not_a_type_mismatch() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![condition("c1", "not an expression, just text")]),
            "c1",
        );

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn an_expression_field_is_never_type_checked_statically() {
        let catalogue = connector_catalogue();
        // `predicate` accepts an expression; whatever it resolves to at
        // runtime is not knowable here, so this must not be flagged.
        let graph = graph_of(vec![condition("c1", "{{ connectors.c2.output.flag }}")]);

        let result = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new());

        // c2 does not exist, so this fails for an unrelated reason (a later
        // rule); the point of this test is that no `FieldTypeMismatch` is
        // ever raised for a dynamic expression.
        if let Err(errors) = result {
            assert!(
                !errors
                    .iter()
                    .any(|e| matches!(e, GraphError::FieldTypeMismatch { .. })),
                "{errors:?}"
            );
        }
    }

    #[test]
    fn an_unparseable_expression_is_refused_and_names_its_position() {
        let catalogue = connector_catalogue();
        let graph = graph_of(vec![condition("c1", "{{ 1 + ")]);

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("syntax error refused");

        let error = errors
            .iter()
            .find(|e| matches!(e, GraphError::InvalidExpression { .. }))
            .unwrap_or_else(|| panic!("expected an InvalidExpression error: {errors:?}"));
        match error {
            GraphError::InvalidExpression {
                connector_id,
                field,
                message,
            } => {
                assert_eq!(connector_id, "c1");
                assert_eq!(field, "predicate");
                assert!(!message.is_empty());
            }
            _ => unreachable!(),
        }
    }

    /// No shipped connector field has `expression: false` today (both
    /// `flow.*` fields accept one), so this rule needs a fabricated
    /// descriptor — the same pattern `connector::catalogue`'s own tests use.
    fn catalogue_with_a_non_expression_field() -> ConnectorCatalogue {
        use crate::domain::automation::connector::{ConnectorDescriptor, Field};

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.plain",
                version: 1,
                family: "test",
                label: "Plain",
                auth: AuthRequirement::None,
                fields: &[Field {
                    name: "plain",
                    label: "Plain",
                    required: true,
                    kind: FieldKind::Text,
                    expression: false,
                    secret: false,
                    visible_when: None,
                }],
                branches: &[],
                output_example: json!({}),
            })
            .expect("first registration succeeds");
        catalogue
    }

    #[test]
    fn a_field_that_forbids_expressions_refuses_one_even_if_it_would_parse() {
        let catalogue = catalogue_with_a_non_expression_field();
        let mut config = serde_json::Map::new();
        config.insert("plain".to_string(), json!("{{ true }}"));
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "test.plain".to_string(),
                version: 1,
                credential_id: None,
                config,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("an expression here is refused");

        assert_eq!(
            errors,
            vec![GraphError::ExpressionNotAllowed {
                connector_id: "c1".to_string(),
                field: "plain".to_string(),
            }]
        );
    }

    #[test]
    fn a_plain_string_on_a_non_expression_field_is_accepted() {
        let catalogue = catalogue_with_a_non_expression_field();
        let mut config = serde_json::Map::new();
        config.insert("plain".to_string(), json!("just text"));
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "test.plain".to_string(),
                version: 1,
                credential_id: None,
                config,
            }]),
            "c1",
        );

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    // --- credential checks -------------------------------------------------
    //
    // Neither shipped connector requires auth (both `flow.*` descriptors are
    // `AuthRequirement::None`), so these rules also need a fabricated
    // descriptor.

    fn catalogue_requiring_bearer_token() -> ConnectorCatalogue {
        use crate::domain::automation::connector::ConnectorDescriptor;

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.http",
                version: 1,
                family: "test",
                label: "HTTP",
                auth: AuthRequirement::Exactly("bearer_token"),
                fields: &[],
                branches: &[],
                output_example: json!({}),
            })
            .expect("first registration succeeds");
        catalogue
    }

    fn credential(id: Uuid, kind: &str) -> Credential {
        use crate::domain::automation::credential::CredentialOrigin;
        use common::{OrganizationId, generate_uuid_v7};

        let now = chrono::Utc::now();
        Credential {
            id,
            org_id: OrganizationId(generate_uuid_v7()),
            kind: kind.to_string(),
            name: "Test credential".to_string(),
            origin: CredentialOrigin::Supplied,
            created_at: now,
            updated_at: now,
        }
    }

    fn http_connector(
        id: &str,
        credential_id: Option<Uuid>,
    ) -> super::super::graph::PlacedConnector {
        super::super::graph::PlacedConnector {
            id: id.to_string(),
            kind: "test.http".to_string(),
            version: 1,
            credential_id,
            config: serde_json::Map::new(),
        }
    }

    #[test]
    fn a_connector_whose_credential_is_optional_is_accepted_without_one() {
        use crate::domain::automation::connector::ConnectorDescriptor;

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.open",
                version: 1,
                family: "test",
                label: "Open",
                auth: AuthRequirement::Optional(&["bearer_token"]),
                fields: &[],
                branches: &[],
                output_example: json!({}),
            })
            .expect("open registers");

        let graph = with_trigger(
            Graph {
                connectors: vec![super::super::graph::PlacedConnector {
                    id: "c1".to_string(),
                    kind: "test.open".to_string(),
                    version: 1,
                    credential_id: None,
                    config: serde_json::Map::new(),
                }],
                edges: vec![],
                triggers: vec![],
            },
            "c1",
        );

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).is_ok());
    }

    #[test]
    fn an_optional_credential_still_has_to_match_an_accepted_scheme() {
        use crate::domain::automation::connector::ConnectorDescriptor;
        use common::generate_uuid_v7;

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.open",
                version: 1,
                family: "test",
                label: "Open",
                auth: AuthRequirement::Optional(&["bearer_token"]),
                fields: &[],
                branches: &[],
                output_example: json!({}),
            })
            .expect("open registers");

        let id = generate_uuid_v7();
        let graph = Graph {
            connectors: vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "test.open".to_string(),
                version: 1,
                credential_id: Some(id),
                config: serde_json::Map::new(),
            }],
            edges: vec![],
            triggers: vec![],
        };

        let errors = validate_graph(
            &graph,
            &catalogue,
            &[credential(id, "odoo_api")],
            &EventCatalogue::new(),
        )
        .expect_err("refused");

        assert!(
            errors.contains(&GraphError::CredentialSchemeNotAccepted {
                connector_id: "c1".to_string(),
                credential_id: id,
                scheme: "odoo_api".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_connector_requiring_a_credential_with_none_set_is_refused_and_named() {
        let catalogue = catalogue_requiring_bearer_token();
        let graph = with_trigger(graph_of(vec![http_connector("c1", None)]), "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("missing credential refused");

        assert_eq!(
            errors,
            vec![GraphError::MissingCredential {
                connector_id: "c1".to_string()
            }]
        );
    }

    #[test]
    fn a_credential_id_absent_from_the_organizations_credentials_is_refused_and_named() {
        let catalogue = catalogue_requiring_bearer_token();
        let missing_id = Uuid::from_u128(42);
        let graph = with_trigger(graph_of(vec![http_connector("c1", Some(missing_id))]), "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("unknown credential refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownCredential {
                connector_id: "c1".to_string(),
                credential_id: missing_id,
            }]
        );
    }

    /// The list of credentials passed in is expected to already be scoped to
    /// the requesting organization (`CredentialRepository::list_by_organization`)
    /// — a credential absent from it, whether from another org or simply
    /// nonexistent, reads back the same way: unknown.
    #[test]
    fn a_credential_from_another_organization_reads_back_as_unknown_the_same_way() {
        let catalogue = catalogue_requiring_bearer_token();
        let stranger_credential_id = Uuid::from_u128(7);
        let graph = with_trigger(
            graph_of(vec![http_connector("c1", Some(stranger_credential_id))]),
            "c1",
        );

        // `credentials` simulates the list already scoped to the caller's
        // organization: the stranger's credential is simply not in it.
        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownCredential {
                connector_id: "c1".to_string(),
                credential_id: stranger_credential_id,
            }]
        );
    }

    #[test]
    fn a_credential_scheme_the_connector_does_not_accept_is_refused_and_named() {
        let catalogue = catalogue_requiring_bearer_token();
        let credential_id = Uuid::from_u128(1);
        let credentials = vec![credential(credential_id, "http_basic")];
        let graph = with_trigger(
            graph_of(vec![http_connector("c1", Some(credential_id))]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &credentials, &EventCatalogue::new())
            .expect_err("wrong scheme refused");

        assert_eq!(
            errors,
            vec![GraphError::CredentialSchemeNotAccepted {
                connector_id: "c1".to_string(),
                credential_id,
                scheme: "http_basic".to_string(),
            }]
        );
    }

    #[test]
    fn a_matching_credential_scheme_is_accepted() {
        let catalogue = catalogue_requiring_bearer_token();
        let credential_id = Uuid::from_u128(1);
        let credentials = vec![credential(credential_id, "bearer_token")];
        let graph = with_trigger(
            graph_of(vec![http_connector("c1", Some(credential_id))]),
            "c1",
        );

        assert_eq!(
            validate_graph(&graph, &catalogue, &credentials, &EventCatalogue::new()),
            Ok(())
        );
    }

    /// A connector that wants no authentication at all (`AuthRequirement::None`,
    /// e.g. `flow.condition`) still refuses a credential attached to it
    /// anyway: `AuthRequirement::None` accepts no scheme, so this falls out
    /// of `CredentialSchemeNotAccepted` with no dedicated rule.
    #[test]
    fn a_credential_on_a_connector_that_wants_no_auth_is_refused() {
        let catalogue = connector_catalogue();
        let credential_id = Uuid::from_u128(1);
        let credentials = vec![credential(credential_id, "bearer_token")];
        let graph = with_trigger(
            graph_of(vec![super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: Some(credential_id),
                config: {
                    let mut m = serde_json::Map::new();
                    m.insert("predicate".to_string(), json!("{{ true }}"));
                    m
                },
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &credentials, &EventCatalogue::new())
            .expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::CredentialSchemeNotAccepted {
                connector_id: "c1".to_string(),
                credential_id,
                scheme: "bearer_token".to_string(),
            }]
        );
    }

    // --- graph shape: dangling edges and branches ---------------------------

    #[test]
    fn an_edge_naming_a_missing_connector_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "ghost".to_string(),
            branch: None,
        });

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("dangling edge refused");

        assert!(
            errors.contains(&GraphError::DanglingEdge {
                from: "c1".to_string(),
                to: "ghost".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_then_edge_between_two_conditions_is_accepted() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn an_edge_from_a_condition_with_no_branch_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: None,
        });

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("a condition needs a branch");

        assert!(
            errors.contains(&GraphError::InvalidBranch {
                connector_id: "c1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_then_edge_from_a_non_condition_connector_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut config = serde_json::Map::new();
        config.insert("name".to_string(), json!("Ada"));
        let mut graph = graph_of(vec![
            PlacedConnector {
                id: "c1".to_string(),
                kind: "mestier.customer.create".to_string(),
                version: 1,
                credential_id: None,
                config,
            },
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("a connector declaring no branch refuses a branched edge");

        assert!(
            errors.contains(&GraphError::InvalidBranch {
                connector_id: "c1".to_string()
            }),
            "{errors:?}"
        );
    }

    // --- graph shape: cycles and reachability -------------------------------

    #[test]
    fn a_cycle_is_refused_and_names_its_members() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        graph.edges.push(Edge {
            from: "c2".to_string(),
            to: "c1".to_string(),
            branch: Some(Branch::Then),
        });

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new())
            .expect_err("cycle refused");

        assert!(
            errors.iter().any(|e| matches!(
                e,
                GraphError::Cycle { connector_ids }
                    if connector_ids == &vec!["c1".to_string(), "c2".to_string()]
            )),
            "{errors:?}"
        );
    }

    #[test]
    fn an_acyclic_graph_has_no_cycle_error() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    /// `c2`/`c3` form a cycle with no edge coming in from `c1`: neither is
    /// ever triggered, independently of the cycle itself.
    #[test]
    fn a_connector_unreachable_from_any_trigger_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
            condition("c3", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c2".to_string(),
            to: "c3".to_string(),
            branch: Some(Branch::Then),
        });
        graph.edges.push(Edge {
            from: "c3".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::UnreachableConnector {
                connector_id: "c2".to_string()
            }),
            "{errors:?}"
        );
        assert!(
            errors.contains(&GraphError::UnreachableConnector {
                connector_id: "c3".to_string()
            }),
            "{errors:?}"
        );
        assert!(
            !errors.contains(&GraphError::UnreachableConnector {
                connector_id: "c1".to_string()
            }),
            "c1 is reachable directly from the trigger: {errors:?}"
        );
    }

    // --- expression graph-relative checks ------------------------------------

    #[test]
    fn an_expression_referencing_an_unknown_connector_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![condition("c1", "{{ connectors.ghost.output.x }}")]),
            "c1",
        );

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownConnectorReference {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                referenced_id: "ghost".to_string(),
            }]
        );
    }

    #[test]
    fn an_expression_referencing_a_downstream_connector_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ connectors.c2.output.x }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::DownstreamConnectorReference {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                referenced_id: "c2".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_expression_referencing_an_upstream_connector_is_accepted() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ connectors.c1.output.x }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn loop_used_outside_a_loop_body_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(graph_of(vec![condition("c1", "{{ loop.item }}")]), "c1");

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::LoopUsedOutsideLoop {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
            }]
        );
    }

    fn loop_connector(id: &str) -> PlacedConnector {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!("{{ trigger.items }}"));
        PlacedConnector {
            id: id.to_string(),
            kind: "flow.loop".to_string(),
            version: 1,
            credential_id: None,
            config,
        }
    }

    #[test]
    fn loop_used_inside_a_loops_each_body_is_accepted() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            loop_connector("loop1"),
            condition("body", "{{ loop.item }}"),
        ]);
        graph.edges.push(Edge {
            from: "loop1".to_string(),
            to: "body".to_string(),
            branch: Some(Branch::Each),
        });
        let graph = with_trigger(graph, "loop1");

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn loop_used_past_a_loops_after_branch_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            loop_connector("loop1"),
            condition("after", "{{ loop.item }}"),
        ]);
        graph.edges.push(Edge {
            from: "loop1".to_string(),
            to: "after".to_string(),
            branch: Some(Branch::After),
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::LoopUsedOutsideLoop {
                connector_id: "after".to_string(),
                field: "predicate".to_string(),
            }),
            "{errors:?}"
        );
    }

    fn catalogue_with_a_custom_branching_connector() -> ConnectorCatalogue {
        use crate::domain::automation::connector::ConnectorDescriptor;

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.fork",
                version: 1,
                family: "test",
                label: "Fork",
                auth: AuthRequirement::None,
                fields: &[],
                branches: &[Branch::Then],
                output_example: json!({}),
            })
            .expect("fork registers");
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.sink",
                version: 1,
                family: "test",
                label: "Sink",
                auth: AuthRequirement::None,
                fields: &[],
                branches: &[],
                output_example: json!({}),
            })
            .expect("sink registers");
        catalogue
    }

    fn fork_graph(branch: Option<Branch>) -> Graph {
        let placed = |id: &str, kind: &str| super::super::graph::PlacedConnector {
            id: id.to_string(),
            kind: kind.to_string(),
            version: 1,
            credential_id: None,
            config: serde_json::Map::new(),
        };

        Graph {
            connectors: vec![placed("fork", "test.fork"), placed("sink", "test.sink")],
            edges: vec![Edge {
                from: "fork".to_string(),
                to: "sink".to_string(),
                branch,
            }],
            triggers: Vec::new(),
        }
    }

    #[test]
    fn a_branch_the_descriptor_declares_is_accepted() {
        let catalogue = catalogue_with_a_custom_branching_connector();

        let graph = with_trigger(fork_graph(Some(Branch::Then)), "fork");

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).is_ok());
    }

    #[test]
    fn a_branch_the_descriptor_does_not_declare_is_refused() {
        let catalogue = catalogue_with_a_custom_branching_connector();

        let graph = fork_graph(Some(Branch::Each));

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::InvalidBranch {
                connector_id: "fork".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_connector_declaring_branches_refuses_an_unbranched_edge() {
        let catalogue = catalogue_with_a_custom_branching_connector();

        let graph = fork_graph(None);

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::InvalidBranch {
                connector_id: "fork".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_custom_connector_declaring_each_opens_a_loop_scope() {
        use crate::domain::automation::connector::{ConnectorDescriptor, Field};

        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.each",
                version: 1,
                family: "test",
                label: "Each",
                auth: AuthRequirement::None,
                fields: &[],
                branches: &[Branch::Each],
                output_example: json!({}),
            })
            .expect("each registers");
        catalogue
            .register(ConnectorDescriptor {
                kind: "test.body",
                version: 1,
                family: "test",
                label: "Body",
                auth: AuthRequirement::None,
                fields: &[Field {
                    name: "predicate",
                    label: "Predicate",
                    required: true,
                    kind: FieldKind::Text,
                    expression: true,
                    secret: false,
                    visible_when: None,
                }],
                branches: &[],
                output_example: json!({}),
            })
            .expect("body registers");

        let mut config = serde_json::Map::new();
        config.insert(
            "predicate".to_string(),
            json!("{{ loop.item }}").as_str().unwrap().into(),
        );

        let graph = with_trigger(
            Graph {
                connectors: vec![
                    super::super::graph::PlacedConnector {
                        id: "each".to_string(),
                        kind: "test.each".to_string(),
                        version: 1,
                        credential_id: None,
                        config: serde_json::Map::new(),
                    },
                    super::super::graph::PlacedConnector {
                        id: "body".to_string(),
                        kind: "test.body".to_string(),
                        version: 1,
                        credential_id: None,
                        config,
                    },
                ],
                edges: vec![Edge {
                    from: "each".to_string(),
                    to: "body".to_string(),
                    branch: Some(Branch::Each),
                }],
                triggers: Vec::new(),
            },
            "each",
        );

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).is_ok());
    }

    // --- graph shape: triggers ----------------------------------------------

    fn manual_trigger(id: &str) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Manual,
        }
    }

    fn events_trigger(id: &str, names: Vec<&str>) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Events(names.into_iter().map(String::from).collect()),
        }
    }

    fn event_catalogue_with(name: &'static str) -> EventCatalogue {
        let mut catalogue = EventCatalogue::new();
        catalogue
            .register(EventDescriptor {
                name,
                version: 1,
                label: "Test event",
                subject_kind: "test",
                payload_example: json!({}),
            })
            .expect("first registration succeeds");
        catalogue
    }

    #[test]
    fn two_triggers_wired_to_different_connectors_validate() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ true }}"),
            condition("c2", "{{ true }}"),
        ]);
        graph.triggers.push(manual_trigger("t1"));
        graph.triggers.push(manual_trigger("t2"));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: None,
        });
        graph.edges.push(Edge {
            from: "t2".to_string(),
            to: "c2".to_string(),
            branch: None,
        });

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()),
            Ok(())
        );
    }

    #[test]
    fn a_graph_with_no_triggers_has_every_connector_unreachable() {
        let catalogue = connector_catalogue();
        let graph = graph_of(vec![condition("c1", "{{ true }}")]);

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::UnreachableConnector {
                connector_id: "c1".to_string()
            }]
        );
    }

    #[test]
    fn a_trigger_wired_to_nothing_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::InertTrigger {
                trigger_id: "t1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_edge_into_a_trigger_is_refused_and_names_it() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: None,
        });
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "t1".to_string(),
            branch: None,
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::EdgeIntoTrigger {
                trigger_id: "t1".to_string(),
                from: "c1".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_edge_between_two_triggers_is_refused_as_an_edge_into_a_trigger() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));
        graph.triggers.push(manual_trigger("t2"));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "t2".to_string(),
            branch: None,
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::EdgeIntoTrigger {
                trigger_id: "t2".to_string(),
                from: "t1".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_branch_on_an_edge_leaving_a_trigger_is_refused() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: Some(Branch::Then),
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::BranchedTriggerEdge {
                trigger_id: "t1".to_string(),
                to: "c1".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_edge_from_a_trigger_to_a_nonexistent_connector_is_a_dangling_edge() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "ghost".to_string(),
            branch: None,
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::DanglingEdge {
                from: "t1".to_string(),
                to: "ghost".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_trigger_id_equal_to_a_connector_id_is_refused() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("c1"));

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::TriggerIdCollidesWithConnectorId {
                id: "c1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn duplicate_trigger_ids_are_refused() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(manual_trigger("t1"));
        graph
            .triggers
            .push(events_trigger("t1", vec!["quote.accepted"]));

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::DuplicateTriggerId {
                id: "t1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_events_trigger_with_an_empty_list_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph.triggers.push(events_trigger("t1", vec![]));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: None,
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::EmptyEventTrigger {
                trigger_id: "t1".to_string()
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_events_trigger_naming_an_unknown_event_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph
            .triggers
            .push(events_trigger("t1", vec!["not.a.real.event"]));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: None,
        });

        let errors =
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new()).expect_err("refused");

        assert!(
            errors.contains(&GraphError::UnknownEventName {
                trigger_id: "t1".to_string(),
                name: "not.a.real.event".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_events_trigger_naming_a_known_event_validates() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
        graph
            .triggers
            .push(events_trigger("t1", vec!["quote.accepted"]));
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c1".to_string(),
            branch: None,
        });

        assert_eq!(
            validate_graph(
                &graph,
                &catalogue,
                &[],
                &event_catalogue_with("quote.accepted")
            ),
            Ok(())
        );
    }
}

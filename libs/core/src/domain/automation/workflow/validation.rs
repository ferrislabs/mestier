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

use crate::domain::automation::collection::{
    COLLECTION_OPERATION_KIND, Collection, ConfigStage, NESTED_EXPRESSION_FIELDS, ParameterIssue,
    check_config,
};
use crate::domain::automation::connector::smtp::{SMTP_SEND_KIND, SmtpFault, literal_faults};
use crate::domain::automation::connector::{ConnectorCatalogue, FieldKind};
use crate::domain::automation::credential::Credential;
use crate::domain::automation::expression::{Template, parse_template};

use super::graph::{Branch, ContractKey, Edge, Graph, PlacedConnector, TriggerKind};
use super::schedule::{parse_time, parse_timezone};
use super::webhook::{WEBHOOK_CREDENTIAL_KINDS, WebhookPathIssue, check_webhook_path};

pub const MAX_NODE_NAME_LENGTH: usize = 120;
pub const MAX_NODE_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_TRIGGERS_PER_GRAPH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRef {
    Connector(String),
    Trigger(String),
}

impl NodeRef {
    pub fn id(&self) -> &str {
        match self {
            Self::Connector(id) | Self::Trigger(id) => id,
        }
    }
}

impl std::fmt::Display for NodeRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connector(id) => write!(formatter, "connector `{id}`"),
            Self::Trigger(id) => write!(formatter, "trigger `{id}`"),
        }
    }
}

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

    #[error("connector `{connector_id}` references unknown collection `{collection_id}`")]
    UnknownCollection {
        connector_id: String,
        collection_id: Uuid,
    },

    #[error(
        "connector `{connector_id}` calls unknown operation `{operation_id}` of collection `{collection_id}`"
    )]
    UnknownOperation {
        connector_id: String,
        collection_id: Uuid,
        operation_id: String,
    },

    #[error("connector `{connector_id}` is missing required parameter `{field}`")]
    MissingParameter { connector_id: String, field: String },

    #[error("connector `{connector_id}` sets unknown parameter `{field}`")]
    UnknownParameter { connector_id: String, field: String },

    #[error("connector `{connector_id}` sets parameter `{field}` to a value the call cannot carry")]
    InvalidParameter { connector_id: String, field: String },

    #[error("connector `{connector_id}` sets parameter `{field}` twice, whatever its case")]
    DuplicateParameter { connector_id: String, field: String },

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

    #[error("connector `{connector_id}` field `{field}` contains a line break")]
    HeaderInjection { connector_id: String, field: String },

    #[error("connector `{connector_id}` sends to more recipients than an e-mail allows")]
    TooManyRecipients { connector_id: String },

    #[error("connector `{connector_id}` field `{field}` references unknown node `{node_name}`")]
    UnknownNodeReference {
        connector_id: String,
        field: String,
        node_name: String,
    },

    #[error(
        "connector `{connector_id}` field `{field}` references node `{node_name}`, which does not run before it"
    )]
    NodeNotUpstream {
        connector_id: String,
        field: String,
        node_name: String,
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

    #[error("trigger `{trigger_id}` has an invalid webhook path: {issue}")]
    InvalidWebhookPath {
        trigger_id: String,
        issue: WebhookPathIssue,
    },

    #[error("trigger `{trigger_id}` reuses the webhook path `{path}` of another trigger")]
    DuplicateWebhookPath { trigger_id: String, path: String },

    #[error("trigger `{trigger_id}` references unknown credential `{credential_id}`")]
    UnknownWebhookCredential {
        trigger_id: String,
        credential_id: Uuid,
    },

    #[error(
        "trigger `{trigger_id}` credential `{credential_id}` has scheme `{scheme}`, which a webhook does not accept"
    )]
    WebhookCredentialSchemeNotAccepted {
        trigger_id: String,
        credential_id: Uuid,
        scheme: String,
    },

    #[error("trigger `{trigger_id}` has a contract key with no name")]
    EmptyContractKeyName { trigger_id: String },

    #[error("trigger `{trigger_id}` names the contract key `{name}` more than once")]
    DuplicateContractKey { trigger_id: String, name: String },

    #[error("trigger `{trigger_id}` has the time `{time}`, which is not HH:MM")]
    InvalidScheduleTime { trigger_id: String, time: String },

    #[error(
        "trigger `{trigger_id}` names the contract key `{name}`, which is not a valid key name"
    )]
    InvalidContractKeyName { trigger_id: String, name: String },

    #[error("a graph holds at most {MAX_TRIGGERS_PER_GRAPH} triggers")]
    TooManyTriggers,

    #[error("trigger `{trigger_id}` has the unknown timezone `{timezone}`")]
    UnknownScheduleTimezone {
        trigger_id: String,
        timezone: String,
    },

    #[error("{node} has an empty name")]
    EmptyNodeName { node: NodeRef },

    #[error("{node} has a name longer than {MAX_NODE_NAME_LENGTH} characters")]
    NodeNameTooLong { node: NodeRef },

    #[error("{node} has a description longer than {MAX_NODE_DESCRIPTION_LENGTH} characters")]
    NodeDescriptionTooLong { node: NodeRef },

    #[error("{node} reuses the name `{name}`, already given to another node")]
    DuplicateNodeName { node: NodeRef, name: String },
}

fn validate_webhook_trigger<'a>(
    trigger_id: &str,
    path: &'a str,
    credential_id: Option<Uuid>,
    contract: &'a [ContractKey],
    credentials: &[Credential],
    seen_paths: &mut HashSet<&'a str>,
    errors: &mut Vec<GraphError>,
) {
    match check_webhook_path(path) {
        Err(issue) => errors.push(GraphError::InvalidWebhookPath {
            trigger_id: trigger_id.to_owned(),
            issue,
        }),
        Ok(()) => {
            if !seen_paths.insert(path) {
                errors.push(GraphError::DuplicateWebhookPath {
                    trigger_id: trigger_id.to_owned(),
                    path: path.to_owned(),
                });
            }
        }
    }

    if let Some(credential_id) = credential_id {
        match credentials
            .iter()
            .find(|credential| credential.id == credential_id)
        {
            None => errors.push(GraphError::UnknownWebhookCredential {
                trigger_id: trigger_id.to_owned(),
                credential_id,
            }),
            Some(credential) => {
                if !WEBHOOK_CREDENTIAL_KINDS.contains(&credential.kind.as_str()) {
                    errors.push(GraphError::WebhookCredentialSchemeNotAccepted {
                        trigger_id: trigger_id.to_owned(),
                        credential_id,
                        scheme: credential.kind.clone(),
                    });
                }
            }
        }
    }

    let mut seen_names: HashSet<&str> = HashSet::new();
    for key in contract {
        if key.name.trim().is_empty() {
            errors.push(GraphError::EmptyContractKeyName {
                trigger_id: trigger_id.to_owned(),
            });
        } else if !is_contract_key_name(&key.name) {
            errors.push(GraphError::InvalidContractKeyName {
                trigger_id: trigger_id.to_owned(),
                name: key.name.clone(),
            });
        } else if !seen_names.insert(key.name.as_str()) {
            errors.push(GraphError::DuplicateContractKey {
                trigger_id: trigger_id.to_owned(),
                name: key.name.clone(),
            });
        }
    }
}

fn is_contract_key_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|next| next.is_ascii_alphanumeric() || matches!(next, '_' | '-'))
}

fn validate_schedule_trigger(
    trigger_id: &str,
    time: &str,
    timezone: Option<&str>,
    errors: &mut Vec<GraphError>,
) {
    if parse_time(time).is_none() {
        errors.push(GraphError::InvalidScheduleTime {
            trigger_id: trigger_id.to_owned(),
            time: time.to_owned(),
        });
    }
    if let Some(timezone) = timezone
        && parse_timezone(timezone).is_none()
    {
        errors.push(GraphError::UnknownScheduleTimezone {
            trigger_id: trigger_id.to_owned(),
            timezone: timezone.to_owned(),
        });
    }
}

fn validate_node_labels(graph: &Graph, errors: &mut Vec<GraphError>) {
    let connectors = graph
        .connectors
        .iter()
        .map(|c| (NodeRef::Connector(c.id.clone()), &c.name, &c.description));
    let triggers = graph
        .triggers
        .iter()
        .map(|t| (NodeRef::Trigger(t.id.clone()), &t.name, &t.description));

    let mut seen_names: HashSet<String> = HashSet::new();
    for (node, name, description) in connectors.chain(triggers) {
        if let Some(name) = name {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                errors.push(GraphError::EmptyNodeName { node: node.clone() });
            } else if trimmed.chars().count() > MAX_NODE_NAME_LENGTH {
                errors.push(GraphError::NodeNameTooLong { node: node.clone() });
            } else if !seen_names.insert(trimmed.to_lowercase()) {
                errors.push(GraphError::DuplicateNodeName {
                    node: node.clone(),
                    name: trimmed.to_owned(),
                });
            }
        }
        if description
            .as_ref()
            .is_some_and(|text| text.chars().count() > MAX_NODE_DESCRIPTION_LENGTH)
        {
            errors.push(GraphError::NodeDescriptionTooLong { node });
        }
    }
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

/// Every node that can reach `of` by forward edges — the set a reference
/// from `of` is allowed to name: a reference to a node that does not run
/// before it is rejected. Walks `predecessors` (the reverse of the edge direction) so it costs one
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

fn check_node_references(
    template: &Template,
    connector_id: &str,
    field: &str,
    ids_by_name: &HashMap<&str, &str>,
    ancestors: &HashSet<&str>,
    errors: &mut Vec<GraphError>,
) {
    for node_name in template.referenced_nodes() {
        match ids_by_name.get(node_name.as_str()) {
            None => errors.push(GraphError::UnknownNodeReference {
                connector_id: connector_id.to_owned(),
                field: field.to_owned(),
                node_name,
            }),
            Some(id) if !ancestors.contains(id) => errors.push(GraphError::NodeNotUpstream {
                connector_id: connector_id.to_owned(),
                field: field.to_owned(),
                node_name,
            }),
            Some(_) => {}
        }
    }
}

fn validate_operation_node(
    connector: &PlacedConnector,
    collections: &[Collection],
    errors: &mut Vec<GraphError>,
) {
    let Some(reference) = &connector.operation else {
        errors.push(GraphError::MissingRequiredField {
            connector_id: connector.id.clone(),
            field: "operation".to_owned(),
        });
        return;
    };
    let Some(collection) = collections
        .iter()
        .find(|collection| collection.id == reference.collection_id)
    else {
        errors.push(GraphError::UnknownCollection {
            connector_id: connector.id.clone(),
            collection_id: reference.collection_id,
        });
        return;
    };
    let Some(operation) = collection.find_operation(&reference.operation_id) else {
        errors.push(GraphError::UnknownOperation {
            connector_id: connector.id.clone(),
            collection_id: reference.collection_id,
            operation_id: reference.operation_id.clone(),
        });
        return;
    };

    for issue in check_config(operation, &connector.config, ConfigStage::Saved) {
        errors.push(match issue {
            ParameterIssue::Missing { field } => GraphError::MissingParameter {
                connector_id: connector.id.clone(),
                field,
            },
            ParameterIssue::Unknown { field } => GraphError::UnknownParameter {
                connector_id: connector.id.clone(),
                field,
            },
            ParameterIssue::InvalidValue { field } => GraphError::InvalidParameter {
                connector_id: connector.id.clone(),
                field,
            },
            ParameterIssue::Duplicate { field } => GraphError::DuplicateParameter {
                connector_id: connector.id.clone(),
                field,
            },
            ParameterIssue::NotAnObject { field, got } => GraphError::FieldTypeMismatch {
                connector_id: connector.id.clone(),
                field: field.to_owned(),
                expected: "object",
                got,
            },
        });
    }
}

fn validate_smtp_node(connector: &PlacedConnector, errors: &mut Vec<GraphError>) {
    for fault in literal_faults(&connector.config) {
        errors.push(match fault {
            SmtpFault::TooManyRecipients => GraphError::TooManyRecipients {
                connector_id: connector.id.clone(),
            },
            SmtpFault::LineBreak(field) => GraphError::HeaderInjection {
                connector_id: connector.id.clone(),
                field: field.to_owned(),
            },
            _ => continue,
        });
    }
}

fn nested_strings<'a>(value: &'a Value, label: &str, found: &mut Vec<(String, &'a Value)>) {
    match value {
        Value::String(_) => found.push((label.to_owned(), value)),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                nested_strings(item, &format!("{label}.{index}"), found);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                nested_strings(item, &format!("{label}.{key}"), found);
            }
        }
        _ => {}
    }
}

fn validate_nested_expressions(
    connector: &PlacedConnector,
    ids_by_name: &HashMap<&str, &str>,
    ancestors: &HashSet<&str>,
    loop_body: &HashSet<&str>,
    errors: &mut Vec<GraphError>,
) {
    let mut found: Vec<(String, &Value)> = Vec::new();
    for field in NESTED_EXPRESSION_FIELDS {
        if let Some(raw @ (Value::Object(_) | Value::Array(_))) = connector.config.get(field) {
            nested_strings(raw, field, &mut found);
        }
    }

    for (label, raw) in found {
        let template = match parse_template(raw) {
            Ok(template) => template,
            Err(error) => {
                errors.push(GraphError::InvalidExpression {
                    connector_id: connector.id.clone(),
                    field: label,
                    message: error.to_string(),
                });
                continue;
            }
        };
        check_node_references(
            &template,
            &connector.id,
            &label,
            ids_by_name,
            ancestors,
            errors,
        );
        if template.uses_loop() && !loop_body.contains(connector.id.as_str()) {
            errors.push(GraphError::LoopUsedOutsideLoop {
                connector_id: connector.id.clone(),
                field: label,
            });
        }
    }
}

/// Rends **toutes** les erreurs, pas la première.
pub fn validate_graph(
    graph: &Graph,
    catalogue: &ConnectorCatalogue,
    credentials: &[Credential],
    events: &EventCatalogue,
    collections: &[Collection],
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

    if graph.triggers.len() > MAX_TRIGGERS_PER_GRAPH {
        errors.push(GraphError::TooManyTriggers);
    }

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
    let mut seen_webhook_paths: HashSet<&str> = HashSet::new();
    for trigger in &graph.triggers {
        if !triggers_with_an_outgoing_edge.contains(trigger.id.as_str()) {
            errors.push(GraphError::InertTrigger {
                trigger_id: trigger.id.clone(),
            });
        }

        match &trigger.kind {
            TriggerKind::Manual => {}
            TriggerKind::Events(names) => {
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
            TriggerKind::Webhook {
                path,
                credential_id,
                contract,
                ..
            } => validate_webhook_trigger(
                &trigger.id,
                path,
                *credential_id,
                contract,
                credentials,
                &mut seen_webhook_paths,
                &mut errors,
            ),
            TriggerKind::Schedule { time, timezone, .. } => {
                validate_schedule_trigger(&trigger.id, time, timezone.as_deref(), &mut errors)
            }
        }
    }

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
    for edge in &trigger_edges {
        predecessors
            .entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
    }

    let mut ids_by_name: HashMap<&str, &str> = HashMap::new();
    let named_nodes = graph
        .connectors
        .iter()
        .map(|c| (c.name.as_deref(), c.id.as_str()))
        .chain(
            graph
                .triggers
                .iter()
                .map(|t| (t.name.as_deref(), t.id.as_str())),
        );
    for (name, id) in named_nodes {
        if let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) {
            ids_by_name.entry(name).or_insert(id);
        }
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

            check_node_references(
                &template,
                &connector.id,
                field.name,
                &ids_by_name,
                &ancestors,
                &mut errors,
            );

            if template.uses_loop() && !loop_body.contains(connector.id.as_str()) {
                errors.push(GraphError::LoopUsedOutsideLoop {
                    connector_id: connector.id.clone(),
                    field: field.name.to_string(),
                });
            }
        }

        if connector.kind == SMTP_SEND_KIND {
            validate_smtp_node(connector, &mut errors);
        }

        if connector.kind == COLLECTION_OPERATION_KIND {
            validate_nested_expressions(
                connector,
                &ids_by_name,
                &ancestors,
                &loop_body,
                &mut errors,
            );
            validate_operation_node(connector, collections, &mut errors);
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

    validate_node_labels(graph, &mut errors);

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
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::connector::{AuthRequirement, ExecutionMode};
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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
            name: None,
            description: None,
        });
        graph.edges.push(Edge {
            from: "t0".to_string(),
            to: target.to_string(),
            branch: None,
        });
        graph
    }

    fn labelled(
        mut connector: super::super::graph::PlacedConnector,
        name: Option<&str>,
        description: Option<&str>,
    ) -> super::super::graph::PlacedConnector {
        connector.name = name.map(str::to_owned);
        connector.description = description.map(str::to_owned);
        connector
    }

    fn node_errors(graph: &Graph) -> Vec<GraphError> {
        let catalogue = connector_catalogue();
        match validate_graph(graph, &catalogue, &[], &EventCatalogue::new(), &[]) {
            Ok(()) => Vec::new(),
            Err(errors) => errors,
        }
    }

    fn connector_ref(id: &str) -> NodeRef {
        NodeRef::Connector(id.to_string())
    }

    fn trigger_ref(id: &str) -> NodeRef {
        NodeRef::Trigger(id.to_string())
    }

    #[test]
    fn named_and_described_nodes_are_valid() {
        let graph = with_trigger(
            graph_of(vec![labelled(
                condition("c1", "{{ true }}"),
                Some("Is it paid"),
                Some("Stops unpaid quotes"),
            )]),
            "c1",
        );

        assert_eq!(node_errors(&graph), Vec::new());
    }

    #[test]
    fn a_blank_node_name_is_refused_for_a_connector_and_a_trigger() {
        let mut graph = with_trigger(
            graph_of(vec![labelled(
                condition("c1", "{{ true }}"),
                Some("   "),
                None,
            )]),
            "c1",
        );
        graph.triggers[0].name = Some(String::new());

        let errors = node_errors(&graph);

        assert!(
            errors.contains(&GraphError::EmptyNodeName {
                node: connector_ref("c1")
            }),
            "{errors:?}"
        );
        assert!(
            errors.contains(&GraphError::EmptyNodeName {
                node: trigger_ref("t0")
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_node_name_holds_at_most_120_characters_after_trimming() {
        let fits = format!("  {}  ", "é".repeat(120));
        let too_long = "é".repeat(121);
        let graph = with_trigger(
            graph_of(vec![
                labelled(condition("c1", "{{ true }}"), Some(&fits), None),
                labelled(condition("c2", "{{ true }}"), Some(&too_long), None),
            ]),
            "c1",
        );

        let errors = node_errors(&graph);

        assert!(
            !errors.contains(&GraphError::NodeNameTooLong {
                node: connector_ref("c1")
            }),
            "{errors:?}"
        );
        assert!(
            errors.contains(&GraphError::NodeNameTooLong {
                node: connector_ref("c2")
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn a_node_description_holds_at_most_500_characters() {
        let fits = "é".repeat(500);
        let too_long = "é".repeat(501);
        let mut graph = with_trigger(
            graph_of(vec![
                labelled(condition("c1", "{{ true }}"), None, Some(&fits)),
                labelled(condition("c2", "{{ true }}"), None, Some(&too_long)),
            ]),
            "c1",
        );
        graph.triggers[0].description = Some("x".repeat(501));

        let errors = node_errors(&graph);

        assert!(
            !errors.contains(&GraphError::NodeDescriptionTooLong {
                node: connector_ref("c1")
            }),
            "{errors:?}"
        );
        assert!(
            errors.contains(&GraphError::NodeDescriptionTooLong {
                node: connector_ref("c2")
            }),
            "{errors:?}"
        );
        assert!(
            errors.contains(&GraphError::NodeDescriptionTooLong {
                node: trigger_ref("t0")
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn two_connectors_cannot_share_a_name_ignoring_case_and_padding() {
        let graph = with_trigger(
            graph_of(vec![
                labelled(condition("c1", "{{ true }}"), Some("Is it paid"), None),
                labelled(condition("c2", "{{ true }}"), Some("  IS IT PAID "), None),
            ]),
            "c1",
        );

        let errors = node_errors(&graph);

        assert!(
            errors.contains(&GraphError::DuplicateNodeName {
                node: connector_ref("c2"),
                name: "IS IT PAID".to_string(),
            }),
            "{errors:?}"
        );
        assert!(
            !errors
                .iter()
                .any(|error| matches!(error, GraphError::DuplicateNodeName { node, .. } if *node == connector_ref("c1"))),
            "the first holder of a name is not at fault: {errors:?}"
        );
    }

    #[test]
    fn a_trigger_and_a_connector_cannot_share_a_name() {
        let mut graph = with_trigger(
            graph_of(vec![labelled(
                condition("c1", "{{ true }}"),
                Some("Start"),
                None,
            )]),
            "c1",
        );
        graph.triggers[0].name = Some("start".to_string());

        let errors = node_errors(&graph);

        assert!(
            errors.contains(&GraphError::DuplicateNodeName {
                node: trigger_ref("t0"),
                name: "start".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn unnamed_nodes_never_collide() {
        let mut graph = with_trigger(
            graph_of(vec![
                condition("c1", "{{ true }}"),
                condition("c2", "{{ true }}"),
            ]),
            "c1",
        );
        graph.edges.push(Edge {
            from: "t0".to_string(),
            to: "c2".to_string(),
            branch: None,
        });

        assert_eq!(node_errors(&graph), Vec::new());
    }

    #[test]
    fn a_blank_name_is_not_also_reported_as_a_duplicate() {
        let graph = with_trigger(
            graph_of(vec![
                labelled(condition("c1", "{{ true }}"), Some(" "), None),
                labelled(condition("c2", "{{ true }}"), Some("  "), None),
            ]),
            "c1",
        );

        let errors = node_errors(&graph);

        assert!(
            !errors
                .iter()
                .any(|error| matches!(error, GraphError::DuplicateNodeName { .. })),
            "{errors:?}"
        );
    }

    #[test]
    fn node_label_errors_read_like_the_other_graph_errors() {
        assert_eq!(
            GraphError::DuplicateNodeName {
                node: connector_ref("c2"),
                name: "Is it paid".to_string(),
            }
            .to_string(),
            "connector `c2` reuses the name `Is it paid`, already given to another node"
        );
        assert_eq!(
            GraphError::EmptyNodeName {
                node: trigger_ref("t1")
            }
            .to_string(),
            "trigger `t1` has an empty name"
        );
        assert_eq!(
            GraphError::NodeNameTooLong {
                node: connector_ref("c1")
            }
            .to_string(),
            "connector `c1` has a name longer than 120 characters"
        );
        assert_eq!(
            GraphError::NodeDescriptionTooLong {
                node: connector_ref("c1")
            }
            .to_string(),
            "connector `c1` has a description longer than 500 characters"
        );
    }

    #[test]
    fn a_disabled_connector_is_validated_like_any_other() {
        let mut connector = condition("c1", "{{ true }}");
        connector.disabled = true;
        connector.config.clear();
        let graph = with_trigger(graph_of(vec![connector]), "c1");

        let errors = node_errors(&graph);

        assert!(
            errors.contains(&GraphError::MissingRequiredField {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn an_empty_graph_is_valid() {
        let catalogue = connector_catalogue();
        assert_eq!(
            validate_graph(
                &Graph::default(),
                &catalogue,
                &[],
                &EventCatalogue::new(),
                &[]
            ),
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
        }]);

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
            Ok(())
        );
    }

    #[test]
    fn an_expression_field_is_never_type_checked_statically() {
        let catalogue = connector_catalogue();
        // `predicate` accepts an expression; whatever it resolves to at
        // runtime is not knowable here, so this must not be flagged.
        let graph = graph_of(vec![condition("c1", "{{ $('Autre').item.json.flag }}")]);

        let result = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]);

        // `Autre` does not exist, so this fails for an unrelated reason (a
        // later rule); the point of this test is that no `FieldTypeMismatch`
        // is ever raised for a dynamic expression.
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
            })
            .expect("first registration succeeds");
        catalogue
    }

    fn credential(id: Uuid, kind: &str) -> Credential {
        use crate::domain::automation::credential::CredentialOrigin;
        use common::{OrganizationId, generate_uuid_v7};

        let mut credential =
            Credential::fixture(OrganizationId(generate_uuid_v7()), kind, "Test credential");
        credential.id = id;
        credential.origin = CredentialOrigin::Supplied;
        credential
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                }],
                edges: vec![],
                triggers: vec![],
            },
            "c1",
        );

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]).is_ok());
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: vec![],
            triggers: vec![],
        };

        let errors = validate_graph(
            &graph,
            &catalogue,
            &[credential(id, "odoo_api")],
            &EventCatalogue::new(),
            &[],
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(
            &graph,
            &catalogue,
            &credentials,
            &EventCatalogue::new(),
            &[],
        )
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
            validate_graph(
                &graph,
                &catalogue,
                &credentials,
                &EventCatalogue::new(),
                &[]
            ),
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }]),
            "c1",
        );

        let errors = validate_graph(
            &graph,
            &catalogue,
            &credentials,
            &EventCatalogue::new(),
            &[],
        )
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
                name: None,
                description: None,
                disabled: false,
                operation: None,
            },
            condition("c2", "{{ true }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
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
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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
    fn an_expression_referencing_an_unknown_node_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(
            graph_of(vec![condition("c1", "{{ $('Fantôme').item.json.x }}")]),
            "c1",
        );

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::UnknownNodeReference {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                node_name: "Fantôme".to_string(),
            }]
        );
        assert_eq!(
            errors[0].to_string(),
            "connector `c1` field `predicate` references unknown node `Fantôme`"
        );
    }

    #[test]
    fn an_expression_referencing_a_downstream_node_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ $('Second').item.json.x }}"),
            labelled(condition("c2", "{{ true }}"), Some("Second"), None),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

        assert_eq!(
            errors,
            vec![GraphError::NodeNotUpstream {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                node_name: "Second".to_string(),
            }]
        );
    }

    #[test]
    fn an_expression_referencing_a_sibling_branch_or_itself_is_refused() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            labelled(condition("c1", "{{ true }}"), Some("Début"), None),
            labelled(
                condition("c2", "{{ $('Côté B').item.json.x }}"),
                Some("Côté A"),
                None,
            ),
            labelled(
                condition("c3", "{{ $('Côté B').item.json.x }}"),
                Some("Côté B"),
                None,
            ),
        ]);
        for (to, branch) in [("c2", Branch::Then), ("c3", Branch::Else)] {
            graph.edges.push(Edge {
                from: "c1".to_string(),
                to: to.to_string(),
                branch: Some(branch),
            });
        }
        let graph = with_trigger(graph, "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

        assert_eq!(
            errors,
            vec![
                GraphError::NodeNotUpstream {
                    connector_id: "c2".to_string(),
                    field: "predicate".to_string(),
                    node_name: "Côté B".to_string(),
                },
                GraphError::NodeNotUpstream {
                    connector_id: "c3".to_string(),
                    field: "predicate".to_string(),
                    node_name: "Côté B".to_string(),
                },
            ]
        );
    }

    #[test]
    fn an_expression_referencing_an_upstream_node_is_accepted() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            labelled(condition("c1", "{{ true }}"), Some("Premier"), None),
            condition("c2", "{{ $('Premier').item.json.x }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
            Ok(())
        );
    }

    #[test]
    fn an_expression_may_reference_a_distant_ancestor_and_a_trigger_by_name() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            labelled(condition("c1", "{{ true }}"), Some("Premier"), None),
            condition("c2", "{{ true }}"),
            condition(
                "c3",
                "{{ $('Premier').first().json.x and $('Départ').item.json.y }}",
            ),
        ]);
        for (from, to) in [("c1", "c2"), ("c2", "c3")] {
            graph.edges.push(Edge {
                from: from.to_string(),
                to: to.to_string(),
                branch: Some(Branch::Then),
            });
        }
        let mut graph = with_trigger(graph, "c1");
        graph.triggers[0].name = Some("Départ".to_string());

        assert_eq!(
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
            Ok(())
        );
    }

    #[test]
    fn a_trigger_that_does_not_feed_the_node_is_not_upstream() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            condition("c1", "{{ $('Autre départ').item.json.y }}"),
            condition("c2", "{{ $('Autre départ').item.json.y }}"),
        ]);
        graph.edges.push(Edge {
            from: "t1".to_string(),
            to: "c2".to_string(),
            branch: None,
        });
        graph.triggers.push(PlacedTrigger {
            id: "t1".to_string(),
            kind: TriggerKind::Manual,
            name: Some("Autre départ".to_string()),
            description: None,
        });
        let graph = with_trigger(graph, "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("c1 is not fed by the trigger it names");

        assert_eq!(
            errors,
            vec![GraphError::NodeNotUpstream {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
                node_name: "Autre départ".to_string(),
            }]
        );
    }

    #[test]
    fn a_node_name_is_matched_exactly_after_trimming() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            labelled(condition("c1", "{{ true }}"), Some("  Premier  "), None),
            condition("c2", "{{ $('Premier').item.json.x }}"),
            condition("c3", "{{ $('premier').item.json.x }}"),
        ]);
        for to in ["c2", "c3"] {
            graph.edges.push(Edge {
                from: "c1".to_string(),
                to: to.to_string(),
                branch: Some(Branch::Then),
            });
        }
        let graph = with_trigger(graph, "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("the lower-case name is another name");

        assert_eq!(
            errors,
            vec![GraphError::UnknownNodeReference {
                connector_id: "c3".to_string(),
                field: "predicate".to_string(),
                node_name: "premier".to_string(),
            }]
        );
    }

    #[test]
    fn the_connectors_root_is_refused_with_a_message_naming_the_new_syntax() {
        let catalogue = connector_catalogue();
        let mut graph = graph_of(vec![
            labelled(condition("c1", "{{ true }}"), Some("Premier"), None),
            condition("c2", "{{ connectors.c1.output.matched }}"),
        ]);
        graph.edges.push(Edge {
            from: "c1".to_string(),
            to: "c2".to_string(),
            branch: Some(Branch::Then),
        });
        let graph = with_trigger(graph, "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

        assert_eq!(errors.len(), 1, "{errors:?}");
        let GraphError::InvalidExpression {
            connector_id,
            field,
            message,
        } = &errors[0]
        else {
            panic!("expected an invalid expression, got {errors:?}");
        };
        assert_eq!((connector_id.as_str(), field.as_str()), ("c2", "predicate"));
        assert!(message.contains("$('Nom du nœud').item.json"), "{message}");
        assert!(
            message.contains("cette expression utilise l'ancienne syntaxe"),
            "{message}"
        );
    }

    #[test]
    fn loop_used_outside_a_loop_body_is_refused_and_named() {
        let catalogue = connector_catalogue();
        let graph = with_trigger(graph_of(vec![condition("c1", "{{ loop.item }}")]), "c1");

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
            name: None,
            description: None,
            disabled: false,
            operation: None,
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

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]).is_ok());
    }

    #[test]
    fn a_branch_the_descriptor_does_not_declare_is_refused() {
        let catalogue = catalogue_with_a_custom_branching_connector();

        let graph = fork_graph(Some(Branch::Each));

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                execution: ExecutionMode::PerItem,
                output_example: json!({}),
                output_mirrors_field: None,
                sample_output: json!({}),
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
                        name: None,
                        description: None,
                        disabled: false,
                        operation: None,
                    },
                    super::super::graph::PlacedConnector {
                        id: "body".to_string(),
                        kind: "test.body".to_string(),
                        version: 1,
                        credential_id: None,
                        config,
                        name: None,
                        description: None,
                        disabled: false,
                        operation: None,
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

        assert!(validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]).is_ok());
    }

    fn manual_trigger(id: &str) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Manual,
            name: None,
            description: None,
        }
    }

    fn events_trigger(id: &str, names: Vec<&str>) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Events(names.into_iter().map(String::from).collect()),
            name: None,
            description: None,
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
            validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[]),
            Ok(())
        );
    }

    #[test]
    fn a_graph_with_no_triggers_has_every_connector_unreachable() {
        let catalogue = connector_catalogue();
        let graph = graph_of(vec![condition("c1", "{{ true }}")]);

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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

        let errors = validate_graph(&graph, &catalogue, &[], &EventCatalogue::new(), &[])
            .expect_err("refused");

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
                &event_catalogue_with("quote.accepted"),
                &[]
            ),
            Ok(())
        );
    }

    mod new_triggers {
        use uuid::Uuid;

        use super::super::super::graph::{
            ContractKey, ContractKind, Frequency, OnInvalid, WebhookMethod, Weekday,
        };
        use super::*;

        fn graph_with(kind: TriggerKind) -> Graph {
            let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
            graph.triggers.push(PlacedTrigger {
                id: "t1".to_string(),
                kind,
                name: None,
                description: None,
            });
            graph.edges.push(Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            });
            graph
        }

        fn webhook(path: &str) -> TriggerKind {
            TriggerKind::Webhook {
                method: WebhookMethod::Post,
                path: path.to_string(),
                credential_id: None,
                contract: Vec::new(),
                on_invalid: OnInvalid::Reject,
            }
        }

        fn webhook_with_credential(credential_id: Uuid) -> TriggerKind {
            TriggerKind::Webhook {
                method: WebhookMethod::Post,
                path: "paiement-recu".to_string(),
                credential_id: Some(credential_id),
                contract: Vec::new(),
                on_invalid: OnInvalid::Reject,
            }
        }

        fn webhook_with_contract(names: &[&str]) -> TriggerKind {
            TriggerKind::Webhook {
                method: WebhookMethod::Post,
                path: "paiement-recu".to_string(),
                credential_id: None,
                contract: names
                    .iter()
                    .map(|name| ContractKey {
                        name: (*name).to_string(),
                        kind: ContractKind::Text,
                        required: true,
                        example: String::new(),
                    })
                    .collect(),
                on_invalid: OnInvalid::Reject,
            }
        }

        fn schedule(frequency: Frequency, time: &str) -> TriggerKind {
            TriggerKind::Schedule {
                frequency,
                time: time.to_string(),
                weekday: None,
                timezone: None,
            }
        }

        fn errors_with(graph: &Graph, credentials: &[Credential]) -> Vec<GraphError> {
            let catalogue = connector_catalogue();
            match validate_graph(graph, &catalogue, credentials, &EventCatalogue::new(), &[]) {
                Ok(()) => Vec::new(),
                Err(errors) => errors,
            }
        }

        #[test]
        fn a_complete_webhook_and_a_complete_schedule_validate() {
            let credential_id = Uuid::from_u128(5);
            let mut graph = graph_with(webhook_with_credential(credential_id));
            graph.triggers.push(PlacedTrigger {
                id: "t2".to_string(),
                kind: TriggerKind::Schedule {
                    frequency: Frequency::Weekly,
                    time: "09:00".to_string(),
                    weekday: Some(Weekday::Monday),
                    timezone: Some("Europe/Paris".to_string()),
                },
                name: None,
                description: None,
            });
            graph.edges.push(Edge {
                from: "t2".to_string(),
                to: "c1".to_string(),
                branch: None,
            });

            let errors = errors_with(&graph, &[credential(credential_id, "bearer_token")]);

            assert_eq!(errors, Vec::new());
        }

        #[test]
        fn a_webhook_path_breaking_a_rule_is_refused_and_names_the_trigger() {
            let cases = [
                ("", WebhookPathIssue::Empty),
                ("webhooks/x", WebhookPathIssue::Prefixed),
                ("/x", WebhookPathIssue::EmptySegment),
                ("a/../b", WebhookPathIssue::ParentSegment),
                ("a//b", WebhookPathIssue::EmptySegment),
                ("Paiement Reçu", WebhookPathIssue::InvalidCharacters),
                (&"a".repeat(201), WebhookPathIssue::TooLong),
            ];

            for (path, issue) in cases {
                let errors = errors_with(&graph_with(webhook(path)), &[]);

                assert_eq!(
                    errors,
                    vec![GraphError::InvalidWebhookPath {
                        trigger_id: "t1".to_string(),
                        issue
                    }],
                    "`{path}`"
                );
            }
        }

        #[test]
        fn two_webhooks_of_one_graph_cannot_share_a_path() {
            let mut graph = graph_with(webhook("paiement-recu"));
            graph.triggers.push(PlacedTrigger {
                id: "t2".to_string(),
                kind: webhook("paiement-recu"),
                name: None,
                description: None,
            });
            graph.edges.push(Edge {
                from: "t2".to_string(),
                to: "c1".to_string(),
                branch: None,
            });

            let errors = errors_with(&graph, &[]);

            assert_eq!(
                errors,
                vec![GraphError::DuplicateWebhookPath {
                    trigger_id: "t2".to_string(),
                    path: "paiement-recu".to_string(),
                }]
            );
        }

        #[test]
        fn a_webhook_credential_that_does_not_exist_is_refused() {
            let missing = Uuid::from_u128(404);

            let errors = errors_with(&graph_with(webhook_with_credential(missing)), &[]);

            assert_eq!(
                errors,
                vec![GraphError::UnknownWebhookCredential {
                    trigger_id: "t1".to_string(),
                    credential_id: missing,
                }]
            );
        }

        #[test]
        fn a_webhook_accepts_header_basic_bearer_and_hmac_credentials_only() {
            let id = Uuid::from_u128(5);

            for scheme in ["http_header", "http_basic", "bearer_token", "hmac"] {
                let errors = errors_with(
                    &graph_with(webhook_with_credential(id)),
                    &[credential(id, scheme)],
                );
                assert_eq!(errors, Vec::new(), "{scheme}");
            }

            for scheme in [
                "odoo_api",
                "anything_else",
                "jwt",
                "digest",
                "api_key_query",
                "smtp",
            ] {
                let errors = errors_with(
                    &graph_with(webhook_with_credential(id)),
                    &[credential(id, scheme)],
                );
                assert_eq!(
                    errors,
                    vec![GraphError::WebhookCredentialSchemeNotAccepted {
                        trigger_id: "t1".to_string(),
                        credential_id: id,
                        scheme: scheme.to_string(),
                    }],
                    "{scheme}"
                );
            }
        }

        #[test]
        fn contract_key_names_must_be_unique_and_not_blank() {
            let errors = errors_with(
                &graph_with(webhook_with_contract(&["a", "b", "a", "", "  ", "b"])),
                &[],
            );

            assert_eq!(
                errors,
                vec![
                    GraphError::DuplicateContractKey {
                        trigger_id: "t1".to_string(),
                        name: "a".to_string(),
                    },
                    GraphError::EmptyContractKeyName {
                        trigger_id: "t1".to_string(),
                    },
                    GraphError::EmptyContractKeyName {
                        trigger_id: "t1".to_string(),
                    },
                    GraphError::DuplicateContractKey {
                        trigger_id: "t1".to_string(),
                        name: "b".to_string(),
                    },
                ]
            );
        }

        #[test]
        fn contract_key_names_that_differ_only_by_case_are_distinct() {
            let errors = errors_with(&graph_with(webhook_with_contract(&["a", "A"])), &[]);

            assert_eq!(errors, Vec::new());
        }

        #[test]
        fn a_contract_key_name_follows_the_rule_of_the_prototype() {
            for name in ["a", "_a", "A", "montant_ht", "client-nom", "a1", "_"] {
                let errors = errors_with(&graph_with(webhook_with_contract(&[name])), &[]);

                assert_eq!(errors, Vec::new(), "`{name}`");
            }
            for name in [
                "1a", "-a", "a b", "a.b", "é", "a/b", " a", "a ", "a\u{0}b", "{{x}}", "a$",
            ] {
                let errors = errors_with(&graph_with(webhook_with_contract(&[name])), &[]);

                assert_eq!(
                    errors,
                    vec![GraphError::InvalidContractKeyName {
                        trigger_id: "t1".to_string(),
                        name: name.to_string(),
                    }],
                    "`{name}`"
                );
            }
        }

        #[test]
        fn a_graph_holds_at_most_twenty_triggers() {
            let many = |count: usize| {
                let mut graph = graph_of(vec![condition("c1", "{{ true }}")]);
                for index in 0..count {
                    let id = format!("t{index}");
                    graph.triggers.push(PlacedTrigger {
                        id: id.clone(),
                        kind: TriggerKind::Manual,
                        name: None,
                        description: None,
                    });
                    graph.edges.push(Edge {
                        from: id,
                        to: "c1".to_string(),
                        branch: None,
                    });
                }
                graph
            };

            assert_eq!(errors_with(&many(20), &[]), Vec::new());
            assert_eq!(
                errors_with(&many(21), &[]),
                vec![GraphError::TooManyTriggers]
            );
        }

        #[test]
        fn a_schedule_time_must_be_hh_mm() {
            for time in ["", "9:00", "24:00", "09:60", "09h00", "09:00:00"] {
                let errors = errors_with(&graph_with(schedule(Frequency::Daily, time)), &[]);

                assert_eq!(
                    errors,
                    vec![GraphError::InvalidScheduleTime {
                        trigger_id: "t1".to_string(),
                        time: time.to_string(),
                    }],
                    "`{time}`"
                );
            }
        }

        #[test]
        fn a_schedule_time_is_checked_for_every_frequency() {
            for frequency in [Frequency::Hourly, Frequency::Daily, Frequency::Weekly] {
                let mut kind = schedule(frequency, "99:99");
                if let TriggerKind::Schedule { weekday, .. } = &mut kind {
                    *weekday = Some(Weekday::Monday);
                }

                let errors = errors_with(&graph_with(kind), &[]);

                assert_eq!(errors.len(), 1, "{frequency:?}: {errors:?}");
            }
        }

        #[test]
        fn a_weekly_schedule_may_leave_its_weekday_out() {
            for frequency in [Frequency::Hourly, Frequency::Daily, Frequency::Weekly] {
                assert_eq!(
                    errors_with(&graph_with(schedule(frequency, "09:00")), &[]),
                    Vec::new(),
                    "{frequency:?}"
                );
            }
        }

        #[test]
        fn a_schedule_timezone_must_be_a_known_iana_zone_when_given() {
            let with_zone = |zone: &str| TriggerKind::Schedule {
                frequency: Frequency::Daily,
                time: "09:00".to_string(),
                weekday: None,
                timezone: Some(zone.to_string()),
            };

            assert_eq!(
                errors_with(&graph_with(with_zone("Europe/Paris")), &[]),
                Vec::new()
            );
            for zone in ["", "Mars/Olympus", "paris", " Europe/Paris"] {
                assert_eq!(
                    errors_with(&graph_with(with_zone(zone)), &[]),
                    vec![GraphError::UnknownScheduleTimezone {
                        trigger_id: "t1".to_string(),
                        timezone: zone.to_string(),
                    }],
                    "`{zone}`"
                );
            }
        }

        #[test]
        fn every_fault_of_a_trigger_is_reported_together() {
            let kind = TriggerKind::Schedule {
                frequency: Frequency::Weekly,
                time: "nope".to_string(),
                weekday: None,
                timezone: Some("Mars/Olympus".to_string()),
            };

            let errors = errors_with(&graph_with(kind), &[]);

            assert_eq!(
                errors,
                vec![
                    GraphError::InvalidScheduleTime {
                        trigger_id: "t1".to_string(),
                        time: "nope".to_string(),
                    },
                    GraphError::UnknownScheduleTimezone {
                        trigger_id: "t1".to_string(),
                        timezone: "Mars/Olympus".to_string(),
                    },
                ]
            );
        }

        #[test]
        fn a_webhook_or_schedule_without_an_outgoing_edge_is_inert() {
            for kind in [webhook("ping"), schedule(Frequency::Daily, "09:00")] {
                let mut graph = graph_with(kind);
                graph.edges.clear();
                graph.connectors.clear();

                assert_eq!(
                    errors_with(&graph, &[]),
                    vec![GraphError::InertTrigger {
                        trigger_id: "t1".to_string()
                    }]
                );
            }
        }
    }

    mod operation_nodes {
        use chrono::Utc;
        use common::OrganizationId;
        use uuid::Uuid;

        use super::super::super::graph::OperationRef;
        use super::*;
        use crate::domain::automation::collection::{
            Collection, CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod,
            Operation, Parameter, ParameterLocation, RequestBody,
        };

        const COLLECTION_ID: u128 = 900;

        fn operation() -> Operation {
            Operation {
                id: "getAddress".to_owned(),
                method: HttpMethod::Get,
                path: "/addresses/{id}".to_owned(),
                summary: String::new(),
                tag: String::new(),
                parameters: vec![
                    Parameter {
                        name: "id".to_owned(),
                        location: ParameterLocation::Path,
                        required: true,
                        description: String::new(),
                        schema: json!({ "type": "string" }),
                    },
                    Parameter {
                        name: "limit".to_owned(),
                        location: ParameterLocation::Query,
                        required: false,
                        description: String::new(),
                        schema: json!({ "type": "integer" }),
                    },
                ],
                request_body: Some(RequestBody {
                    schema: json!({ "type": "object" }),
                }),
                response: None,
                response_status: None,
            }
        }

        fn collection() -> Collection {
            let draft = CollectionDraft::new(CollectionDraftInput {
                title: "Addresses".to_owned(),
                version: "1".to_owned(),
                description: String::new(),
                base_url: "https://api.example.com".to_owned(),
                source: CollectionSource::Url {
                    url: "https://api.example.com/openapi.json".to_owned(),
                },
                security: Vec::new(),
                operations: vec![operation()],
                schemas: serde_json::Map::new(),
            })
            .unwrap();
            Collection::from_draft(
                draft,
                Uuid::from_u128(COLLECTION_ID),
                OrganizationId(Uuid::from_u128(1)),
                "addresses".to_owned(),
                Utc::now(),
            )
        }

        fn node(id: &str, config: Value) -> super::super::super::graph::PlacedConnector {
            super::super::super::graph::PlacedConnector {
                id: id.to_string(),
                kind: "collection.operation".to_string(),
                version: 1,
                credential_id: None,
                config: config.as_object().cloned().expect("a config is an object"),
                name: None,
                description: None,
                disabled: false,
                operation: Some(OperationRef {
                    collection_id: Uuid::from_u128(COLLECTION_ID),
                    operation_id: "getAddress".to_owned(),
                }),
            }
        }

        fn errors_with(graph: &Graph, collections: &[Collection]) -> Vec<GraphError> {
            let catalogue = connector_catalogue();
            match validate_graph(graph, &catalogue, &[], &EventCatalogue::new(), collections) {
                Ok(()) => Vec::new(),
                Err(errors) => errors,
            }
        }

        fn graph_of_node(node: super::super::super::graph::PlacedConnector) -> Graph {
            with_trigger(graph_of(vec![node]), "c1")
        }

        #[test]
        fn a_node_naming_a_stored_operation_with_its_parameters_is_valid() {
            let graph = graph_of_node(node(
                "c1",
                json!({ "path": { "id": "42" }, "query": { "limit": 5 }, "body": { "a": 1 } }),
            ));

            assert_eq!(errors_with(&graph, &[collection()]), Vec::new());
        }

        #[test]
        fn parameters_given_as_whole_expressions_are_accepted() {
            let graph = graph_of_node(node(
                "c1",
                json!({ "path": "{{ trigger.path }}", "query": "{{ trigger.query }}" }),
            ));

            assert_eq!(errors_with(&graph, &[collection()]), Vec::new());
        }

        #[test]
        fn a_collection_that_is_gone_is_refused_and_named() {
            let graph = graph_of_node(node("c1", json!({ "path": { "id": "1" } })));

            let errors = errors_with(&graph, &[]);

            assert_eq!(
                errors,
                vec![GraphError::UnknownCollection {
                    connector_id: "c1".to_owned(),
                    collection_id: Uuid::from_u128(COLLECTION_ID),
                }]
            );
            let message = errors[0].to_string();
            assert!(
                message.contains("c1")
                    && message.contains(&Uuid::from_u128(COLLECTION_ID).to_string()),
                "{message}"
            );
        }

        #[test]
        fn an_operation_that_is_gone_is_refused_and_named() {
            let mut gone = node("c1", json!({}));
            gone.operation = Some(OperationRef {
                collection_id: Uuid::from_u128(COLLECTION_ID),
                operation_id: "deleteAddress".to_owned(),
            });

            let errors = errors_with(&graph_of_node(gone), &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::UnknownOperation {
                    connector_id: "c1".to_owned(),
                    collection_id: Uuid::from_u128(COLLECTION_ID),
                    operation_id: "deleteAddress".to_owned(),
                }]
            );
            assert!(errors[0].to_string().contains("deleteAddress"));
        }

        #[test]
        fn a_node_with_no_operation_reference_misses_its_operation() {
            let mut bare = node("c1", json!({}));
            bare.operation = None;

            let errors = errors_with(&graph_of_node(bare), &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::MissingRequiredField {
                    connector_id: "c1".to_owned(),
                    field: "operation".to_owned(),
                }]
            );
        }

        #[test]
        fn a_missing_required_parameter_is_refused_naming_the_node_and_the_field() {
            let graph = graph_of_node(node("c1", json!({ "query": { "limit": 5 } })));

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::MissingParameter {
                    connector_id: "c1".to_owned(),
                    field: "path.id".to_owned(),
                }]
            );
            let message = errors[0].to_string();
            assert!(
                message.contains("c1") && message.contains("path.id"),
                "{message}"
            );
        }

        #[test]
        fn an_unknown_parameter_is_refused_naming_the_node_and_the_field() {
            let graph = graph_of_node(node(
                "c1",
                json!({ "path": { "id": "1" }, "query": { "sort": "asc" } }),
            ));

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::UnknownParameter {
                    connector_id: "c1".to_owned(),
                    field: "query.sort".to_owned(),
                }]
            );
            let message = errors[0].to_string();
            assert!(
                message.contains("c1") && message.contains("query.sort"),
                "{message}"
            );
        }

        #[test]
        fn the_url_the_method_and_the_security_never_live_in_the_node() {
            for forbidden in ["url", "method", "base_url", "security", "headers_security"] {
                let graph = graph_of_node(node(
                    "c1",
                    json!({ "path": { "id": "1" }, forbidden: "copied" }),
                ));

                let errors = errors_with(&graph, &[collection()]);

                assert_eq!(
                    errors,
                    vec![GraphError::UnknownConfigField {
                        connector_id: "c1".to_owned(),
                        field: forbidden.to_owned(),
                    }],
                    "{forbidden}"
                );
            }
        }

        #[test]
        fn a_group_that_is_a_literal_non_object_is_a_type_mismatch() {
            let graph = graph_of_node(node("c1", json!({ "path": 5 })));

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::FieldTypeMismatch {
                    connector_id: "c1".to_owned(),
                    field: "path".to_owned(),
                    expected: "object",
                    got: "number",
                }]
            );
        }

        #[test]
        fn every_mistake_of_a_node_is_reported_at_once() {
            let graph = graph_of_node(node(
                "c1",
                json!({ "query": { "sort": "asc" }, "headers": { "X-Nope": "1" } }),
            ));

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![
                    GraphError::MissingParameter {
                        connector_id: "c1".to_owned(),
                        field: "path.id".to_owned(),
                    },
                    GraphError::UnknownParameter {
                        connector_id: "c1".to_owned(),
                        field: "query.sort".to_owned(),
                    },
                    GraphError::UnknownParameter {
                        connector_id: "c1".to_owned(),
                        field: "headers.X-Nope".to_owned(),
                    },
                ]
            );
        }

        fn upstream(id: &str) -> super::super::super::graph::PlacedConnector {
            super::super::super::graph::PlacedConnector {
                id: id.to_string(),
                kind: "flow.config".to_string(),
                version: 1,
                credential_id: None,
                config: json!({ "variables": { "id": "x" } })
                    .as_object()
                    .cloned()
                    .unwrap(),
                name: Some(format!("Amont {id}")),
                description: None,
                disabled: false,
                operation: None,
            }
        }

        fn chain(connectors: Vec<super::super::super::graph::PlacedConnector>) -> Graph {
            let mut graph = with_trigger(graph_of(connectors), "c0");
            let ids: Vec<String> = graph.connectors.iter().map(|c| c.id.clone()).collect();
            for pair in ids.windows(2) {
                graph.edges.push(Edge {
                    from: pair[0].clone(),
                    to: pair[1].clone(),
                    branch: None,
                });
            }
            graph
        }

        #[test]
        fn a_nested_reference_to_an_upstream_node_is_accepted() {
            let graph = chain(vec![
                upstream("c0"),
                node(
                    "c1",
                    json!({
                        "path": { "id": "{{ $('Amont c0').item.json.id }}" },
                        "body": { "items": [{ "name": "{{ trigger.name }}" }] }
                    }),
                ),
            ]);

            assert_eq!(errors_with(&graph, &[collection()]), Vec::new());
        }

        #[test]
        fn a_nested_reference_to_an_unknown_node_is_refused_naming_the_nested_field() {
            let graph = chain(vec![
                upstream("c0"),
                node(
                    "c1",
                    json!({ "path": { "id": "{{ $('Fantôme').item.json.id }}" } }),
                ),
            ]);

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::UnknownNodeReference {
                    connector_id: "c1".to_owned(),
                    field: "path.id".to_owned(),
                    node_name: "Fantôme".to_owned(),
                }]
            );
        }

        #[test]
        fn a_nested_reference_to_a_downstream_node_is_refused() {
            let graph = chain(vec![
                upstream("c0"),
                node(
                    "c1",
                    json!({
                        "path": { "id": "1" },
                        "body": { "items": [{ "name": "{{ $('Amont c2').item.json.id }}" }] }
                    }),
                ),
                upstream("c2"),
            ]);

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::NodeNotUpstream {
                    connector_id: "c1".to_owned(),
                    field: "body.items.0.name".to_owned(),
                    node_name: "Amont c2".to_owned(),
                }]
            );
        }

        #[test]
        fn a_nested_loop_reference_outside_a_loop_is_refused() {
            let graph = chain(vec![
                upstream("c0"),
                node(
                    "c1",
                    json!({
                        "path": { "id": "1" },
                        "query": { "limit": ["a", "{{ loop.item }}"] }
                    }),
                ),
            ]);

            let errors = errors_with(&graph, &[collection()]);

            assert_eq!(
                errors,
                vec![GraphError::LoopUsedOutsideLoop {
                    connector_id: "c1".to_owned(),
                    field: "query.limit.1".to_owned(),
                }]
            );
        }

        #[test]
        fn a_nested_expression_that_does_not_parse_is_refused_with_its_field() {
            let graph = chain(vec![
                upstream("c0"),
                node(
                    "c1",
                    json!({
                        "path": { "id": "1" },
                        "headers": { "X-Trace": "{{ trigger.a + }}" }
                    }),
                ),
            ]);

            let errors = errors_with(&graph, &[collection()]);

            assert!(
                errors.iter().any(|error| matches!(
                    error,
                    GraphError::InvalidExpression { connector_id, field, .. }
                        if connector_id == "c1" && field == "headers.X-Trace"
                )),
                "{errors:?}"
            );
        }

        #[test]
        fn another_kind_keeps_not_validating_what_is_nested_in_its_fields() {
            let mut other = upstream("c0");
            other.config = json!({ "variables": { "a": "{{ $('Fantôme').item.json.id }}" } })
                .as_object()
                .cloned()
                .unwrap();

            assert_eq!(errors_with(&chain(vec![other]), &[]), Vec::new());
        }

        #[test]
        fn a_dot_segment_literal_is_refused_when_saving_and_an_empty_one_is_missing() {
            for dots in [".", ".."] {
                let graph = graph_of_node(node("c1", json!({ "path": { "id": dots } })));

                assert_eq!(
                    errors_with(&graph, &[collection()]),
                    vec![GraphError::InvalidParameter {
                        connector_id: "c1".to_owned(),
                        field: "path.id".to_owned(),
                    }],
                    "{dots}"
                );
            }

            let empty = graph_of_node(node("c1", json!({ "path": { "id": "" } })));
            assert_eq!(
                errors_with(&empty, &[collection()]),
                vec![GraphError::MissingParameter {
                    connector_id: "c1".to_owned(),
                    field: "path.id".to_owned(),
                }]
            );
        }

        #[test]
        fn a_header_repeated_under_another_case_is_refused_when_saving() {
            let mut op = operation();
            op.parameters.push(Parameter {
                name: "X-Trace".to_owned(),
                location: ParameterLocation::Header,
                required: false,
                description: String::new(),
                schema: json!({ "type": "string" }),
            });
            let mut col = collection();
            col.operations = vec![op];
            let graph = graph_of_node(node(
                "c1",
                json!({
                    "path": { "id": "1" },
                    "headers": { "X-Trace": "a", "x-trace": "b" }
                }),
            ));

            let errors = errors_with(&graph, &[col]);

            assert_eq!(
                errors,
                vec![GraphError::DuplicateParameter {
                    connector_id: "c1".to_owned(),
                    field: "headers.x-trace".to_owned(),
                }]
            );
        }

        #[test]
        fn another_kind_is_never_checked_against_the_collections() {
            let mut http = condition("c1", "{{ true }}");
            http.operation = Some(OperationRef {
                collection_id: Uuid::from_u128(404),
                operation_id: "ghost".to_owned(),
            });

            assert_eq!(errors_with(&graph_of_node(http), &[]), Vec::new());
        }

        #[test]
        fn the_collection_is_looked_up_by_id_not_by_position() {
            let mut other = collection();
            other.id = Uuid::from_u128(1);
            let graph = graph_of_node(node("c1", json!({ "path": { "id": "1" } })));

            assert_eq!(errors_with(&graph, &[other, collection()]), Vec::new());
        }
    }

    mod smtp_nodes {
        use serde_json::Value;

        use super::*;
        use crate::domain::automation::connector::connector_catalogue;

        fn smtp_node(
            config: Value,
            credential_id: Option<Uuid>,
        ) -> super::super::super::graph::PlacedConnector {
            let Value::Object(config) = config else {
                panic!("a config is an object");
            };
            super::super::super::graph::PlacedConnector {
                id: "c1".to_string(),
                kind: "smtp.send".to_string(),
                version: 1,
                credential_id,
                config,
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }
        }

        fn mail() -> Value {
            json!({
                "to": "claire@fontaine-immo.fr",
                "subject": "Votre dossier",
                "body": "Bonjour",
            })
        }

        fn errors_of(
            node: super::super::super::graph::PlacedConnector,
            saved: &[Credential],
        ) -> Vec<GraphError> {
            let graph = with_trigger(graph_of(vec![node]), "c1");
            match validate_graph(
                &graph,
                &connector_catalogue(),
                saved,
                &EventCatalogue::new(),
                &[],
            ) {
                Ok(()) => Vec::new(),
                Err(errors) => errors,
            }
        }

        fn merged(base: Value, extra: Value) -> Value {
            let (Value::Object(mut base), Value::Object(extra)) = (base, extra) else {
                panic!("objects");
            };
            base.extend(extra);
            Value::Object(base)
        }

        #[test]
        fn a_mail_with_an_smtp_credential_is_valid() {
            let id = Uuid::from_u128(7);

            let errors = errors_of(smtp_node(mail(), Some(id)), &[credential(id, "smtp")]);

            assert_eq!(errors, Vec::new());
        }

        #[test]
        fn only_an_smtp_credential_is_accepted_and_one_is_required() {
            let id = Uuid::from_u128(7);

            let wrong = errors_of(
                smtp_node(mail(), Some(id)),
                &[credential(id, "bearer_token")],
            );
            let missing = errors_of(smtp_node(mail(), None), &[]);

            assert_eq!(
                wrong,
                vec![GraphError::CredentialSchemeNotAccepted {
                    connector_id: "c1".to_string(),
                    credential_id: id,
                    scheme: "bearer_token".to_string(),
                }]
            );
            assert_eq!(
                missing,
                vec![GraphError::MissingCredential {
                    connector_id: "c1".to_string()
                }]
            );
        }

        #[test]
        fn a_line_break_in_a_literal_address_or_subject_is_refused_at_save() {
            let id = Uuid::from_u128(7);
            let saved = [credential(id, "smtp")];
            for field in ["to", "cc", "subject"] {
                let config = merged(mail(), json!({ field: "x\r\nBcc: spy@evil.fr" }));

                let errors = errors_of(smtp_node(config, Some(id)), &saved);

                assert_eq!(
                    errors,
                    vec![GraphError::HeaderInjection {
                        connector_id: "c1".to_string(),
                        field: field.to_string(),
                    }],
                    "{field}"
                );
            }
        }

        #[test]
        fn more_than_fifty_literal_recipients_are_refused_at_save() {
            let id = Uuid::from_u128(7);
            let saved = [credential(id, "smtp")];
            let list = |prefix: &str, count: usize| {
                (0..count)
                    .map(|n| format!("{prefix}{n}@x.fr"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let within = merged(mail(), json!({ "to": list("a", 30), "cc": list("b", 20) }));
            let over = merged(mail(), json!({ "to": list("a", 30), "cc": list("b", 21) }));

            assert_eq!(errors_of(smtp_node(within, Some(id)), &saved), Vec::new());
            assert_eq!(
                errors_of(smtp_node(over, Some(id)), &saved),
                vec![GraphError::TooManyRecipients {
                    connector_id: "c1".to_string()
                }]
            );
        }

        #[test]
        fn a_value_built_by_an_expression_is_left_to_the_run() {
            let id = Uuid::from_u128(7);
            let config = merged(
                mail(),
                json!({ "to": "{{ trigger.email }}", "subject": "{{ trigger.title }}" }),
            );

            let errors = errors_of(smtp_node(config, Some(id)), &[credential(id, "smtp")]);

            assert_eq!(errors, Vec::new());
        }
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The graph one [`super::WorkflowVersion`] freezes: connectors placed on the
/// canvas and the edges wiring them together.
///
/// Stored as JSONB, whole, never normalized into tables — see the migration's
/// comment for why. `Serialize`/`Deserialize` are this type's on-disk and
/// wire format both, so its shape is locked by an exact-JSON test the same
/// way `events::EventEnvelope` locks its own (`libs/events/src/envelope.rs`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub connectors: Vec<PlacedConnector>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub triggers: Vec<PlacedTrigger>,
}

impl Graph {
    pub fn referenced_collection_ids(&self) -> Vec<Uuid> {
        let mut ids: Vec<Uuid> = Vec::new();
        for operation in self
            .connectors
            .iter()
            .filter_map(|connector| connector.operation.as_ref())
        {
            if !ids.contains(&operation.collection_id) {
                ids.push(operation.collection_id);
            }
        }
        ids
    }
}

/// One connector instance on the canvas.
///
/// `id` is local to this graph (`"c1"`), assigned by the editor and never
/// reassigned: it is what edges connect, so renaming a connector's label must
/// never change it. Expressions reach a connector by its `name`, not its `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedConnector {
    pub id: String,
    pub kind: String,
    pub version: u16,
    pub credential_id: Option<Uuid>,
    pub config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<OperationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRef {
    pub collection_id: Uuid,
    pub operation_id: String,
}

/// One connection between two connectors, by their graph-local `id`s.
///
/// A single shape for every kind of edge on purpose: a plain sequence edge,
/// a `flow.condition` branch and a `flow.loop` branch all carry the same
/// `from`/`to`, differing only in `branch`. Three edge types would have meant
/// three things for the engine (#200) to execute and three things for the
/// editor to draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub branch: Option<Branch>,
}

/// Which branch of a `flow.condition` or `flow.loop` connector this edge
/// leaves from. `None` on every other edge — see
/// [`super::validation::GraphError::InvalidBranch`] for the rule tying a
/// branch to the connector kind it is legal on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Branch {
    Then,
    Else,
    Each,
    After,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedTrigger {
    pub id: String,
    pub kind: TriggerKind,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerKind {
    Manual,
    Events(Vec<String>),
    Webhook {
        method: WebhookMethod,
        path: String,
        #[serde(default)]
        credential_id: Option<Uuid>,
        #[serde(default)]
        contract: Vec<ContractKey>,
        #[serde(default)]
        on_invalid: OnInvalid,
    },
    Schedule {
        frequency: Frequency,
        time: String,
        #[serde(default)]
        weekday: Option<Weekday>,
        #[serde(default)]
        timezone: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WebhookMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl WebhookMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractKind {
    Text,
    Number,
    Boolean,
    Object,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractKey {
    pub name: String,
    pub kind: ContractKind,
    pub required: bool,
    #[serde(default)]
    pub example: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnInvalid {
    #[default]
    Reject,
    Accept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};
    use uuid::Uuid;

    use super::*;

    fn sample_graph() -> Graph {
        let mut config = serde_json::Map::new();
        config.insert(
            "kind".to_string(),
            json!("{{ trigger.payload.customer_kind }}"),
        );

        Graph {
            connectors: vec![
                PlacedConnector {
                    id: "c1".to_string(),
                    kind: "flow.condition".to_string(),
                    version: 1,
                    credential_id: None,
                    config: {
                        let mut m = serde_json::Map::new();
                        m.insert("predicate".to_string(), json!("{{ true }}"));
                        m
                    },
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                },
                PlacedConnector {
                    id: "c2".to_string(),
                    kind: "odoo.create_partner".to_string(),
                    version: 1,
                    credential_id: Some(Uuid::from_u128(1)),
                    config,
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                },
            ],
            edges: vec![
                Edge {
                    from: "c1".to_string(),
                    to: "c2".to_string(),
                    branch: Some(Branch::Then),
                },
                Edge {
                    from: "c1".to_string(),
                    to: "c1".to_string(),
                    branch: None,
                },
            ],
            triggers: vec![
                PlacedTrigger {
                    id: "t1".to_string(),
                    kind: TriggerKind::Manual,
                    name: None,
                    description: None,
                },
                PlacedTrigger {
                    id: "t2".to_string(),
                    kind: TriggerKind::Events(vec!["quote.accepted".to_string()]),
                    name: None,
                    description: None,
                },
            ],
        }
    }

    /// The graph's JSON shape is a contract: the editor reads and writes it
    /// whole, and `automation.workflow_version.graph` stores exactly this.
    /// Locking the exact shape catches an accidental rename the same way
    /// `envelope_serializes_to_a_stable_wire_shape` does for events.
    #[test]
    fn graph_serializes_to_a_stable_wire_shape() {
        let json = to_value(sample_graph()).expect("graph serializes");

        assert_eq!(
            json,
            json!({
                "connectors": [
                    {
                        "id": "c1",
                        "kind": "flow.condition",
                        "version": 1,
                        "credential_id": null,
                        "config": { "predicate": "{{ true }}" },
                        "name": null,
                        "description": null,
                        "disabled": false
                    },
                    {
                        "id": "c2",
                        "kind": "odoo.create_partner",
                        "version": 1,
                        "credential_id": "00000000-0000-0000-0000-000000000001",
                        "config": { "kind": "{{ trigger.payload.customer_kind }}" },
                        "name": null,
                        "description": null,
                        "disabled": false
                    }
                ],
                "edges": [
                    { "from": "c1", "to": "c2", "branch": "Then" },
                    { "from": "c1", "to": "c1", "branch": null }
                ],
                "triggers": [
                    { "id": "t1", "kind": "Manual", "name": null, "description": null },
                    {
                        "id": "t2",
                        "kind": { "Events": ["quote.accepted"] },
                        "name": null,
                        "description": null
                    }
                ]
            })
        );
    }

    #[test]
    fn graph_round_trips_through_serde() {
        let graph = sample_graph();

        let encoded = serde_json::to_string(&graph).expect("graph serializes");
        let decoded: Graph = serde_json::from_str(&encoded).expect("graph deserializes");

        assert_eq!(decoded, graph);
    }

    #[test]
    fn an_empty_graph_is_the_default() {
        assert_eq!(
            Graph::default(),
            Graph {
                connectors: Vec::new(),
                edges: Vec::new(),
                triggers: Vec::new(),
            }
        );
    }

    #[test]
    fn a_graph_stored_before_triggers_existed_still_deserializes() {
        let json = json!({
            "connectors": [],
            "edges": []
        });

        let graph: Graph =
            serde_json::from_value(json).expect("a graph with no triggers key deserializes");

        assert_eq!(graph, Graph::default());
    }

    #[test]
    fn a_graph_stored_before_node_labels_existed_reads_back_with_the_defaults() {
        let stored = json!({
            "connectors": [{
                "id": "c1",
                "kind": "flow.condition",
                "version": 1,
                "credential_id": null,
                "config": { "predicate": "{{ true }}" }
            }],
            "edges": [{ "from": "t1", "to": "c1", "branch": null }],
            "triggers": [{ "id": "t1", "kind": "Manual" }]
        });

        let graph: Graph = serde_json::from_value(stored).expect("an old graph still reads");

        assert_eq!(graph.connectors[0].name, None);
        assert_eq!(graph.connectors[0].description, None);
        assert!(!graph.connectors[0].disabled);
        assert_eq!(graph.triggers[0].name, None);
        assert_eq!(graph.triggers[0].description, None);
    }

    #[test]
    fn a_graph_stored_before_operation_refs_existed_reads_back_without_one() {
        let stored = json!({
            "connectors": [{
                "id": "c1",
                "kind": "http.request",
                "version": 1,
                "credential_id": null,
                "config": { "method": "GET", "url": "https://example.com" },
                "name": null,
                "description": null,
                "disabled": false
            }],
            "edges": [],
            "triggers": []
        });

        let graph: Graph = serde_json::from_value(stored).expect("an old graph still reads");

        assert_eq!(graph.connectors[0].operation, None);
    }

    #[test]
    fn a_node_without_an_operation_serializes_exactly_as_before() {
        let graph = sample_graph();

        let json = to_value(&graph).expect("graph serializes");

        assert!(json["connectors"][0].get("operation").is_none());
    }

    #[test]
    fn an_operation_ref_carries_the_collection_and_the_operation_and_nothing_else() {
        let collection_id = Uuid::from_u128(42);
        let mut graph = sample_graph();
        graph.connectors[1].kind = "collection.operation".to_string();
        graph.connectors[1].operation = Some(OperationRef {
            collection_id,
            operation_id: "getAddress".to_string(),
        });

        let json = to_value(&graph).expect("graph serializes");

        assert_eq!(
            json["connectors"][1]["operation"],
            json!({
                "collection_id": "00000000-0000-0000-0000-00000000002a",
                "operation_id": "getAddress"
            })
        );
        let decoded: Graph = serde_json::from_value(json).expect("graph deserializes");
        assert_eq!(decoded, graph);
    }

    #[test]
    fn the_collections_a_graph_names_are_listed_once_each() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let mut graph = sample_graph();
        graph.connectors[0].operation = Some(OperationRef {
            collection_id: first,
            operation_id: "a".to_string(),
        });
        graph.connectors[1].operation = Some(OperationRef {
            collection_id: first,
            operation_id: "b".to_string(),
        });
        graph.connectors.push(PlacedConnector {
            id: "c3".to_string(),
            kind: "collection.operation".to_string(),
            version: 1,
            credential_id: None,
            config: serde_json::Map::new(),
            name: None,
            description: None,
            disabled: false,
            operation: Some(OperationRef {
                collection_id: second,
                operation_id: "c".to_string(),
            }),
        });

        assert_eq!(graph.referenced_collection_ids(), vec![first, second]);
        assert!(Graph::default().referenced_collection_ids().is_empty());
    }

    #[test]
    fn node_labels_and_the_disabled_flag_round_trip() {
        let mut graph = sample_graph();
        graph.connectors[0].name = Some("Is it paid".to_string());
        graph.connectors[0].description = Some("Stops unpaid quotes".to_string());
        graph.connectors[0].disabled = true;
        graph.triggers[1].name = Some("Quote accepted".to_string());
        graph.triggers[1].description = Some("Fires on acceptance".to_string());

        let encoded = serde_json::to_string(&graph).expect("graph serializes");
        let decoded: Graph = serde_json::from_str(&encoded).expect("graph deserializes");

        assert_eq!(decoded, graph);
        assert!(decoded.connectors[0].disabled);
    }

    fn webhook_trigger() -> TriggerKind {
        TriggerKind::Webhook {
            method: WebhookMethod::Post,
            path: "paiement-recu".to_string(),
            credential_id: Some(Uuid::from_u128(7)),
            contract: vec![
                ContractKey {
                    name: "montant".to_string(),
                    kind: ContractKind::Number,
                    required: true,
                    example: "2340".to_string(),
                },
                ContractKey {
                    name: "client".to_string(),
                    kind: ContractKind::Object,
                    required: false,
                    example: String::new(),
                },
            ],
            on_invalid: OnInvalid::Accept,
        }
    }

    fn schedule_trigger() -> TriggerKind {
        TriggerKind::Schedule {
            frequency: Frequency::Weekly,
            time: "09:00".to_string(),
            weekday: Some(Weekday::Monday),
            timezone: Some("Europe/Paris".to_string()),
        }
    }

    #[test]
    fn a_webhook_trigger_serializes_to_a_stable_wire_shape() {
        let json = to_value(webhook_trigger()).expect("trigger serializes");

        assert_eq!(
            json,
            json!({
                "Webhook": {
                    "method": "POST",
                    "path": "paiement-recu",
                    "credential_id": "00000000-0000-0000-0000-000000000007",
                    "contract": [
                        { "name": "montant", "kind": "Number", "required": true, "example": "2340" },
                        { "name": "client", "kind": "Object", "required": false, "example": "" }
                    ],
                    "on_invalid": "Accept"
                }
            })
        );
    }

    #[test]
    fn a_schedule_trigger_serializes_to_a_stable_wire_shape() {
        let json = to_value(schedule_trigger()).expect("trigger serializes");

        assert_eq!(
            json,
            json!({
                "Schedule": {
                    "frequency": "Weekly",
                    "time": "09:00",
                    "weekday": "Monday",
                    "timezone": "Europe/Paris"
                }
            })
        );
    }

    #[test]
    fn the_new_trigger_kinds_round_trip_through_serde() {
        for kind in [webhook_trigger(), schedule_trigger()] {
            let encoded = serde_json::to_string(&kind).expect("trigger serializes");
            let decoded: TriggerKind = serde_json::from_str(&encoded).expect("trigger reads back");

            assert_eq!(decoded, kind);
        }
    }

    #[test]
    fn a_webhook_with_only_its_required_fields_reads_back_with_the_defaults() {
        let stored = json!({ "Webhook": { "method": "GET", "path": "ping" } });

        let kind: TriggerKind = serde_json::from_value(stored).expect("defaults apply");

        assert_eq!(
            kind,
            TriggerKind::Webhook {
                method: WebhookMethod::Get,
                path: "ping".to_string(),
                credential_id: None,
                contract: Vec::new(),
                on_invalid: OnInvalid::Reject,
            }
        );
    }

    #[test]
    fn a_daily_schedule_without_weekday_or_timezone_reads_back_with_none() {
        let stored = json!({ "Schedule": { "frequency": "Daily", "time": "09:00" } });

        let kind: TriggerKind = serde_json::from_value(stored).expect("defaults apply");

        assert_eq!(
            kind,
            TriggerKind::Schedule {
                frequency: Frequency::Daily,
                time: "09:00".to_string(),
                weekday: None,
                timezone: None,
            }
        );
    }

    #[test]
    fn a_contract_key_without_an_example_reads_back_with_an_empty_one() {
        let stored = json!({ "name": "a", "kind": "Text", "required": true });

        let key: ContractKey = serde_json::from_value(stored).expect("example defaults");

        assert_eq!(key.example, "");
    }

    #[test]
    fn an_unknown_method_is_refused() {
        let stored = json!({ "Webhook": { "method": "TRACE", "path": "ping" } });

        assert!(serde_json::from_value::<TriggerKind>(stored).is_err());
    }

    #[test]
    fn a_graph_holding_every_trigger_kind_round_trips() {
        let mut graph = sample_graph();
        graph.triggers.push(PlacedTrigger {
            id: "t3".to_string(),
            kind: webhook_trigger(),
            name: None,
            description: None,
        });
        graph.triggers.push(PlacedTrigger {
            id: "t4".to_string(),
            kind: schedule_trigger(),
            name: None,
            description: None,
        });

        let encoded = serde_json::to_string(&graph).expect("graph serializes");
        let decoded: Graph = serde_json::from_str(&encoded).expect("graph deserializes");

        assert_eq!(decoded, graph);
    }

    #[test]
    fn the_methods_spell_themselves_the_way_the_wire_does() {
        for method in [
            WebhookMethod::Get,
            WebhookMethod::Post,
            WebhookMethod::Put,
            WebhookMethod::Patch,
            WebhookMethod::Delete,
        ] {
            assert_eq!(to_value(method).unwrap(), json!(method.as_str()));
            assert_eq!(WebhookMethod::parse(method.as_str()), Some(method));
        }
        assert_eq!(WebhookMethod::parse("post"), None);
        assert_eq!(WebhookMethod::parse("TRACE"), None);
    }

    #[test]
    fn a_manual_trigger_and_an_events_trigger_are_distinguished_by_type() {
        assert_ne!(
            TriggerKind::Manual,
            TriggerKind::Events(Vec::new()),
            "manual and events-with-nothing-selected must never compare equal"
        );
    }
}

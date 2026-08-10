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
}

/// One connector instance on the canvas.
///
/// `id` is local to this graph (`"c1"`), assigned by the editor and never
/// reassigned: it is what expressions reference
/// (`{{ connectors.c1.output.… }}`) and what edges connect, so renaming a
/// connector's label must never change it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedConnector {
    pub id: String,
    pub kind: String,
    pub version: u16,
    pub credential_id: Option<Uuid>,
    pub config: serde_json::Map<String, serde_json::Value>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Branch {
    Then,
    Else,
    Each,
    After,
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
                },
                PlacedConnector {
                    id: "c2".to_string(),
                    kind: "odoo.create_partner".to_string(),
                    version: 1,
                    credential_id: Some(Uuid::from_u128(1)),
                    config,
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
                        "config": { "predicate": "{{ true }}" }
                    },
                    {
                        "id": "c2",
                        "kind": "odoo.create_partner",
                        "version": 1,
                        "credential_id": "00000000-0000-0000-0000-000000000001",
                        "config": { "kind": "{{ trigger.payload.customer_kind }}" }
                    }
                ],
                "edges": [
                    { "from": "c1", "to": "c2", "branch": "Then" },
                    { "from": "c1", "to": "c1", "branch": null }
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
            }
        );
    }
}

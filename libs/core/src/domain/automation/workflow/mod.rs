//! The workflow graph: a named, versioned [`Graph`] of connectors an
//! organization triggers from a domain event (#195). This module owns the
//! model, its validation at save time (#199), and the commands that create
//! and edit it. Execution (#200) is somebody else's module.

mod commands;
mod graph;
mod validation;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde::{Deserialize, Deserializer, Serialize, de};
use uuid::Uuid;

pub use commands::{CreateWorkflowCommand, SaveWorkflowVersionCommand, UpdateWorkflowCommand};
pub use graph::{Branch, Edge, Graph, PlacedConnector, PlacedTrigger, TriggerKind};
pub use validation::{GraphError, validate_graph};

/// A workflow: the organization-facing identity (name, whether it is
/// enabled) plus a pointer at whichever [`WorkflowVersion`] currently runs.
///
/// The graph itself never lives here — see [`WorkflowVersion`] for why.
#[derive(Debug, Clone, PartialEq)]
pub struct Workflow {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    /// `None` until the first version is saved. Set, and only ever moved
    /// forward, by [`super::ports::WorkflowRepository::insert_version`].
    pub current_version_id: Option<Uuid>,
    pub layout: Option<WorkflowLayout>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One immutable snapshot of a workflow's graph.
///
/// Editing a workflow **inserts** a version and moves
/// [`Workflow::current_version_id`]; it never rewrites one. A version a run
/// is executing (#200) can therefore never change under it, and the engine
/// never needs to copy the graph into the run — it just keeps reading this
/// row.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowVersion {
    pub id: Uuid,
    pub workflow_id: Uuid,
    /// 1, 2, 3, … within one workflow. Assigned by the repository, which
    /// serializes concurrent saves so two editors can never mint the same
    /// number.
    pub version: i32,
    pub graph: Graph,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Just enough to name a workflow in an error — what
/// [`super::ports::WorkflowRepository::workflows_referencing_credential`]
/// returns for the credential-deletion guard
/// (`application::automation::credential::delete_credential`). Counting is
/// `.len()` on the result; naming is `.name` on each entry — one query
/// answers both, rather than a count query and a naming query disagreeing
/// under concurrent writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowReference {
    pub id: Uuid,
    pub name: String,
}

pub const MAX_LAYOUT_ENTRIES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct WorkflowLayout(BTreeMap<String, NodePosition>);

impl WorkflowLayout {
    pub fn position_of(&self, connector_id: &str) -> Option<NodePosition> {
        self.0.get(connector_id).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<BTreeMap<String, NodePosition>> for WorkflowLayout {
    fn from(positions: BTreeMap<String, NodePosition>) -> Self {
        Self(positions)
    }
}

impl From<WorkflowLayout> for BTreeMap<String, NodePosition> {
    fn from(layout: WorkflowLayout) -> Self {
        layout.0
    }
}

impl<'de> Deserialize<'de> for WorkflowLayout {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let positions = BTreeMap::<String, NodePosition>::deserialize(deserializer)?;

        if positions.len() > MAX_LAYOUT_ENTRIES {
            return Err(de::Error::custom(format!(
                "a layout holds at most {MAX_LAYOUT_ENTRIES} entries, got {}",
                positions.len()
            )));
        }

        Ok(Self(positions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn a_layout_is_a_map_of_connector_id_to_a_point() {
        let layout: WorkflowLayout =
            serde_json::from_value(json!({ "c1": { "x": 10.5, "y": -20.0 } })).expect("parses");

        assert_eq!(
            layout.position_of("c1"),
            Some(NodePosition { x: 10.5, y: -20.0 })
        );
        assert_eq!(layout.position_of("c2"), None);
        assert_eq!(
            to_value(&layout).expect("serializes"),
            json!({ "c1": { "x": 10.5, "y": -20.0 } })
        );
    }

    #[test]
    fn a_layout_entry_missing_a_coordinate_is_refused() {
        let parsed: Result<WorkflowLayout, _> =
            serde_json::from_value(json!({ "c1": { "x": 1.0 } }));

        assert!(parsed.is_err());
    }

    #[test]
    fn a_layout_round_trips_through_a_plain_map_of_positions() {
        let mut positions = BTreeMap::new();
        positions.insert("c1".to_string(), NodePosition { x: 1.0, y: 2.0 });
        positions.insert("c2".to_string(), NodePosition { x: -3.5, y: 4.5 });

        let layout = WorkflowLayout::from(positions.clone());

        assert_eq!(
            layout.position_of("c1"),
            Some(NodePosition { x: 1.0, y: 2.0 })
        );
        assert_eq!(BTreeMap::<String, NodePosition>::from(layout), positions);
    }

    #[test]
    fn a_layout_holding_more_entries_than_the_cap_is_refused() {
        let mut oversized = serde_json::Map::new();
        for index in 0..=MAX_LAYOUT_ENTRIES {
            oversized.insert(format!("c{index}"), json!({ "x": 0.0, "y": 0.0 }));
        }

        let parsed: Result<WorkflowLayout, _> =
            serde_json::from_value(serde_json::Value::Object(oversized));

        assert!(parsed.is_err());
    }
}

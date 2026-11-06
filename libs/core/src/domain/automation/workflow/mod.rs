//! The workflow graph: a named, versioned [`Graph`] of connectors an
//! organization triggers from a domain event (#195). This module owns the
//! model, its validation at save time (#199), and the commands that create
//! and edit it. Execution (#200) is somebody else's module.

mod archive;
mod commands;
mod graph;
mod tags;
mod text;
mod validation;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde::{Deserialize, Deserializer, Serialize, de};
use uuid::Uuid;

pub use archive::{ArchiveScope, WorkflowRefusal};
pub use commands::{
    CreateWorkflowCommand, SaveWorkflowVersionCommand, UpdateWorkflowCommand, WorkflowInitialState,
};
pub use graph::{Branch, Edge, Graph, OperationRef, PlacedConnector, PlacedTrigger, TriggerKind};
pub use tags::{MAX_RAW_TAGS, MAX_TAG_LENGTH, MAX_TAGS, WorkflowTags, WorkflowTagsError};
pub use text::{
    MAX_WORKFLOW_DESCRIPTION_LENGTH, MAX_WORKFLOW_NAME_LENGTH, WorkflowDescription,
    WorkflowDescriptionError, WorkflowName, WorkflowNameError,
};
pub use validation::{
    GraphError, MAX_NODE_DESCRIPTION_LENGTH, MAX_NODE_NAME_LENGTH, NodeRef, validate_graph,
};

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
    pub folder_id: Option<Uuid>,
    pub tags: WorkflowTags,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    pub fn ensure_not_archived(&self) -> Result<(), WorkflowRefusal> {
        if self.is_archived() {
            return Err(WorkflowRefusal::Archived);
        }
        Ok(())
    }

    pub fn apply(
        self,
        command: UpdateWorkflowCommand,
        now: DateTime<Utc>,
    ) -> Result<Self, WorkflowRefusal> {
        if command.archived == Some(true) && command.enabled == Some(true) {
            return Err(WorkflowRefusal::ConflictingFlags);
        }

        let renamed = command
            .name
            .as_ref()
            .is_some_and(|name| name.as_str() != self.name);
        let redescribed = command.description.as_ref().is_some_and(|description| {
            description.as_ref().map(WorkflowDescription::as_str) != self.description.as_deref()
        });
        let name = command.name.map_or(self.name, WorkflowName::into_string);
        let description = command.description.map_or(self.description, |replacement| {
            replacement.map(WorkflowDescription::into_string)
        });
        let archived_at = match command.archived {
            Some(true) => self.archived_at.or(Some(now)),
            Some(false) => None,
            None => self.archived_at,
        };
        let enabled = match (archived_at, command.enabled) {
            (Some(_), Some(true)) => return Err(WorkflowRefusal::Archived),
            (Some(_), _) => false,
            (None, requested) => requested.unwrap_or(self.enabled),
        };

        Ok(Self {
            name,
            description,
            enabled,
            folder_id: command.folder_id.unwrap_or(self.folder_id),
            tags: command.tags.unwrap_or(self.tags),
            archived_at,
            updated_at: if renamed || redescribed {
                now
            } else {
                self.updated_at
            },
            ..self
        })
    }
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
    use chrono::Duration;
    use common::generate_uuid_v7;

    use super::*;
    use serde_json::{json, to_value};

    fn workflow() -> Workflow {
        let created_at = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid instant");
        Workflow {
            id: generate_uuid_v7(),
            org_id: OrganizationId(generate_uuid_v7()),
            name: "Quote accepted".to_owned(),
            description: Some("Creates the customer".to_owned()),
            enabled: true,
            current_version_id: None,
            layout: None,
            folder_id: None,
            tags: WorkflowTags::default(),
            archived_at: None,
            created_at,
            updated_at: created_at,
        }
    }

    fn untouched(workflow: &Workflow) -> UpdateWorkflowCommand {
        UpdateWorkflowCommand {
            org_id: workflow.org_id,
            id: workflow.id,
            name: None,
            description: None,
            enabled: None,
            folder_id: None,
            tags: None,
            archived: None,
        }
    }

    fn later(workflow: &Workflow) -> DateTime<Utc> {
        workflow.updated_at + Duration::hours(1)
    }

    #[test]
    fn renaming_moves_updated_at() {
        let before = workflow();
        let now = later(&before);

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    name: Some(WorkflowName::try_from("Renamed".to_owned()).expect("valid name")),
                    ..untouched(&before)
                },
                now,
            )
            .expect("applies");

        assert_eq!(after.name, "Renamed");
        assert_eq!(after.updated_at, now);
    }

    #[test]
    fn changing_the_description_moves_updated_at() {
        let before = workflow();
        let now = later(&before);

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    description: Some(None),
                    ..untouched(&before)
                },
                now,
            )
            .expect("applies");

        assert_eq!(after.description, None);
        assert_eq!(after.updated_at, now);
    }

    #[test]
    fn repeating_the_current_name_and_description_leaves_updated_at_alone() {
        let before = workflow();

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    name: Some(WorkflowName::try_from(before.name.clone()).expect("valid name")),
                    description: Some(
                        before
                            .description
                            .clone()
                            .map(|text| WorkflowDescription::try_from(text).expect("valid text")),
                    ),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect("applies");

        assert_eq!(after.updated_at, before.updated_at);
    }

    #[test]
    fn enabled_folder_tags_and_archive_leave_updated_at_alone() {
        let before = workflow();
        let folder_id = generate_uuid_v7();
        let tags = WorkflowTags::try_from(vec!["crm".to_owned()]).expect("valid tags");

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    enabled: Some(false),
                    folder_id: Some(Some(folder_id)),
                    tags: Some(tags.clone()),
                    archived: Some(true),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect("applies");

        assert!(!after.enabled);
        assert_eq!(after.folder_id, Some(folder_id));
        assert_eq!(after.tags, tags);
        assert!(after.archived_at.is_some());
        assert_eq!(after.updated_at, before.updated_at);
    }

    #[test]
    fn a_null_folder_sends_the_workflow_back_to_the_root() {
        let before = Workflow {
            folder_id: Some(generate_uuid_v7()),
            ..workflow()
        };

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    folder_id: Some(None),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect("applies");

        assert_eq!(after.folder_id, None);
    }

    #[test]
    fn archiving_stamps_the_instant_and_disables_the_workflow() {
        let before = workflow();
        let now = later(&before);

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(true),
                    ..untouched(&before)
                },
                now,
            )
            .expect("applies");

        assert_eq!(after.archived_at, Some(now));
        assert!(!after.enabled);
        assert!(after.is_archived());
    }

    #[test]
    fn archiving_twice_keeps_the_first_instant() {
        let before = workflow();
        let first = later(&before);
        let archived = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(true),
                    ..untouched(&before)
                },
                first,
            )
            .expect("applies");

        let again = archived
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(true),
                    ..untouched(&archived)
                },
                first + Duration::hours(1),
            )
            .expect("applies");

        assert_eq!(again.archived_at, Some(first));
    }

    #[test]
    fn unarchiving_clears_the_instant_and_leaves_the_workflow_disabled() {
        let before = Workflow {
            enabled: false,
            archived_at: Some(later(&workflow())),
            ..workflow()
        };

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(false),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect("applies");

        assert_eq!(after.archived_at, None);
        assert!(!after.enabled);
    }

    #[test]
    fn unarchiving_and_enabling_in_one_call_enables_the_workflow() {
        let before = Workflow {
            enabled: false,
            archived_at: Some(later(&workflow())),
            ..workflow()
        };

        let after = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(false),
                    enabled: Some(true),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect("applies");

        assert_eq!(after.archived_at, None);
        assert!(after.enabled);
    }

    #[test]
    fn enabling_an_archived_workflow_is_refused() {
        let before = Workflow {
            enabled: false,
            archived_at: Some(later(&workflow())),
            ..workflow()
        };

        let refused = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    enabled: Some(true),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect_err("an archived workflow cannot be enabled");

        assert_eq!(refused, WorkflowRefusal::Archived);
    }

    #[test]
    fn archiving_and_enabling_in_one_call_is_a_contradiction() {
        let before = workflow();

        let refused = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(true),
                    enabled: Some(true),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect_err("contradictory");

        assert_eq!(refused, WorkflowRefusal::ConflictingFlags);
    }

    #[test]
    fn archiving_an_already_archived_workflow_while_enabling_is_still_a_contradiction() {
        let before = Workflow {
            enabled: false,
            archived_at: Some(later(&workflow())),
            ..workflow()
        };

        let refused = before
            .clone()
            .apply(
                UpdateWorkflowCommand {
                    archived: Some(true),
                    enabled: Some(true),
                    ..untouched(&before)
                },
                later(&before),
            )
            .expect_err("contradictory");

        assert_eq!(refused, WorkflowRefusal::ConflictingFlags);
    }

    #[test]
    fn a_name_is_trimmed_and_held_between_one_and_120_characters() {
        assert_eq!(
            WorkflowName::try_from("  Quote accepted ".to_string())
                .unwrap()
                .as_str(),
            "Quote accepted"
        );
        assert_eq!(
            WorkflowName::try_from("   ".to_string()),
            Err(WorkflowNameError::Empty)
        );
        assert_eq!(
            WorkflowName::try_from(String::new()),
            Err(WorkflowNameError::Empty)
        );
        assert!(WorkflowName::try_from(format!(" {} ", "é".repeat(120))).is_ok());
        assert_eq!(
            WorkflowName::try_from("é".repeat(121)),
            Err(WorkflowNameError::TooLong)
        );
    }

    #[test]
    fn a_description_holds_at_most_2000_characters_and_may_be_empty() {
        assert!(WorkflowDescription::try_from("é".repeat(2000)).is_ok());
        assert!(WorkflowDescription::try_from(String::new()).is_ok());
        assert_eq!(
            WorkflowDescription::try_from("é".repeat(2001)),
            Err(WorkflowDescriptionError::TooLong)
        );
    }

    #[test]
    fn the_text_errors_never_echo_the_offending_text() {
        let huge = "x".repeat(10_000);

        let name = WorkflowName::try_from(huge.clone())
            .unwrap_err()
            .to_string();
        let description = WorkflowDescription::try_from(huge).unwrap_err().to_string();

        assert!(!name.contains("xxxx") && !description.contains("xxxx"));
    }

    #[test]
    fn only_an_archived_workflow_fails_the_archive_guard() {
        let active = workflow();
        let archived = Workflow {
            enabled: false,
            archived_at: Some(later(&active)),
            ..active.clone()
        };

        assert_eq!(active.ensure_not_archived(), Ok(()));
        assert_eq!(
            archived.ensure_not_archived(),
            Err(WorkflowRefusal::Archived)
        );
    }

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

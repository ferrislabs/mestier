//! The workflow graph: a named, versioned [`Graph`] of connectors an
//! organization triggers from a domain event (#195). This module owns the
//! model, its validation at save time (#199), and the commands that create
//! and edit it. Execution (#200) is somebody else's module.

mod commands;
mod graph;
mod validation;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use uuid::Uuid;

pub use commands::{CreateWorkflowCommand, SaveWorkflowVersionCommand, UpdateWorkflowCommand};
pub use graph::{Branch, Edge, Graph, PlacedConnector};
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

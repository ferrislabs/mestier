use common::OrganizationId;
use uuid::Uuid;

use super::WorkflowLayout;
use super::graph::Graph;
use super::tags::WorkflowTags;

#[derive(Debug, Clone, PartialEq)]
pub struct CreateWorkflowCommand {
    pub org_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowInitialState {
    pub enabled: bool,
    pub folder_id: Option<Uuid>,
    pub tags: WorkflowTags,
}

impl Default for WorkflowInitialState {
    fn default() -> Self {
        Self {
            enabled: true,
            folder_id: None,
            tags: WorkflowTags::default(),
        }
    }
}

/// Every field optional and applied only when present. `description` is
/// itself nullable, so it needs the double option — `None` leaves it
/// unchanged, `Some(None)` clears it, `Some(Some(text))` sets it. Same
/// convention as `domain::task::commands::PatchTaskCommand`.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateWorkflowCommand {
    pub org_id: OrganizationId,
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub enabled: Option<bool>,
    pub folder_id: Option<Option<Uuid>>,
    pub tags: Option<WorkflowTags>,
    pub archived: Option<bool>,
}

/// Saves an edit: the graph is validated, then persisted as a new immutable
/// version, moving `current_version_id` to it — see
/// `application::automation::workflow::MestierUseCase::save_workflow_version`.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveWorkflowVersionCommand {
    pub org_id: OrganizationId,
    pub workflow_id: Uuid,
    pub graph: Graph,
    pub layout: Option<WorkflowLayout>,
    /// `users.id` of whoever saved it, when a human did. `None` for a
    /// system-authored save (an import, say).
    pub created_by: Option<Uuid>,
}

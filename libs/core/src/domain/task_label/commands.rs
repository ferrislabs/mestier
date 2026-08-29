use authz::Subject;

use crate::{OrganizationId, domain::task_label::TaskLabelId};

#[derive(Debug, Clone)]
pub struct CreateTaskLabelCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct UpdateTaskLabelCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: TaskLabelId,
    pub name: String,
    pub color: String,
}

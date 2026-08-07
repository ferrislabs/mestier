use crate::{OrganizationId, domain::task_label::TaskLabelId};

#[derive(Debug, Clone)]
pub struct CreateTaskLabelCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct UpdateTaskLabelCommand {
    pub id: TaskLabelId,
    pub name: String,
    pub color: String,
}

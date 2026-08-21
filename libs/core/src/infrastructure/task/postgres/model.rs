use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId, QuoteId, Task,
    TaskAssignment, TaskAssignmentId, TaskId, TaskStatus,
};

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub parent_task_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: String,
    pub blocks_availability: bool,
    pub customer_id: Option<Uuid>,
    pub customer_context_id: Option<Uuid>,
    pub quote_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub expenses_cents: i32,
    pub expenses_label: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskRow {
    pub fn into_task(self, assignments: Vec<TaskAssignment>) -> Result<Task, CoreError> {
        let status = TaskStatus::from_str(&self.status)
            .map_err(|e| CoreError::Internal(format!("invalid task status in database: {e}")))?;

        Ok(Task {
            id: TaskId(self.id),
            organization_id: OrganizationId(self.org_id),
            parent_task_id: self.parent_task_id.map(TaskId),
            title: self.title,
            description: self.description,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            status,
            blocks_availability: self.blocks_availability,
            customer_id: self.customer_id.map(CustomerId),
            customer_context_id: self.customer_context_id.map(CustomerContextId),
            quote_id: self.quote_id.map(QuoteId),
            project_id: self.project_id.map(ProjectId),
            expenses_cents: self.expenses_cents,
            expenses_label: self.expenses_label,
            assignments,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TaskAssignmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub task_id: Uuid,
    pub member_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<TaskAssignmentRow> for TaskAssignment {
    fn from(row: TaskAssignmentRow) -> Self {
        Self {
            id: TaskAssignmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            task_id: TaskId(row.task_id),
            member_id: MemberId(row.member_id),
            created_at: row.created_at,
        }
    }
}

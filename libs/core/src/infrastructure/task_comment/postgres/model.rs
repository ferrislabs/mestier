use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{OrganizationId, TaskComment, TaskCommentId, TaskId, UserId};

#[derive(Debug, Clone)]
pub struct TaskCommentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub task_id: Uuid,
    pub author_user_id: Uuid,
    pub body: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaskCommentRow> for TaskComment {
    fn from(row: TaskCommentRow) -> Self {
        Self {
            id: TaskCommentId(row.id),
            organization_id: OrganizationId(row.org_id),
            task_id: TaskId(row.task_id),
            author_user_id: UserId(row.author_user_id),
            body: row.body,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

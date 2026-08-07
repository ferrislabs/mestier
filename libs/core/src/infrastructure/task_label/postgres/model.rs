use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{OrganizationId, TaskLabel, TaskLabelId};

#[derive(Debug, Clone)]
pub struct TaskLabelRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaskLabelRow> for TaskLabel {
    fn from(row: TaskLabelRow) -> Self {
        Self {
            id: TaskLabelId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            color: row.color,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

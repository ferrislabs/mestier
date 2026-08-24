use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{PlanningTask, TaskAssignment, infrastructure::task::postgres::model::TaskRow};

/// A `tasks` row projected for the planning read model: `TaskRow`'s own
/// columns plus the fields it does not itself carry — `customer_name`/
/// `context_label` (both `None` for a task with no customer, via the `LEFT
/// JOIN`) and `child_count`. Delegates to `TaskRow::into_task` for the
/// shared status-parsing logic rather than duplicating it — `task` (T1) is
/// read-only from here, never modified.
#[derive(Debug, Clone)]
pub struct PlanningTaskRow {
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
    pub recurrence_id: Option<Uuid>,
    pub occurrence_date: Option<chrono::NaiveDate>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub customer_name: Option<String>,
    pub context_label: Option<String>,
    pub child_count: i64,
}

impl PlanningTaskRow {
    pub fn into_planning_task(
        self,
        assignments: Vec<TaskAssignment>,
    ) -> Result<PlanningTask, CoreError> {
        let customer_name = self.customer_name;
        let context_label = self.context_label;
        let child_count = self.child_count;
        let row = TaskRow {
            id: self.id,
            org_id: self.org_id,
            parent_task_id: self.parent_task_id,
            title: self.title,
            description: self.description,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            status: self.status,
            blocks_availability: self.blocks_availability,
            customer_id: self.customer_id,
            customer_context_id: self.customer_context_id,
            quote_id: self.quote_id,
            project_id: self.project_id,
            expenses_cents: self.expenses_cents,
            expenses_label: self.expenses_label,
            recurrence_id: self.recurrence_id,
            occurrence_date: self.occurrence_date,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        Ok(PlanningTask {
            task: row.into_task(assignments)?,
            customer_name,
            context_label,
            child_count,
        })
    }
}

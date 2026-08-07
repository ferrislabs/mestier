use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Assignment, PlanningWorkOrder, infrastructure::work_order::postgres::model::WorkOrderRow,
};

/// A `work_orders` row projected for the planning read model:
/// `WorkOrderRow`'s own columns plus the two joined display fields it does
/// not itself carry, `customer_name` and `context_label`. Delegates to
/// `WorkOrderRow::into_work_order` for the shared status-parsing logic
/// rather than duplicating it — `work_order` (W1) is read-only from here,
/// never modified.
#[derive(Debug, Clone)]
pub struct PlanningWorkOrderRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    pub quote_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub status: String,
    pub title: Option<String>,
    pub note: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub customer_name: String,
    pub context_label: String,
}

impl PlanningWorkOrderRow {
    pub fn into_planning_work_order(
        self,
        assignments: Vec<Assignment>,
    ) -> Result<PlanningWorkOrder, CoreError> {
        let customer_name = self.customer_name;
        let context_label = self.context_label;
        let row = WorkOrderRow {
            id: self.id,
            org_id: self.org_id,
            customer_id: self.customer_id,
            customer_context_id: self.customer_context_id,
            quote_id: self.quote_id,
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            status: self.status,
            title: self.title,
            note: self.note,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        Ok(PlanningWorkOrder {
            work_order: row.into_work_order(assignments)?,
            customer_name,
            context_label,
        })
    }
}

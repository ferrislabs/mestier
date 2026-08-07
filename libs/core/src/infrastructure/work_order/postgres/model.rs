use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Assignment, AssignmentId, CustomerContextId, CustomerId, EmployeeId, EquipmentId,
    OrganizationId, QuoteId, WorkOrder, WorkOrderEquipment, WorkOrderEquipmentId, WorkOrderId,
    WorkOrderStatus,
};

#[derive(Debug, Clone)]
pub struct WorkOrderRow {
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
}

impl WorkOrderRow {
    pub fn into_work_order(
        self,
        assignments: Vec<Assignment>,
        equipment: Vec<WorkOrderEquipment>,
    ) -> Result<WorkOrder, CoreError> {
        let status = WorkOrderStatus::from_str(&self.status).map_err(|e| {
            CoreError::Internal(format!("invalid work order status in database: {e}"))
        })?;

        Ok(WorkOrder {
            id: WorkOrderId(self.id),
            organization_id: OrganizationId(self.org_id),
            customer_id: CustomerId(self.customer_id),
            customer_context_id: CustomerContextId(self.customer_context_id),
            quote_id: self.quote_id.map(QuoteId),
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            all_day: self.all_day,
            status,
            title: self.title,
            note: self.note,
            assignments,
            equipment,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub work_order_id: Uuid,
    pub employee_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<AssignmentRow> for Assignment {
    fn from(row: AssignmentRow) -> Self {
        Self {
            id: AssignmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            work_order_id: WorkOrderId(row.work_order_id),
            employee_id: EmployeeId(row.employee_id),
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkOrderEquipmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub work_order_id: Uuid,
    pub equipment_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<WorkOrderEquipmentRow> for WorkOrderEquipment {
    fn from(row: WorkOrderEquipmentRow) -> Self {
        Self {
            id: WorkOrderEquipmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            work_order_id: WorkOrderId(row.work_order_id),
            equipment_id: EquipmentId(row.equipment_id),
            created_at: row.created_at,
        }
    }
}

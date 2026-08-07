use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{EmployeeId, OrganizationId, TimeEntry, TimeEntryId, WorkOrderId};

#[derive(Debug, Clone)]
pub struct TimeEntryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub work_order_id: Uuid,
    pub employee_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub photos_before: Vec<String>,
    pub photos_during: Vec<String>,
    pub photos_after: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimeEntryRow {
    pub fn into_time_entry(self) -> TimeEntry {
        TimeEntry {
            id: TimeEntryId(self.id),
            organization_id: OrganizationId(self.org_id),
            work_order_id: WorkOrderId(self.work_order_id),
            employee_id: EmployeeId(self.employee_id),
            started_at: self.started_at,
            ended_at: self.ended_at,
            photos_before: self.photos_before,
            photos_during: self.photos_during,
            photos_after: self.photos_after,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

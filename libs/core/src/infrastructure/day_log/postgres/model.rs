use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{DayLog, DayLogId, EmployeeId, OrganizationId};

#[derive(Debug, Clone)]
pub struct DayLogRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub employee_id: Uuid,
    pub work_date: NaiveDate,
    pub ended_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl DayLogRow {
    pub fn into_day_log(self) -> DayLog {
        DayLog {
            id: DayLogId(self.id),
            organization_id: OrganizationId(self.org_id),
            employee_id: EmployeeId(self.employee_id),
            work_date: self.work_date,
            ended_at: self.ended_at,
            created_at: self.created_at,
        }
    }
}

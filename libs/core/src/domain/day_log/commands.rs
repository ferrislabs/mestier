use chrono::{DateTime, NaiveDate, Utc};

use crate::{EmployeeId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CloseDayCommand {
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub work_date: NaiveDate,
    pub ended_at: Option<DateTime<Utc>>,
}

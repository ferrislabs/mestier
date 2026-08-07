use common::CoreError;

use crate::{DayLog, DayLogId, EmployeeId, OrganizationId};
use chrono::NaiveDate;

#[cfg_attr(test, mockall::automock)]
pub trait DayLogRepository: Send {
    fn insert(&mut self, day_log: &DayLog) -> impl Future<Output = Result<DayLog, CoreError>> + Send;

    #[allow(dead_code)]
    fn find_by_id(
        &mut self,
        id: DayLogId,
    ) -> impl Future<Output = Result<Option<DayLog>, CoreError>> + Send;

    #[allow(dead_code)]
    fn find_by_employee_and_date(
        &mut self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> impl Future<Output = Result<Option<DayLog>, CoreError>> + Send;
}

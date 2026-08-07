use common::CoreError;

use crate::{EmployeeId, OrganizationId, TimeEntry, TimeEntryId};

#[cfg_attr(test, mockall::automock)]
pub trait TimeEntryRepository: Send {
    fn insert(
        &mut self,
        time_entry: &TimeEntry,
    ) -> impl Future<Output = Result<TimeEntry, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TimeEntryId,
    ) -> impl Future<Output = Result<Option<TimeEntry>, CoreError>> + Send;

    /// The single open (`ended_at IS NULL`) entry for this employee, if any.
    fn find_active_by_employee(
        &mut self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
    ) -> impl Future<Output = Result<Option<TimeEntry>, CoreError>> + Send;

    fn update(
        &mut self,
        time_entry: &TimeEntry,
    ) -> impl Future<Output = Result<TimeEntry, CoreError>> + Send;
}

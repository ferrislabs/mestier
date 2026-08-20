use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{DayLog, EmployeeId, OrganizationId, TimeEntry, TimeEntryId, TimeEntryPhoto};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TimeEntryRepository: Send {
    fn insert(
        &mut self,
        entry: &TimeEntry,
    ) -> impl Future<Output = Result<TimeEntry, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TimeEntryId,
    ) -> impl Future<Output = Result<Option<TimeEntry>, CoreError>> + Send;

    /// The employee's open entry, if they have one.
    ///
    /// Named for the question rather than the query: "is this person already
    /// clocked on somewhere" is the rule, and a partial unique index already
    /// guarantees the answer is at most one row.
    fn find_running_for_employee(
        &mut self,
        employee_id: EmployeeId,
    ) -> impl Future<Output = Result<Option<TimeEntry>, CoreError>> + Send;

    fn list_for_employee_on(
        &mut self,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<TimeEntry>, CoreError>> + Send;

    /// Closes a running entry.
    ///
    /// `after_the_fact` records that the end was declared on a later day than
    /// the work, so a reader can tell a recollection from a measurement.
    fn close(
        &mut self,
        id: TimeEntryId,
        ended_at: DateTime<Utc>,
        after_the_fact: bool,
    ) -> impl Future<Output = Result<TimeEntry, CoreError>> + Send;

    fn attach_photo(
        &mut self,
        photo: &TimeEntryPhoto,
    ) -> impl Future<Output = Result<TimeEntryPhoto, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DayLogRepository: Send {
    fn upsert(
        &mut self,
        day_log: &DayLog,
    ) -> impl Future<Output = Result<DayLog, CoreError>> + Send;

    fn find_for_employee_on(
        &mut self,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> impl Future<Output = Result<Option<DayLog>, CoreError>> + Send;

    fn list_by_organization_on(
        &mut self,
        organization_id: OrganizationId,
        work_date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<DayLog>, CoreError>> + Send;
}

use chrono::NaiveDate;
use common::CoreError;

use crate::{
    EmployeeId, OrganizationId,
    domain::work_time::{EmployeeRhythm, EmployeeRhythmId, EmployeeWorkSlot},
};

#[cfg_attr(test, mockall::automock)]
pub trait RhythmRepository: Send {
    /// The employee's currently open rhythm version (`effective_to IS
    /// NULL`), if any. By construction of `WorkTimeService::replace_rhythm`,
    /// at most one open version exists per employee at a time.
    fn find_open_by_employee(
        &mut self,
        employee_id: EmployeeId,
    ) -> impl Future<Output = Result<Option<EmployeeRhythm>, CoreError>> + Send;

    /// Every rhythm version whose `[effective_from, effective_to)` window
    /// overlaps `[from, to]`, including the one currently in effect —
    /// mirrors the `GET work-time` contract, which returns whole versions
    /// rather than windowing them like `employee_work_slots`.
    fn find_overlapping(
        &mut self,
        employee_id: EmployeeId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> impl Future<Output = Result<Vec<EmployeeRhythm>, CoreError>> + Send;

    /// Persists a brand new rhythm version together with its slots.
    fn insert(
        &mut self,
        rhythm: &EmployeeRhythm,
    ) -> impl Future<Output = Result<EmployeeRhythm, CoreError>> + Send;

    /// Replaces an existing version's own fields and its complete slot list
    /// — physical delete then insert of the slots, matching the `PUT`
    /// contract, where the payload is never a delta.
    fn update(
        &mut self,
        rhythm: &EmployeeRhythm,
    ) -> impl Future<Output = Result<EmployeeRhythm, CoreError>> + Send;

    /// Closes an open version by setting `effective_to`, without touching
    /// its slots — used when a later version takes over so history stays
    /// readable.
    fn set_effective_to(
        &mut self,
        id: EmployeeRhythmId,
        effective_to: NaiveDate,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait WorkSlotRepository: Send {
    /// Work slots pinned inside `[from, to]` — windowed, unlike rhythm
    /// versions.
    fn list_in_window(
        &mut self,
        employee_id: EmployeeId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> impl Future<Output = Result<Vec<EmployeeWorkSlot>, CoreError>> + Send;

    /// Full replace of the `[from, to]` window: physical delete of whatever
    /// is there, then insert of the given set — the `PUT` contract treats
    /// the window's content as never a delta.
    fn replace_window(
        &mut self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        from: NaiveDate,
        to: NaiveDate,
        slots: &[EmployeeWorkSlot],
    ) -> impl Future<Output = Result<Vec<EmployeeWorkSlot>, CoreError>> + Send;
}

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    Employee, EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, MemberId, OrganizationId,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait EmployeeRepository: Send {
    fn insert(
        &mut self,
        employee: &Employee,
    ) -> impl Future<Output = Result<Employee, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: EmployeeId,
    ) -> impl Future<Output = Result<Option<Employee>, CoreError>> + Send;

    /// The active contractual profile attached to `member_id`, if any. A member
    /// without one is not an error: they are plannable, they simply have no
    /// cost. Backs the upsert — attaching a profile to a member who already has
    /// one updates it instead of opening a second.
    fn find_by_member_id(
        &mut self,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Option<Employee>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Employee>, u64), CoreError>> + Send;

    /// Every active profile of the organization, unpaginated. The planning read
    /// model needs the whole set in one query, to attach a rate to the members
    /// that have one — never one page at a time (see the planning module design
    /// doc).
    fn list_active_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Employee>, CoreError>> + Send;

    fn update(
        &mut self,
        employee: &Employee,
    ) -> impl Future<Output = Result<Employee, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: EmployeeId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Persists the versioned history of what an employee costs. Mirrors
/// [`crate::domain::work_time::ports::RhythmRepository`]'s shape: an
/// employee's history is versions of the same kind of fact a rhythm is,
/// closed and reopened the same way.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait EmployeeCostBasisRepository: Send {
    /// The employee's currently open cost basis version (`effective_to IS
    /// NULL`), if any. By construction of
    /// `EmployeeCostBasisService::set_cost_basis`, at most one open version
    /// exists per employee at a time — the database enforces it too, via
    /// `uq_employee_cost_bases_open_version`.
    fn find_open_by_employee(
        &mut self,
        employee_id: EmployeeId,
    ) -> impl Future<Output = Result<Option<EmployeeCostBasis>, CoreError>> + Send;

    /// Persists a brand new cost basis version.
    fn insert(
        &mut self,
        basis: &EmployeeCostBasis,
    ) -> impl Future<Output = Result<EmployeeCostBasis, CoreError>> + Send;

    /// Replaces an existing version's own fields in place — used when the
    /// same `effective_from` is set again, so calling `set_cost_basis` twice
    /// in a row for the same date edits one row rather than accumulating a
    /// second.
    fn update(
        &mut self,
        basis: &EmployeeCostBasis,
    ) -> impl Future<Output = Result<EmployeeCostBasis, CoreError>> + Send;

    /// Closes an open version by setting `effective_to` — used when a later
    /// version takes over, so the closed version survives as history.
    fn set_effective_to(
        &mut self,
        id: EmployeeCostBasisId,
        effective_to: NaiveDate,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

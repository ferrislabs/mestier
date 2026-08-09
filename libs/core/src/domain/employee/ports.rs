use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Employee, EmployeeId, MemberId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
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

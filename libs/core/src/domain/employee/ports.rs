use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Employee, EmployeeId, OrganizationId, UserId};

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

    /// The active employee record linked to `user_id` within `organization_id`,
    /// if any. Backs the planning module's dedup rule ("a person who is both a
    /// member and an employee appears once, on the employee side") and the
    /// `PATCH /work-orders/{id}` on-the-fly employee provisioning: a `member`
    /// assignee reuses this record instead of creating a duplicate one.
    fn find_by_user_id(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Employee>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Employee>, u64), CoreError>> + Send;

    /// Every active employee of the organization, unpaginated. Added
    /// additively for the planning read model, which always needs the
    /// whole roster in one query — to dedup against organization members,
    /// never one page at a time (see the planning module design doc).
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

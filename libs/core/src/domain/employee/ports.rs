use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Employee, EmployeeId, OrganizationId};

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

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Employee>, u64), CoreError>> + Send;

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

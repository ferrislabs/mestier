use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{EmployeeAbsence, EmployeeAbsenceId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
pub trait AbsenceRepository: Send {
    fn insert(
        &mut self,
        absence: &EmployeeAbsence,
    ) -> impl Future<Output = Result<EmployeeAbsence, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: EmployeeAbsenceId,
    ) -> impl Future<Output = Result<Option<EmployeeAbsence>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<EmployeeAbsence>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        absence: &EmployeeAbsence,
    ) -> impl Future<Output = Result<EmployeeAbsence, CoreError>> + Send;

    /// Logical delete, unlike `assignments`: an absence is business data the
    /// user edits, so its history stays queryable in the row rather than
    /// being physically removed.
    fn soft_delete(
        &mut self,
        id: EmployeeAbsenceId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

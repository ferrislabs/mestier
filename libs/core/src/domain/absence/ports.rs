use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Absence, AbsenceId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AbsenceRepository: Send {
    fn insert(
        &mut self,
        absence: &Absence,
    ) -> impl Future<Output = Result<Absence, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: AbsenceId,
    ) -> impl Future<Output = Result<Option<Absence>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Absence>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        absence: &Absence,
    ) -> impl Future<Output = Result<Absence, CoreError>> + Send;

    /// Logical delete, unlike `assignments`: an absence is business data the
    /// user edits, so its history stays queryable in the row rather than
    /// being physically removed.
    fn soft_delete(
        &mut self,
        id: AbsenceId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

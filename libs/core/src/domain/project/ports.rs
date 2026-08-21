use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{CustomerId, OrganizationId, Project, ProjectId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ProjectRepository: Send {
    fn insert(
        &mut self,
        project: &Project,
    ) -> impl Future<Output = Result<Project, CoreError>> + Send;

    /// Returns an archived project like any other. Unlike the soft-deleted
    /// rows other repositories filter out, an archived project is still a
    /// legitimate read target: profitability reports on periods during which
    /// it was worked, and the figures have to keep resolving.
    fn find_by_id(
        &mut self,
        id: ProjectId,
    ) -> impl Future<Output = Result<Option<Project>, CoreError>> + Send;

    /// One page of the organization's projects, plus the total.
    ///
    /// `include_archived` defaults to excluding them at the call site rather
    /// than here, so a picker and a report can ask the same question and get
    /// the answer each needs.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        customer_id: Option<CustomerId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Project>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        project: &Project,
    ) -> impl Future<Output = Result<Project, CoreError>> + Send;

    /// Sets or clears `archived_at`. One method for both directions: archiving
    /// is reversible, and a separate `restore` would be the same UPDATE with a
    /// different literal.
    fn set_archived_at(
        &mut self,
        id: ProjectId,
        archived_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
